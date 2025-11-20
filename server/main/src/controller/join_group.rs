use actix_web::{HttpResponse, web};
use chrono::Utc;
use dxe_data::queries::identity::get_group;
use dxe_data::types::GroupId;
use sqlx::SqlitePool;

use crate::models::Error;

const HTML_TEMPLATE: &str = r#"
<!DOCTYPE html>
<html lang="ko">
  <head>
    <meta charset="utf-8" />
    <title>{group_name} 그룹 가입 | 드림하우스 합주실</title>
    <meta http-equiv="refresh" content="0;url=/join/{group_id}" />
    <meta property="og:title" content="드림하우스 합주실" />
    <meta property="og:description" content="{group_name} 그룹에 가입해 주세요." />
    <meta property="og:type" content="website" />
    <meta property="og:image" content="/assets/og.png" />
    <meta property="og:locale" content="ko_KR" />
    <meta property="og:url" content="https://dream-house.kr" />
  </head>
  <body></body>
</html>
"#;

async fn redirect(
    group_id: web::Path<GroupId>,
    database: web::Data<SqlitePool>,
) -> Result<HttpResponse, Error> {
    let now = Utc::now();

    let mut connection = database.acquire().await?;

    let group = get_group(&mut *connection, &now, group_id.as_ref())
        .await?
        .ok_or(Error::GroupNotFound)?;

    let html = HTML_TEMPLATE
        .replace("{group_id}", group_id.to_string().as_str())
        .replace("{group_name}", &group.name);

    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

pub fn resource() -> actix_web::Resource {
    web::resource("/join/{group_id}").route(web::get().to(redirect))
}
