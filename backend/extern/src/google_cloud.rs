pub mod calendar;
pub mod drive;

use std::path::PathBuf;
use std::sync::Arc;

use gcp_auth::{CustomServiceAccount, Token, TokenProvider};

pub trait GoogleCloudAuthConfig {
    fn service_account_path(&self) -> &PathBuf;
}

#[derive(Debug, Clone)]
pub struct CredentialManager {
    service_account: Arc<CustomServiceAccount>,
}

impl CredentialManager {
    pub fn new(config: &impl GoogleCloudAuthConfig, sub: Option<String>) -> Result<Self, Error> {
        let mut service_account = CustomServiceAccount::from_file(config.service_account_path())?;
        if let Some(sub) = sub {
            service_account = service_account.with_subject(sub);
        }
        Ok(Self {
            service_account: Arc::new(service_account),
        })
    }

    pub async fn get_token(&self, scopes: &[&str]) -> Result<Arc<Token>, Error> {
        Ok(self.service_account.token(scopes).await?)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Authentication error: {0}")]
    GcpAuth(#[from] gcp_auth::Error),
}
