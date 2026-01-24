use std::net::Ipv4Addr;
use std::str::FromStr;
use serde::Deserialize;
use crate::public_ip::error::PublicIpLookupError;
use crate::public_ip::provider::PublicIpLookupProvider;
use crate::public_ip::public_ip::PublicIpLookupResponse;

/// Represents ip-api.com (TLS not supported in this free API)
///
/// Rate limit: 45 req/minute
///
/// Test it:
/// ```shell
/// curl -H "Accept: application/json" http://ip-api.com/json?fields=query,as
/// ```
#[derive(Debug, Clone)]
pub struct IpApiCom {
    url: &'static str,
}

const URL: &str = "http://ip-api.com/json?fields=query,as";

impl IpApiCom {
    pub fn new() -> Self {
        Self { url: URL }
    }
}

/// Represents the response from the ip-api.com
///
/// Example:
/// ```json
/// {
///   "as": "AS3320 Deutsche Telekom AG",
///   "query": "93.254.94.16"
/// }
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct IpApiComResponse {
    query: String,
    r#as: String
}

impl PublicIpLookupProvider for IpApiCom {
    type ResponseType = IpApiComResponse;

    fn url(&self) -> &'static str {
        self.url
    }
}

impl TryFrom<IpApiComResponse> for PublicIpLookupResponse {
    type Error = PublicIpLookupError;

    fn try_from(value: IpApiComResponse) -> Result<Self, Self::Error> {
        Ok(
            Self {
                ip: Ipv4Addr::from_str(value.query.as_str())?,
                asn: parse_asn(value.r#as)
            }
        )
    }
}

fn parse_asn(asn: String) -> Option<String> {
    let mut split_iter = asn.split(' ');
    if let Some(asn_and_org) = split_iter.next() && asn_and_org.starts_with("AS") {
        return Some(asn_and_org[2..].to_string());
    }

    None
}
