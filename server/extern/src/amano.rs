use std::collections::HashMap;

use chrono::{Local, TimeDelta};
use reqwest::{
    StatusCode,
    header::{CONTENT_TYPE, HeaderValue},
    redirect::Policy,
};
use url::Url;

const PATH_LOGIN: &str = "/login";
const PATH_LIST: &str = "/discount/registration/listForDiscount";
const PATH_GET: &str = "/discount/registration/getForDiscount";
const PATH_EXEMPTION: &str = "/discount/registration/save";

pub struct AmanoClient {
    client: reqwest::Client,

    url_base: Url,
    lot_id: String,
    user_id: String,
    hashed_password: String,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CarParkExemptionResult {
    NotFound,
    AlreadyApplied,
    Success,
}

impl AmanoClient {
    pub fn new(config: &impl AmanoConfig) -> Self {
        Self {
            client: reqwest::ClientBuilder::new()
                .redirect(Policy::none())
                .cookie_store(true)
                .build()
                .unwrap(),

            url_base: config.url_base().clone(),
            lot_id: config.lot_id().to_owned(),
            user_id: config.user_id().to_owned(),
            hashed_password: config.hashed_password().to_owned(),
        }
    }

    async fn refresh_session_if_required(&self) -> Result<(), Error> {
        let mut url = self.url_base.clone();
        url.set_path(PATH_LOGIN);

        let result = self.client.get(url.clone()).send().await?;
        if result.status() == StatusCode::OK {
            let login_response = self
                .client
                .post(url)
                .form(&HashMap::from([
                    ("userId", self.user_id.clone()),
                    ("userPwd", self.hashed_password.clone()),
                ]))
                .send()
                .await?;

            if login_response.status() != StatusCode::FOUND {
                return Err(Error::RefreshSession(login_response.status()));
            }
        }

        Ok(())
    }

    pub async fn exempt(
        &self,
        license_plate_number: &str,
    ) -> Result<CarParkExemptionResult, Error> {
        self.refresh_session_if_required().await?;

        let yesterday = Local::now() - TimeDelta::days(1);
        let entry_date = yesterday.format("%Y%m%d").to_string();

        let mut list_url = self.url_base.clone();
        list_url.set_path(PATH_LIST);

        let mut request = self
            .client
            .post(list_url.clone())
            .form(&HashMap::from([
                ("iLotArea", self.lot_id.clone()),
                ("entryDate", entry_date.clone()),
                ("carNo", license_plate_number.to_owned()),
            ]))
            .build()?;
        *request.headers_mut().get_mut(CONTENT_TYPE).unwrap() =
            HeaderValue::from_static("application/x-www-form-urlencoded; charset=UTF-8");
        let result = self.client.execute(request).await?;

        let json_body: Vec<serde_json::Value> = result.json().await?;
        if json_body.len() > 1 {
            return Err(Error::MultipleOccurrencesFound);
        }

        let Some(entry) = json_body.first() else {
            return Ok(CarParkExemptionResult::NotFound);
        };
        let entry_object = entry.as_object().ok_or(Error::InvalidJsonType("0"))?;

        let discount_count = entry_object
            .get("dscnt_cnt")
            .ok_or(Error::MissingField("dscnt_cnt", list_url.clone()))?
            .as_str()
            .ok_or(Error::InvalidJsonType("dscnt_cnt"))?
            .parse::<i32>()
            .map_err(|_| Error::InvalidJsonType("dscnt_cnt"))?;

        if discount_count > 0 {
            return Ok(CarParkExemptionResult::AlreadyApplied);
        }

        let pe_id = entry_object
            .get("id")
            .ok_or(Error::MissingField("id", list_url.clone()))?
            .as_i64()
            .ok_or(Error::InvalidJsonType("id"))?
            .to_string();
        let card_type = entry_object
            .get("iCardType")
            .ok_or(Error::MissingField("iCardType", list_url.clone()))?
            .as_str()
            .ok_or(Error::InvalidJsonType("iCardtype"))?
            .to_string();

        let mut get_url = self.url_base.clone();
        get_url.set_path(PATH_GET);

        let mut request = self
            .client
            .post(get_url.clone())
            .form(&HashMap::from([
                ("id", pe_id.to_owned()),
                ("iCardType", card_type.clone()),
                ("member_id", self.user_id.clone()),
                ("startDate", entry_date.clone()),
            ]))
            .build()?;
        *request.headers_mut().get_mut(CONTENT_TYPE).unwrap() =
            HeaderValue::from_static("application/x-www-form-urlencoded; charset=UTF-8");
        let result = self.client.execute(request).await?;

        let details_data: serde_json::Map<String, serde_json::Value> = result.json().await?;
        let list_discount_type = details_data
            .get("listDiscountType")
            .ok_or(Error::MissingField("listDiscountType", get_url.clone()))?
            .as_array()
            .ok_or(Error::InvalidJsonType("listDiscountType"))?;
        let park_entry = details_data
            .get("parkEntry")
            .ok_or(Error::MissingField("parkEntry", get_url.clone()))?
            .as_object()
            .ok_or(Error::InvalidJsonType("parkEntry"))?;

        let mut free_discount_ids = list_discount_type.iter().filter_map(|v| {
            if let Some(object) = v.as_object() {
                if object.get("iLotArea") == Some(&serde_json::Value::String(self.lot_id.clone()))
                    && object.get("discount_price")
                        == Some(&serde_json::Value::Number(
                            serde_json::Number::from_i128(0).unwrap(),
                        ))
                {
                    object.get("id").map(|v| v.as_str())
                } else {
                    None
                }
            } else {
                None
            }
        });

        let free_discount_id = free_discount_ids
            .next()
            .flatten()
            .ok_or(Error::NoFreeDiscountFound)?
            .to_owned();

        let car_no = park_entry
            .get("acPlate1")
            .ok_or(Error::MissingField("acPlate1", get_url.clone()))?
            .as_str()
            .ok_or(Error::InvalidJsonType("acPlate1"))?
            .to_owned();
        let ac_plate_2 = park_entry
            .get("acPlate2")
            .ok_or(Error::MissingField("acPlate2", get_url.clone()))?
            .as_str()
            .unwrap_or("")
            .to_owned();

        let mut exemption_url = self.url_base.clone();
        exemption_url.set_path(PATH_EXEMPTION);

        let mut request = self
            .client
            .post(exemption_url)
            .form(&HashMap::from([
                ("peId", pe_id),
                ("discountType", free_discount_id),
                ("saveCnt", "1".to_owned()),
                ("iCardType", card_type),
                ("carNo", car_no),
                ("acPlate2", ac_plate_2),
                ("memo", String::new()),
            ]))
            .build()?;
        *request.headers_mut().get_mut(CONTENT_TYPE).unwrap() =
            HeaderValue::from_static("application/x-www-form-urlencoded; charset=UTF-8");
        let exemption_result = self.client.execute(request).await?;

        if exemption_result.status() != StatusCode::OK {
            Err(Error::ExemptionFailed(exemption_result.status()))
        } else {
            Ok(CarParkExemptionResult::Success)
        }
    }
}

pub trait AmanoConfig {
    fn url_base(&self) -> &Url;
    fn lot_id(&self) -> &str;
    fn user_id(&self) -> &str;
    fn hashed_password(&self) -> &str;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Could not refresh session: {0}")]
    RefreshSession(StatusCode),
    #[error("Missing field {0} at url: {1}")]
    MissingField(&'static str, Url),
    #[error("Invalid JSON type: {0}")]
    InvalidJsonType(&'static str),
    #[error("Multiple occurrences found")]
    MultipleOccurrencesFound,
    #[error("No free discount found")]
    NoFreeDiscountFound,
    #[error("Failed to apply exemption")]
    ExemptionFailed(StatusCode),
}
