use dxe_data::entities::Identity;
use dxe_data::queries::identity::is_member_of;
use dxe_data::types::UserId;
use sqlx::SqliteConnection;

use crate::models::Error;

pub async fn check_membership(
    connection: &mut SqliteConnection,
    user_id: &UserId,
    identity: &Identity,
) -> Result<bool, Error> {
    match identity {
        Identity::User(u) => Ok(&u.id == &user_id.clone().into()),
        Identity::Group(g) => Ok(is_member_of(&mut *connection, &g.id, user_id).await?),
    }
}
