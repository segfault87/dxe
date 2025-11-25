use dxe_types::{SpaceId, UnitId};
use sqlx::SqliteConnection;

use crate::Error;
use crate::entities::{Space, Unit};

pub async fn get_space_by_unit_id(
    connection: &mut SqliteConnection,
    unit_id: &UnitId,
) -> Result<Option<Space>, Error> {
    Ok(sqlx::query_as!(
        Space,
        r#"
        SELECT
            s.id AS "id: SpaceId",
            s.enabled
        FROM unit "u"
        JOIN space "s" ON u.space_id = s.id
        WHERE u.id = ?1
        "#,
        unit_id
    )
    .fetch_optional(&mut *connection)
    .await?)
}

pub async fn get_units_by_space_id(
    connection: &mut SqliteConnection,
    space_id: &SpaceId,
) -> Result<Vec<Unit>, Error> {
    Ok(sqlx::query_as!(
        Unit,
        r#"
        SELECT
            u.id AS "id: UnitId",
            u.space_id AS "space_id: SpaceId",
            u.enabled
        FROM space "s"
        JOIN unit "u" ON u.space_id = s.id
        WHERE s.id = ?1
        "#,
        space_id
    )
    .fetch_all(&mut *connection)
    .await?)
}

pub async fn is_unit_enabled(
    connection: &mut SqliteConnection,
    unit_id: &UnitId,
) -> Result<Option<bool>, Error> {
    let result = sqlx::query!(
        r#"
        SELECT
            enabled
        FROM unit
        WHERE id = ?1
        "#,
        unit_id
    )
    .fetch_optional(&mut *connection)
    .await?;

    Ok(result.map(|v| v.enabled))
}
