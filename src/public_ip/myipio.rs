use std::net::Ipv4Addr;
use std::str::FromStr;
use serde::Deserialize;
use crate::public_ip::error::PublicIpLookupError;
use crate::public_ip::provider::PublicIpLookupProvider;
use crate::public_ip::public_ip::PublicIpLookupResponse;

/// Represents my-ip.io
///
/// Rate limit: unknown
///
/// Test it:
///
/// ```shell
/// curl -H "Accept: application/json" -H "User-Agent: vpn-killswitch/0.1.0" https://api.my-ip.io/v2/ip.json
/// ```
#[derive(Debug, Clone)]
pub struct MyIpIo {
    url: &'static str,
}

const URL: &str = "https://api.my-ip.io/v2/ip.json";

impl MyIpIo {
    pub fn new() -> Self {
        MyIpIo { url: URL }
    }
}

/// Represents the response we get from my-ip.io
///
/// Example:
/// ```json
/// {
///   "success": true,
///   "ip": "93.254.94.16",
///   "type": "IPv4",
///   "country": {
///     "code": "DE",
///     "name": "Germany"
///   },
///   "region": "Hesse",
///   "city": "Frankfurt am Main",
///   "location": {
///     "lat": 50.1367,
///     "lon": 8.7138
///   },
///   "timeZone": "Europe/Berlin",
///   "asn": {
///     "number": 3320,
///     "name": "Deutsche Telekom AG",
///     "network": "93.192.0.0/10"
///   }
/// }
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct MyIpIoResponse {
    ip: String,
    asn: MyIpIoResponseAsn

}

#[derive(Debug, Clone, Deserialize)]
struct MyIpIoResponseAsn {
    number: u64
}

impl PublicIpLookupProvider for MyIpIo {
    type ResponseType = MyIpIoResponse;

    fn url(&self) -> &'static str {
        self.url
    }
}

impl TryFrom<MyIpIoResponse> for PublicIpLookupResponse {
    type Error = PublicIpLookupError;

    fn try_from(value: MyIpIoResponse) -> Result<Self, Self::Error> {
        Ok(
            Self{
                ip: Ipv4Addr::from_str(value.ip.as_str())?,
                asn: Some(value.asn.number.to_string()),
            }
        )
    }
}
