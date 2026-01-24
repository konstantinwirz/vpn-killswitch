use std::net::Ipv4Addr;
use std::str::FromStr;
use serde::Deserialize;
use crate::public_ip::error::PublicIpLookupError;
use crate::public_ip::provider::PublicIpLookupProvider;
use crate::public_ip::public_ip::PublicIpLookupResponse;

/// Represents ipwho.is
///
/// Rate Limit: 10_000 req/month
///
/// Test it:
/// ```shell
/// curl -H "Accept: application/json" https://ipwho.is
/// ```
#[derive(Clone, Debug)]
pub struct IpWhoIs {
    url: &'static str
}

const URL: &str = "https://ipwho.is?fields=ip,connection.asn";

impl IpWhoIs {

    pub fn new() -> Self {
        Self { url: URL }
    }
}

/// Represents a response from ipwho.is
///
/// We are only interested in ip and asn.
///
/// Example:
/// ```json
/// {
///   "ip": "93.254.94.16",
///   "success": true,
///   "type": "IPv4",
///   "continent": "Europe",
///   "continent_code": "EU",
///   "country": "Germany",
///   "country_code": "DE",
///   "region": "Hesse",
///   "region_code": "HE",
///   "city": "Frankfurt",
///   "latitude": 50.1109221,
///   "longitude": 8.6821267,
///   "is_eu": true,
///   "postal": "60311",
///   "calling_code": "49",
///   "capital": "Berlin",
///   "borders": "AT,BE,CH,CZ,DK,FR,LU,NL,PL",
///   "flag": {
///     "img": "https://cdn.ipwhois.io/flags/de.svg",
///     "emoji": "🇩🇪",
///     "emoji_unicode": "U+1F1E9 U+1F1EA"
///   },
///   "connection": {
///     "asn": 3320,
///     "org": "Deutsche Telekom AG",
///     "isp": "Deutsche Telekom AG",
///     "domain": "telekom.de"
///   },
///   "timezone": {
///     "id": "Europe/Berlin",
///     "abbr": "CET",
///     "is_dst": false,
///     "offset": 3600,
///     "utc": "+01:00",
///     "current_time": "2026-01-12T17:25:16+01:00"
///   }
/// }
/// ```
#[derive(Clone, Debug, Deserialize)]
pub struct IpWhoIsResponse {
    ip: String,
    connection: IpWhoIsConnectionResponse,
}

#[derive(Clone, Debug, Deserialize)]
struct IpWhoIsConnectionResponse {
    asn: u64,
}

impl PublicIpLookupProvider for IpWhoIs {
    type ResponseType = IpWhoIsResponse;

    #[inline]
    fn url(&self) -> &'static str {
        self.url
    }
}

impl TryFrom<IpWhoIsResponse> for PublicIpLookupResponse {
    type Error = PublicIpLookupError;

    fn try_from(value: IpWhoIsResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            ip: Ipv4Addr::from_str(value.ip.as_str())?,
            asn: Some(value.connection.asn.to_string()),
        })
    }
}