use axum::extract::State;
use axum::routing::get;
use clap::Parser;
use log::{error, info};
use reqwest::StatusCode;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::{mpsc, Mutex};

mod ipinfo;
use crate::ipinfo::{IpInfo, IpInfoFetcher};

struct KillSwitchState<F>
where
    F: Fn(&IpInfo) -> bool,
{
    rx: Receiver<IpInfo>, // receives ip info from the ipinfo fetcher
    rule: F,              // the kill switch rule
}

impl<F> KillSwitchState<F>
where
    F: Fn(&IpInfo) -> bool,
{
    fn new(rx: Receiver<IpInfo>, rule: F) -> Self {
        Self { rx, rule }
    }
}

#[derive(Parser)]
struct Cli {
    /// API token for the ipinfo.io API
    #[arg(short = 't', long, value_name = "API-TOKEN", env = "IPINFO_API_TOKEN")]
    api_token: String,
    /// ASN of the VPN provider
    #[arg(short, long, value_name = "ASN", env = "VPN_PROVIDER_ASN")]
    asn: String,
    /// Log level (Error, Warn, Info, Debug, Trace)
    #[arg(short, long, default_value = "Info", value_name = "LOG-LEVEL", env = "VPN_KILLSWITCH_LOG_LEVEL")]
    log_level: log::Level,
    /// HTTP port
    #[arg(short = 'p', long, default_value = "8080", value_name = "HTTP-PORT", env = "VPN_KILLSWITCH_HTTP_PORT")]
    http_port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // parse command line arguments
    let cli = Cli::parse();
    // initialize the logger
    env_logger::Builder::new()
        .filter_level(cli.log_level.to_level_filter())
        .init();
    // poll the api to get ip info
    let (tx, rx) = mpsc::channel::<IpInfo>(1);
    let ip_info_fetch_task = tokio::spawn(async move {
        let client = IpInfoFetcher::new(cli.api_token.as_str());
        loop {
            match client.fetch_ip_info().await {
                Ok(ip_info) => {
                    if let Err(err) = tx.send(ip_info).await {
                        error!("Error sending ip info to a channel: {}, aborting...", err);
                        break;
                    }
                }
                Err(_) => {
                    error!("Error fetching ip info from the API, aborting...");
                    break;
                }
            };
        }
    });

    let state = Arc::new(Mutex::new(KillSwitchState::new(rx, move |ip_info| {
        ip_info.as_number.eq_ignore_ascii_case(cli.asn.as_str())
    })));

    let app = axum::Router::new()
        .route("/killswitch", get(handle_killswitch))
        .route("/health", get(handle_healthcheck))
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", cli.http_port)).await?;

    tokio::select! {
        _ = axum::serve(listener, app) => {
            info!("http server finished");
        },
        _ = ip_info_fetch_task => {
            info!("ip info fetch task finished");
        },
    }

    Ok(())
}

async fn handle_killswitch<F: Fn(&IpInfo) -> bool>(
    State(state): State<Arc<Mutex<KillSwitchState<F>>>>,
) -> (StatusCode, &'static str) {
    let mut state = state.as_ref().lock().await;
    match state.rx.recv().await {
        Some(ref ip_info) => {
            if (state.rule)(ip_info) {
                (StatusCode::OK, "VPN on")
            } else {
                (StatusCode::SERVICE_UNAVAILABLE, "VPN off")
            }
        }
        None => {
            error!("Error receiving ip info from the channel");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "couldn't determine the vpn state",
            )
        }
    }
}

async fn handle_healthcheck() -> (StatusCode, &'static str) {
    (StatusCode::OK, "OK")
}
