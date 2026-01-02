use std::fmt::Display;
use serde::{Serialize, Deserialize};
use log::{trace, debug, info};

#[derive(Debug, Clone)]
pub (crate) struct IpInfo {
    pub ip: String,
    pub as_number: String,
    pub as_name: String,
    pub as_domain: String,
}

impl Display for IpInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IpInfo {{ ip: {}, as_number: {}, as_name: {},  as_domain: {} }}", self.ip, self.as_number, self.as_name, self.as_domain)
    }
}

pub (crate) struct IpInfoFetcher<'a> {
    http_client: reqwest::Client,
    api_token: &'a str,
}

/// Response from the IpInof API
#[derive(Serialize, Deserialize, Debug, Clone)]
struct IPInfoResponse {
    ip: String,
    asn: String,
    as_name: String,
    as_domain: String,
    country_code: String,
    country: String,
    continent_code: String,
    continent: String,
}

impl From<IPInfoResponse> for IpInfo {

    fn from(response: IPInfoResponse) -> Self {
        Self {
            ip: response.ip,
            as_number: response.asn,
            as_name: response.as_name,
            as_domain: response.as_domain,
        }
    }
}

const API_URL: &str = "https://api.ipinfo.io/lite/me";
const TOKEN_QUERY_PARAM: &str = "token";

impl <'a> IpInfoFetcher<'a> {

    pub (crate) fn new(api_token: &'a str) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            api_token,
        }
    }

    pub (crate) async fn fetch_ip_info(&self) -> Result<IpInfo, anyhow::Error> {
        trace!("about to fetch ip info from {API_URL}");

        let response = self.http_client.get(API_URL)
        .query(&[(TOKEN_QUERY_PARAM, self.api_token)])
        .send()
        .await?;

        debug!("called ip info API, got http status code: {}", response.status().as_u16());

        if !response.status().is_success() {
            anyhow::bail!("IP Info API didn't respond with 2xx")
        }

        info!("got a successful response from {}; http status code: {}", API_URL, response.status().as_u16());

        let payload: IPInfoResponse = response.json().await?;
        debug!("payload = {:?}", payload);

        Ok(payload.into())
    }
}
