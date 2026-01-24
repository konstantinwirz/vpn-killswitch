use reqwest::header::{ACCEPT, HeaderMap, USER_AGENT};
use serde::de::DeserializeOwned;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub trait PublicIpLookupProvider {
    type ResponseType: DeserializeOwned;

    fn url(&self) -> &'static str;
    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "application/json".parse().expect("must work"));
        headers.insert(
            USER_AGENT,
            format!("{}/{}", APP_NAME, APP_VERSION)
                .parse()
                .expect("must work"),
        );
        headers
    }
}
