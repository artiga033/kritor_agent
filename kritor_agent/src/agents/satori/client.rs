use super::schema;

use reqwest::Url;
use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug)]
pub enum Error {
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    ServerSide,
    NonStandardResponseStatus,
    Reqwest(reqwest::Error),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for Error {}
impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value)
    }
}
impl From<reqwest::StatusCode> for Error {
    fn from(value: reqwest::StatusCode) -> Self {
        match value {
            reqwest::StatusCode::BAD_REQUEST => Self::BadRequest,
            reqwest::StatusCode::UNAUTHORIZED => Self::Unauthorized,
            reqwest::StatusCode::FORBIDDEN => Self::Forbidden,
            reqwest::StatusCode::NOT_FOUND => Self::NotFound,
            reqwest::StatusCode::METHOD_NOT_ALLOWED => Self::MethodNotAllowed,
            x if x.as_u16() >= 500 && x.as_u16() < 600 => Self::ServerSide,
            _ => Self::NonStandardResponseStatus,
        }
    }
}

pub struct SatoriClient {
    /// a reqwest client to be used to send http requests to a satori server.
    pub client: reqwest::Client,
    /// consists of the schema, host, port, path and version of the satori server.
    /// {scheme}://{host}:{port}/{path}/{version}/
    ///
    /// When it comes to the `path`, the tailing slash **MUST** be included.
    /// This is to make proper relative url for rpc calls.
    pub base_url: Url,
    pub token: Option<String>,
}

impl SatoriClient {
    #[inline]
    async fn rpc<Resp: DeserializeOwned>(&self, endpoint: &str) -> Result<Resp, Error> {
        self.make_rpc_req(endpoint, None::<()>).await
    }
    #[inline]
    async fn rpc_with<Req: Serialize, Resp: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: Req,
    ) -> Result<Resp, Error> {
        self.make_rpc_req(endpoint, Some(body)).await
    }
    async fn make_rpc_req<Req: Serialize, Resp: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: Option<Req>,
    ) -> Result<Resp, Error> {
        let url = self.base_url.join(endpoint).unwrap();
        let mut req = self.client.post(url);
        if let Some(token) = &self.token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }
        if let Some(body) = body {
            req = req.json(&body);
        }

        let resp = req.send().await?;
        if !resp.status().is_success() {
            return Err(resp.status().into());
        }
        Ok(resp.json().await?)
    }

    pub async fn login_get(&self) -> Result<schema::Login, Error> {
        self.rpc("login.get").await
    }

    pub async fn message_create(
        &self,
        channel_id: String,
        content: String,
    ) -> Result<Vec<schema::Message>, Error> {
        #[derive(Serialize)]
        struct Req {
            channel_id: String,
            content: String,
        }
        self.rpc_with(
            "message.create",
            Req {
                channel_id,
                content,
            },
        )
        .await
    }
}
