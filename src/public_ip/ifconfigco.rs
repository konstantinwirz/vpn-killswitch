use std::net::Ipv4Addr;
use std::str::FromStr;
use serde::Deserialize;

use crate::public_ip::{provider::PublicIpLookupProvider, public_ip::PublicIpLookupResponse};
use crate::public_ip::error::PublicIpLookupError;

/// Represents ifconfig.co
///
/// rate limit: 1 req/minute
///
/// test it:
/// ```shell
/// curl -H "Accept: application/json" -H "User-Agent: vpn-killswitch/0.1.0" https://ifconfig.co/
/// ```
#[derive(Debug)]
pub struct IfConfigCo {
    url: &'static str,
}

const URL: &str = "https://ifconfig.co/";

impl IfConfigCo {
    pub fn new() -> Self {
        Self { url: URL }
    }
}

/// Represents the response we get from ifconfig.co
///
/// Example:
/// ```json
/// {
///   "ip": "93.254.94.16",
///   "ip_decimal": 1576951312,
///   "country": "Germany",
///   "country_iso": "DE",
///   "country_eu": true,
///   "region_name": "Bavaria",
///   "region_code": "BY",
///   "zip_code": "97074",
///   "city": "Würzburg",
///   "latitude": 49.7786,
///   "longitude": 9.9308,
///   "time_zone": "Europe/Berlin",
///   "asn": "AS3320",
///   "asn_org": "Deutsche Telekom AG",
///   "hostname": "p5dfe5e10.dip0.t-ipconnect.de",
///   "user_agent": {
///     "product": "vpn-killswitch",
///     "version": "0.1.0",
///     "raw_value": "vpn-killswitch/0.1.0"
///   }
/// }
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct IfConfigCoResponse {
    ip: String,
    asn: String,
}

impl PublicIpLookupProvider for IfConfigCo {
    type ResponseType = IfConfigCoResponse;

    fn url(&self) -> &'static str {
        self.url
    }
}

impl TryFrom<IfConfigCoResponse> for PublicIpLookupResponse {
    type Error = PublicIpLookupError;

    fn try_from(value: IfConfigCoResponse) -> Result<Self, Self::Error> {
        Ok(
            Self {
                ip: Ipv4Addr::from_str(&value.ip)?,
                asn: Some(remove_asn_prefix(value.asn)),
            }
        )
    }
}

fn remove_asn_prefix(asn: String) -> String {
    if asn.starts_with("AS") {
        return asn[2..].to_string();
    }

    asn
}
