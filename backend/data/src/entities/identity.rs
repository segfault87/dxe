use chrono::{DateTime, Utc};
use dxe_types::{GroupId, IdentityId, IdentityProvider, UserId};
use sqlx::FromRow;

#[derive(Clone, Debug, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum IdentityDiscriminator {
    User,
    Group,
}

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: UserId,
    pub provider: IdentityProvider,
    pub foreign_id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub deactivated_at: Option<DateTime<Utc>>,
    pub license_plate_number: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct Group {
    pub id: GroupId,
    pub name: String,
    pub owner_id: UserId,
    pub is_open: bool,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub enum Identity {
    User(User),
    Group(Group),
}

impl Identity {
    pub fn id(&self) -> IdentityId {
        match self {
            Identity::User(u) => u.id.into(),
            Identity::Group(g) => g.id.into(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Identity::User(u) => &u.name,
            Identity::Group(g) => &g.name,
        }
    }
}

#[derive(Debug, FromRow)]
pub struct GroupAssociation {
    pub group_id: GroupId,
    pub user_id: UserId,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct UserCashPaymentInformation {
    pub user_id: UserId,
    pub depositor_name: Option<String>,
    pub refund_account: Option<String>,
}

#[derive(Debug, FromRow)]
pub struct UserPlainCredential {
    pub user_id: UserId,
    pub handle: String,
    pub argon2_password: String,
}
