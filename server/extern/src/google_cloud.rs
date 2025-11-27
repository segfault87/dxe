pub mod calendar;
pub mod drive;

use std::path::PathBuf;
use std::sync::Arc;

use gcp_auth::{CustomServiceAccount, Token, TokenProvider};

pub trait GoogleCloudAuthConfig {
    fn service_account_path(&self) -> &PathBuf;
}

pub async fn get_token(
    config: &impl GoogleCloudAuthConfig,
    scopes: &[&str],
) -> Result<Arc<Token>, Error> {
    let service_account = CustomServiceAccount::from_file(config.service_account_path())?;

    Ok(service_account.token(scopes).await?)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Authentication error: {0}")]
    GcpAuth(#[from] gcp_auth::Error),
}
