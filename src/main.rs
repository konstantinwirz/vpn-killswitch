mod asn_resolver;
mod public_ip;

use std::{
    io::{Write, stdout},
    net::Ipv4Addr,
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use asn_resolver::AsnResolver;
use axum::{Router, extract::State, http::StatusCode, routing::get};
use log::{debug, error, info, warn};
use public_ip::PublicIpResolver;
use tokio::{sync::RwLock, time::sleep};

const DEFAULT_HTTP_PORT: u16 = 8080;
const DEFAULT_POLL_INTERVAL_SECONDS: u64 = 15;
const DEFAULT_LOG_LEVEL: log::Level = log::Level::Info;

#[derive(Debug, Clone)]
struct AppState {
    current_ip: Option<Ipv4Addr>,
    current_asn: Option<String>,
    expected_asn: String,
}

impl AppState {
    fn with_expected_asn(asn: String) -> Self {
        Self {
            current_ip: None,
            current_asn: None,
            expected_asn: asn,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse()?;

    env_logger::Builder::new()
        .filter_level(args.log_level.to_level_filter())
        .init();

    if args.help {
        let mut out = stdout().lock();
        out.write_all(args.usage().as_bytes())?;
        return Ok(());
    }

    let state = Arc::new(RwLock::new(AppState::with_expected_asn(args.asn)));

    let app = Router::new()
        .route("/", get(health))
        .with_state(Arc::clone(&state));
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", args.port)).await?;

    tokio::select! {
        _ = axum::serve(listener, app) => {

        }
        _ = resolve_asn(args.poll_interval, Arc::clone(&state)) => {

        }
    }

    Ok(())
}

async fn health(State(state): State<Arc<RwLock<AppState>>>) -> StatusCode {
    let state = state.read().await;

    match state.current_asn {
        Some(ref current_asn) => {
            debug!(
                "current asn={}, expecting {}",
                current_asn, state.expected_asn
            );
            if *current_asn == state.expected_asn {
                info!(
                    "current asn ({}) = expected asn ({})",
                    current_asn, state.expected_asn
                );
                StatusCode::OK
            } else {
                warn!(
                    "current asn ({}) != expected asn ({})",
                    current_asn, state.expected_asn
                );
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
        _ => {
            warn!("no asn available");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

async fn resolve_asn(poll_interval: u64, state: Arc<RwLock<AppState>>) -> ! {
    let public_ip_resolver = PublicIpResolver::new();
    let asn_resolver = AsnResolver::default();

    const ATTEMPTS: u8 = 5;
    loop {
        // get public ip
        let mut public_ip = None;
        for attempt in 0..ATTEMPTS {
            debug!(
                "about to get public ip, attempt = {} of {}",
                attempt + 1,
                ATTEMPTS
            );

            match public_ip_resolver.lookup_with_random_provider().await {
                Ok(ip) => {
                    public_ip = Some(ip);
                    break;
                }
                Err(_) => continue,
            }
        }

        let mut state = state.write().await;

        if let Some(ip) = public_ip {
            if state.current_ip != Some(ip) {
                match asn_resolver.resolve(&ip).await {
                    Ok(asn) => {
                        _ = state.current_ip.insert(ip);
                        _ = state.current_asn.insert(asn);
                    }
                    _ => continue,
                }
            }
        } else {
            error!("failed to get public ip");
            _ = state.current_ip.take();
            _ = state.current_asn.take();
        }

        drop(state); // drop it before going to sleep, otherwise it is being locked for too long

        sleep(Duration::from_secs(poll_interval)).await;
    }
}

#[derive(Debug, Clone)]
struct Args {
    app_name: String,
    help: bool,
    port: u16,
    poll_interval: u64,
    asn: String,
    log_level: log::Level,
}

impl Args {
    fn usage(&self) -> String {
        format!(
            "\
            Usage: {} [OPTIONS]\n\
            \n\
            Options:\n\
            \n\
            \t--help/-h\t\t\tPrints this message and exits\n\
            \t--port/-p\t\t\tHttp port (DEFAULT: {})\n\
            \t--poll-interval/-i\t\tPoll interval in seconds (DEFAULT: {})\n\
            \t--log-level/-l\t\t\tLog level (DEFAULT: {})\n\
            \t--asn/-a\t\t\tASN of the VPN provider\n\
            ",
            self.app_name, DEFAULT_HTTP_PORT, DEFAULT_POLL_INTERVAL_SECONDS, DEFAULT_LOG_LEVEL
        )
    }

    fn parse() -> anyhow::Result<Args> {
        let mut args = std::env::args();
        let app_name: String = args.next().expect("app name is first arg");
        let mut help = false;
        let mut port: u16 = DEFAULT_HTTP_PORT;
        let mut poll_interval = DEFAULT_POLL_INTERVAL_SECONDS;
        let mut log_level = DEFAULT_LOG_LEVEL;
        let mut asn: Option<String> = None;

        loop {
            match args.next().as_deref() {
                Some("--help") | Some("-h") => {
                    help = true;
                    // no need to parse further
                    break;
                }
                Some("--port") | Some("-p") => {
                    // get port number
                    port = Self::next_arg(&mut args, "port")?;
                }
                Some("--poll-intervall") | Some("-i") => {
                    poll_interval = Self::next_arg(&mut args, "poll interval")?;
                }
                Some("--log-level") | Some("-l") => {
                    log_level = Self::next_arg(&mut args, "log level")?;
                }
                Some("--asn") | Some("-a") => {
                    asn = Some(Self::next_arg(&mut args, "asn")?);
                }
                Some(_) => {}  // unknown arg, ignoring it
                None => break, // no more args
            }
        }

        if !help && asn.is_none() {
            anyhow::bail!("ASN is required");
        }

        Ok(Args {
            app_name,
            help,
            port,
            poll_interval,
            asn: asn.unwrap_or(String::from("")),
            log_level,
        })
    }

    fn next_arg<I, R, E>(iter: &mut I, arg_name: &str) -> anyhow::Result<R>
    where
        I: Iterator<Item = String>,
        R: FromStr<Err = E>,
    {
        match iter.next() {
            Some(ref value) => value.parse::<R>().map_err(|_| {
                anyhow::anyhow!("error parsing {} - '{}'", arg_name, value.to_owned())
            }),
            None => anyhow::bail!("no value for {}", arg_name),
        }
    }
}
