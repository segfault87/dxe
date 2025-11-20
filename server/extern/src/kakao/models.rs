use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Hash, Eq, PartialEq)]
pub enum AccountPropertyKey {
    #[serde(rename = "kakao_account.profile")]
    Profile,
    #[serde(rename = "kakao_account.name")]
    Name,
    #[serde(rename = "kakao_account.email")]
    Email,
    #[serde(rename = "kakao_account.age_range")]
    AgeRange,
    #[serde(rename = "kakao_account.birthday")]
    Birthday,
    #[serde(rename = "kakao_account.gender")]
    Gender,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum BirthdayType {
    Solar,
    Lunar,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Gender {
    Female,
    Male,
}

#[derive(Debug, Deserialize)]
pub struct KakaoAccount {
    pub profile_needs_agreement: Option<bool>,
    pub profile_nickname_needs_agreement: Option<bool>,
    pub profile_image_needs_agreement: Option<bool>,
    pub profile: Option<Profile>,
    pub name_needs_agreement: Option<bool>,
    pub name: Option<String>,
    pub email_needs_agreement: Option<bool>,
    pub is_email_valid: Option<bool>,
    pub is_email_verified: Option<bool>,
    pub email: Option<String>,
    pub age_range_needs_agreement: Option<bool>,
    pub age_range: Option<String>,
    pub birthyear_needs_agreement: Option<bool>,
    pub birthyear: Option<String>,
    pub birthday_needs_agreement: Option<bool>,
    pub birthday: Option<String>,
    pub birthday_type: Option<BirthdayType>,
    pub is_leap_month: Option<bool>,
    pub gender_needs_agreement: Option<bool>,
    pub gender: Option<Gender>,
    pub phone_number_needs_agreement: Option<bool>,
    pub phone_number: Option<String>,
    pub ci_needs_agreement: Option<bool>,
    pub ci: Option<String>,
    pub ci_authenticated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct Profile {
    pub nickname: Option<String>,
    pub thumbnail_image_url: Option<String>,
    pub profile_image_url: Option<String>,
    pub is_default_image: Option<bool>,
    pub is_default_nickname: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct Partner {
    pub uuid: Option<uuid::Uuid>,
}
