use chrono::{DateTime, Utc};
use dxe_types::entities::MixerPreferences;
use dxe_types::{IdentityId, UnitId};
use sqlx::SqliteConnection;
use sqlx::types::Json;

use crate::Error;
use crate::entities::MixerConfig;

pub async fn create_or_update_mixer_config(
    connection: &mut SqliteConnection,
    now: &DateTime<Utc>,
    identity_id: IdentityId,
    unit_id: &UnitId,
    data: &MixerPreferences,
) -> Result<bool, Error> {
    let data = Json(data);
    let result = sqlx::query!(
        r#"
        INSERT INTO mixer_config(identity_id, unit_id, data, created_at, updated_at)
        VALUES($1, $2, $3, $4, $4)
        ON CONFLICT(identity_id, unit_id)
        DO UPDATE SET
            data = excluded.data,
            updated_at = excluded.updated_at
        "#,
        identity_id,
        unit_id,
        data,
        now
    )
    .execute(&mut *connection)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn get_mixer_config(
    connection: &mut SqliteConnection,
    identity_id: IdentityId,
    unit_id: &UnitId,
) -> Result<Option<MixerConfig>, Error> {
    Ok(sqlx::query_as!(
        MixerConfig,
        r#"
        SELECT
            identity_id AS "identity_id: IdentityId",
            unit_id AS "unit_id: UnitId",
            data AS "data: Json<MixerPreferences>",
            created_at AS "created_at: DateTime<Utc>",
            updated_at AS "updated_at: DateTime<Utc>"
        FROM
            mixer_config
        WHERE
            identity_id = ?1 AND
            unit_id = ?2
        "#,
        identity_id,
        unit_id
    )
    .fetch_optional(&mut *connection)
    .await?)
}
