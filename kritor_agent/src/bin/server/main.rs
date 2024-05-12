use std::{io, sync::Arc};

use clap::{Arg, Command};
use kritor::auth::authentication_service_server::AuthenticationServiceServer;
use kritor::core::core_service_server::CoreServiceServer;
use kritor::developer::developer_service_server::DeveloperServiceServer;
use kritor::event::event_service_server::EventServiceServer;
use kritor::file::group_file_service_server::GroupFileServiceServer;
use kritor::friend::friend_service_server::FriendServiceServer;
use kritor::group::group_service_server::GroupServiceServer;
use kritor::guild::guild_service_server::GuildServiceServer;
use kritor::message::message_service_server::MessageServiceServer;
use kritor::process::process_service_server::ProcessServiceServer;
use kritor::reverse::reverse_service_server::ReverseServiceServer;
use kritor::web::web_service_server::WebServiceServer;
use kritor_agent::agents::{satori, SatoriAgent};
use tonic::transport::Server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let matches = cmd().get_matches();
    let config_path: &String = matches.get_one("config").unwrap();
    println!("config_path: {}", config_path);
    let config = Config::from_file(config_path);

    if let Err(e) = &config {
        if let Some(e) = e.downcast_ref::<io::Error>() {
            if e.kind() == io::ErrorKind::NotFound {
                println!("Config file not found. Generate an example? (y/n)");
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                if input.trim().to_lowercase() == "y" {
                    let example_str = toml::to_string_pretty(&Config::example())?;
                    std::fs::write(config_path, example_str)?;
                    println!("🚀 Example config file generated at {}", config_path);
                    println!("✨✨ Please modify it to fit your needs and run the program again.");
                    return Ok(());
                }
            }
        }
    }
    let config = config.unwrap();

    if let Some(rust_log) = &config.server.rust_log {
        std::env::set_var("RUST_LOG", rust_log);
    }
    env_logger::init();

    let agent = match config.backend {
        BackendConfig::Satori(opts) => SatoriAgent::try_from_opts(opts)?,
    };
    let agent = Arc::new(agent);

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(kritor::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    Server::builder()
        .add_service(AuthenticationServiceServer::from_arc(agent.clone()))
        .add_service(CoreServiceServer::from_arc(agent.clone()))
        .add_service(DeveloperServiceServer::from_arc(agent.clone()))
        .add_service(EventServiceServer::from_arc(agent.clone()))
        .add_service(GroupFileServiceServer::from_arc(agent.clone()))
        .add_service(FriendServiceServer::from_arc(agent.clone()))
        .add_service(GroupServiceServer::from_arc(agent.clone()))
        .add_service(GuildServiceServer::from_arc(agent.clone()))
        .add_service(MessageServiceServer::from_arc(agent.clone()))
        .add_service(ProcessServiceServer::from_arc(agent.clone()))
        .add_service(ReverseServiceServer::from_arc(agent.clone()))
        .add_service(WebServiceServer::from_arc(agent.clone()))
        .add_service(reflection)
        .serve(config.server.listen.parse()?)
        .await?;
    Ok(())
}

fn cmd() -> Command {
    Command::new(env!("CARGO_CRATE_NAME")).args([Arg::new("config")
        .short('c')
        .default_value("kritor_agent.toml")
        .help("Path to the configuration file")])
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Config {
    server: ServerConfig,
    backend: BackendConfig,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ServerConfig {
    listen: String,
    rust_log: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
enum BackendConfig {
    #[serde(rename = "satori")]
    Satori(satori::SatoriConfig),
}

impl Config {
    fn from_file(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }
    fn example() -> Self {
        Self {
            server: ServerConfig {
                listen: "127.0.0.1:51405".into(),
                rust_log: Some("info".into()),
            },
            backend: BackendConfig::Satori(satori::SatoriConfig {
                scheme: "http".into(),
                host: "127.0.0.1".into(),
                port: 15500,
                path: None,
                token: Some("super_secret".into()),
                version: "v1".into(),
            }),
        }
    }
}
