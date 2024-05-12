mod client;
pub use client::SatoriClient;
mod message;
pub mod schema;

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
#[allow(unused_imports)]
use kritor::{
    auth::{authentication_service_server::AuthenticationService, *},
    common::Scene,
    core::{core_service_server::CoreService, *},
    developer::{developer_service_server::DeveloperService, *},
    event::{event_service_server::EventService, *},
    file::{group_file_service_server::GroupFileService, *},
    friend::{friend_service_server::FriendService, *},
    group::{group_service_server::GroupService, *},
    guild::{guild_service_server::GuildService, *},
    message::{message_service_server::MessageService, *},
    process::{process_service_server::ProcessService, *},
    reverse::{reverse_service_server::ReverseService, *},
    web::{web_service_server::WebService, *},
};
use serde_json::json;
use tokio::{select, sync::watch::Receiver};
use tokio_stream::wrappers::WatchStream;
use tokio_tungstenite::tungstenite;
use tonic::{async_trait, Response};

type TonicServiceResult<T> = std::result::Result<tonic::Response<T>, tonic::Status>;

pub struct SatoriAgent {
    pub client: SatoriClient,
    pub events: Receiver<EventStructure>,
}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SatoriConfig {
    /// should be "http" or "https"
    pub scheme: String,
    /// host of the satori server.
    pub host: String,
    /// port of the satori server.
    pub port: u16,
    /// a optional path as [introduced by satori](https://satori.js.org/zh-CN/protocol/api.html#http-api)
    ///
    /// The version is **not** included, use the `version` field.
    pub path: Option<String>,
    /// satori protocol version.
    /// e.g. "v1"
    pub version: String,
    pub token: Option<String>,
}
impl SatoriAgent {
    pub fn new(base_url: reqwest::Url, token: Option<String>) -> Self {
        let (tx, rx) = tokio::sync::watch::channel(Default::default());
        let base_url_clone = base_url.clone();
        let token_clone = token.clone();
        tokio::spawn(async move {
            let mut base_url = base_url_clone;
            let token = token_clone;
            base_url.set_scheme("ws").unwrap();
            let ws_endpoint = base_url.join("events").unwrap();
            loop {
                log::info!(
                    "try connecting satori event websocket on {}...",
                    ws_endpoint
                );
                let conn = tokio_tungstenite::connect_async(&ws_endpoint).await;
                if let Ok((mut ws, _)) = conn {
                    log::info!("Connected to satori server for event websocket");
                    if let Some(token) = &token {
                        let _ = ws
                            .send(tungstenite::Message::text(format!(
                                r#"{{"op":3,"body":{{"token":"{}"}}}}"#,
                                token
                            )))
                            .await;
                    }
                    let mut interval = tokio::time::interval(Duration::from_secs(10));
                    loop {
                        select! {
                            _ = interval.tick() => {
                               let r = ws.send(tungstenite::Message::text(r#"{"op":1}"#)).await;
                               if r.is_err() {
                                   log::error!("Failed to send ping to satori server for event websocket/connection closed?...reconnecting...");
                                   break;
                               }
                            },
                            next = ws.next() => {
                                if let Some(Ok(msg)) = next {
                                    log::debug!("Received message from satori server for event websocket: {:?}", msg);
                                    if let tungstenite::Message::Text(s) = msg{
                                        let mut data:serde_json::Value = serde_json::from_str(s.as_str()).unwrap();
                                        if !data.get("op").is_some_and(|x|x==&json!(0)) {
                                            continue;
                                        }
                                        if let Some(Ok(ev)) =data.get_mut("body")
                                            .map(|x|serde_json::from_value::<schema::Event>(x.take())
                                            .map_err(|_|())
                                            .and_then(TryInto::try_into)) {
                                            log::debug!("received message event:{:?}",ev);
                                            let _ = tx.send(ev);
                                        }
                                    }
                                    //    if let Ok(ev) = serde_json::from_str(msg.to_text().unwrap()) {
                                    //        let _ = tx.send(ev);
                                    //    }
                                } else {
                                    log::error!("Failed to receive message from satori server for event websocket/connection closed?...reconnecting...");
                                    break;
                                }
                            }
                        }
                    }
                } else {
                    log::error!("Failed to connect to satori server for event websocket:{}...retrying in 5 seconds",conn.unwrap_err());
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        });
        Self {
            client: SatoriClient {
                client: reqwest::Client::new(),
                base_url,
                token,
            },
            events: rx,
        }
    }
    pub fn try_from_opts(opts: SatoriConfig) -> Result<Self, url::ParseError> {
        let base_url = reqwest::Url::parse(&format!(
            "{}://{}:{}/{}{}/",
            opts.scheme,
            opts.host,
            opts.port,
            opts.path.unwrap_or_default(),
            opts.version
        ))?;
        Ok(Self::new(base_url, opts.token))
    }
}

impl AuthenticationService for SatoriAgent {}
#[async_trait]
impl CoreService for SatoriAgent {
    async fn get_version(
        &self,
        _request: tonic::Request<GetVersionRequest>,
    ) -> TonicServiceResult<GetVersionResponse> {
        Ok(Response::new(GetVersionResponse {
            app_name: "kritor_agent:Satori".into(),
            version: env!("CARGO_PKG_VERSION").into(),
        }))
    }
    async fn get_current_account(
        &self,
        _request: tonic::Request<GetCurrentAccountRequest>,
    ) -> TonicServiceResult<GetCurrentAccountResponse> {
        self.client
            .login_get()
            .await
            .map(|login| {
                Response::new({
                    if let Some(user) = login.user {
                        GetCurrentAccountResponse {
                            account_name: user.name.unwrap_or_default(),
                            account_uin: user.id.parse().unwrap_or_default(),
                            account_uid: user.id,
                        }
                    } else {
                        let id = login.self_id.unwrap_or_default();
                        GetCurrentAccountResponse {
                            account_name: "".into(),
                            account_uin: id.parse().unwrap_or_default(),
                            account_uid: id,
                        }
                    }
                })
            })
            .map_err(Into::into)
    }
}
impl DeveloperService for SatoriAgent {}
#[async_trait]
impl EventService for SatoriAgent {
    async fn register_active_listener(
        &self,
        _request: tonic::Request<RequestPushEvent>,
    ) -> TonicServiceResult<tonic::codegen::BoxStream<EventStructure>> {
        Ok(Response::new(Box::pin(
            WatchStream::from_changes(self.events.clone()).map(Ok),
        )))
    }
}

impl GroupFileService for SatoriAgent {}
impl FriendService for SatoriAgent {}
impl GroupService for SatoriAgent {}
impl GuildService for SatoriAgent {}
#[async_trait]
impl MessageService for SatoriAgent {
    async fn send_message(
        &self,
        request: tonic::Request<SendMessageRequest>,
    ) -> std::result::Result<tonic::Response<SendMessageResponse>, tonic::Status> {
        let mut request = request.into_inner();
        let contact = request
            .contact
            .take()
            .ok_or(tonic::Status::invalid_argument("contact"))?;
        let channel_id = match contact.scene.try_into().ok() {
            Some(Scene::Group) => contact.peer,
            Some(Scene::Friend) => format!("private:{}", contact.peer),
            _ => {
                return Err(tonic::Status::invalid_argument(
                    "The scene is not supported by satori",
                ))
            }
        };
        let content = message::element::Root::try_from_kritor_elements(request.elements)
            .map(|x| x.root_element.serialize())
            .map_err(|e| {
                tonic::Status::invalid_argument(format!(
                    "Failed to convert kritor elements to satori elements:{}",
                    e
                ))
            })?;
        let mut resp = self
            .client
            .message_create(channel_id, content)
            .await
            .map_err(|e| tonic::Status::internal(format!("satori returned a error for {}", e)))?;
        let last = resp.pop().ok_or(tonic::Status::internal(
            "satori returned a empty message list",
        ))?;
        Ok(Response::new(SendMessageResponse {
            message_id: last.id,
            message_time: last.created_at.unwrap_or_default() as _,
        }))
    }
}

impl ProcessService for SatoriAgent {}
impl ReverseService for SatoriAgent {}
impl WebService for SatoriAgent {}

impl From<client::Error> for tonic::Status {
    fn from(value: client::Error) -> Self {
        match value {
            client::Error::Reqwest(e) => tonic::Status::internal(format!("Request failed: {}", e)),
            client::Error::BadRequest => tonic::Status::invalid_argument("请求格式错误"),
            client::Error::Unauthorized => tonic::Status::unauthenticated("缺失鉴权"),
            client::Error::Forbidden => tonic::Status::permission_denied("权限不足"),
            client::Error::NotFound => tonic::Status::not_found("资源不存在"),
            client::Error::MethodNotAllowed => tonic::Status::unimplemented("请求方法不支持"),
            client::Error::ServerSide => tonic::Status::internal("服务器错误"),
            client::Error::NonStandardResponseStatus => {
                tonic::Status::internal("Non Standard Response Status")
            }
        }
    }
}
