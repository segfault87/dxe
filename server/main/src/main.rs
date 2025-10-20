mod config;
mod controller;
mod middleware;

use clap::Parser;

use crate::config::Config;

#[derive(clap::Parser, Debug)]
struct Args {
    #[arg(short)]
    config_path: std::path::PathBuf,
    #[arg(default_value = "127.0.0.1:8000")]
    address: std::net::SocketAddr,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::parse();
    let config = toml::from_str::<Config>(&std::fs::read_to_string(&args.config_path)?)?;

    let _ = actix_web::HttpServer::new(|| actix_web::App::new().service(controller::api()))
        .bind(&args.address)?
        .run()
        .await?;

    Ok(())
}
