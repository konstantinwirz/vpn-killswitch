use std::net::Ipv4Addr;
use std::str::FromStr;
use serde::Deserialize;
use crate::public_ip::error::PublicIpLookupError;
use crate::public_ip::provider::PublicIpLookupProvider;
use crate::public_ip::public_ip::PublicIpLookupResponse;

/// Represents ipapi.co
///
/// Rate limit: 30000 req/month (1000 req/day)
///
/// Test it:
/// ```shell
/// curl -H "Accept: application/json" https://ipapi.co/json
/// ```
pub struct IpApiCo {
    url: &'static str
}

const URL: &str = "https://ipapi.co/json";

impl IpApiCo {
    pub fn new() -> Self {
        Self { url: URL }
    }
}

/// Represents the response from ipapi.co
///
/// Example:
/// ```json
/// {
///     "ip": "93.254.94.16",
///     "network": "93.254.80.0/20",
///     "version": "IPv4",
///     "city": "Frankfurt am Main",
///     "region": "Hesse",
///     "region_code": "HE",
///     "country": "DE",
///     "country_name": "Germany",
///     "country_code": "DE",
///     "country_code_iso3": "DEU",
///     "country_capital": "Berlin",
///     "country_tld": ".de",
///     "continent_code": "EU",
///     "in_eu": true,
///     "postal": "60389",
///     "latitude": 50.1367,
///     "longitude": 8.7138,
///     "timezone": "Europe/Berlin",
///     "utc_offset": "+0100",
///     "country_calling_code": "+49",
///     "currency": "EUR",
///     "currency_name": "Euro",
///     "languages": "de",
///     "country_area": 357021.0,
///     "country_population": 82927922,
///     "asn": "AS3320",
///     "org": "Deutsche Telekom AG"
/// }
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct IpApiCoResponse {
    ip: String,
    asn: String,
}

impl PublicIpLookupProvider for IpApiCo {
    type ResponseType = IpApiCoResponse;

    #[inline]
    fn url(&self) -> &'static str {
        self.url
    }
}

impl TryFrom<IpApiCoResponse> for PublicIpLookupResponse {
    type Error = PublicIpLookupError;

    fn try_from(value: IpApiCoResponse) -> Result<Self, Self::Error> {
        Ok(
            Self {
                ip: Ipv4Addr::from_str(value.ip.as_str())?,
                asn: Some(remove_as_prefix(value.asn))

            }
        )
    }
}

fn remove_as_prefix(asn: String) -> String {
    if asn.starts_with("AS") {
        return asn[2..].to_string();
    }

    asn
}