use reqwest::Client;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Channel {
    Important,
    General,
    Minor,
}

pub trait NtfyConfig {
    fn access_token(&self) -> Option<&str>;

    fn channel(&self, channel: Channel) -> &str;
}

#[derive(Clone, Debug)]
pub struct NtfyClient {
    client: reqwest::Client,
    token: Option<String>,

    channel_important: String,
    channel_general: String,
    channel_minor: String,
}

impl NtfyClient {
    pub fn new(config: &impl NtfyConfig) -> Self {
        Self {
            client: Client::new(),
            token: config.access_token().map(ToOwned::to_owned),

            channel_important: config.channel(Channel::Important).to_owned(),
            channel_general: config.channel(Channel::General).to_owned(),
            channel_minor: config.channel(Channel::Minor).to_owned(),
        }
    }

    pub async fn send(&self, channel: Channel, message: String) -> Result<(), Error> {
        let channel = match channel {
            Channel::Important => &self.channel_important,
            Channel::General => &self.channel_general,
            Channel::Minor => &self.channel_minor,
        };

        let mut request = self
            .client
            .post(format!("https://ntfy.sh/{channel}"))
            .body(message);

        if let Some(token) = &self.token {
            request = request.header("Authorization", format!("Bearer {token}"));
        }

        self.client.execute(request.build()?).await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}
