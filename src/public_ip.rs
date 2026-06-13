use std::str::FromStr;
use std::{collections::HashMap, net::Ipv4Addr};

use anyhow::Context;
use log::debug;
use rand::seq::IndexedRandom;
use reqwest::header::{ACCEPT, HeaderMap, HeaderName, USER_AGENT};
use serde_json::Value;

pub trait PublicIpProvider {
    fn url(&self) -> &str;

    fn deserialize_payload(&self, payload: &Value) -> anyhow::Result<Ipv4Addr> {
        match payload[self.ip_field()] {
            Value::String(ref ip) => ip.parse::<Ipv4Addr>().context("parsing Ipv4Addr"),
            _ => anyhow::bail!("response payload doesn't contain a string ip field"),
        }
    }

    fn additional_headers(&self) -> HashMap<String, String> {
        HashMap::default()
    }

    /// Returns the name of the field containing ip
    /// override it, if the fields` name differs
    #[inline]
    fn ip_field(&self) -> &'static str {
        "ip"
    }
}

const USER_AGENT_VAL: &str = concat!(env!("CARGO_BIN_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub struct PublicIpResolver {
    http_client: reqwest::Client,
    providers: Vec<Box<dyn PublicIpProvider>>,
}

impl PublicIpResolver {
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::new(),
            providers: vec![
                Box::new(IfConfigCo::default()),
                Box::new(IpifyOrg::default()),
                Box::new(IpInfoIo::default()),
                Box::new(IpApiCom::default()),
                Box::new(IpWhoIs::default()),
                Box::new(MyIpIo::default()),
            ],
        }
    }

    pub async fn lookup_with_random_provider(&self) -> anyhow::Result<Ipv4Addr> {
        match self.providers.choose(&mut rand::rng()) {
            Some(p) => self.lookup_with_provider(p.as_ref()).await,
            None => anyhow::bail!("no providers given"),
        }
    }

    pub async fn lookup_with_provider<P: PublicIpProvider + ?Sized>(
        &self,
        provider: &P,
    ) -> anyhow::Result<Ipv4Addr> {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "application/json".parse()?);
        headers.insert(USER_AGENT, USER_AGENT_VAL.parse()?);
        for (key, value) in provider.additional_headers() {
            headers.insert(HeaderName::from_str(&key)?, value.parse()?);
        }

        debug!(
            "about to make a get request to '{}', with headers: {:?}",
            provider.url(),
            &headers
        );

        let response = self
            .http_client
            .get(provider.url())
            .headers(headers)
            .send()
            .await
            .context(format!("couldn't get response from {}", provider.url()))?;

        debug!(
            "got response from '{}', status code={}",
            provider.url(),
            response.status()
        );

        let data = response.bytes().await?;
        let json: Value =
            serde_json::from_slice(&data).context("create json value from response")?;
        provider.deserialize_payload(&json)
    }
}

///
/// ifConfigCo represents ifconfig.co.
///
/// rate limit: 1 req/minute
///
/// test it:
/// ```shell
/// curl -H "Accept: application/json" https://ifconfig.co/
/// ```
pub struct IfConfigCo {
    url: String,
}

impl Default for IfConfigCo {
    fn default() -> Self {
        Self {
            url: String::from("https://ifconfig.co/"),
        }
    }
}

#[cfg(test)]
impl IfConfigCo {
    fn with_url(url: &str) -> Self {
        return Self {
            url: String::from(url),
        };
    }
}

impl PublicIpProvider for IfConfigCo {
    fn url(&self) -> &str {
        &self.url
    }
}

///
/// IpifyOrg represents ipify.org
///
/// Rate limit: no limits
///
/// Test it:
/// ```shell
/// curl -H "Accept: application/json" https://api.ipify.org?format=json
/// ```
pub struct IpifyOrg {
    url: String,
}

impl Default for IpifyOrg {
    fn default() -> Self {
        Self {
            url: String::from("https://api.ipify.org?format=json"),
        }
    }
}

#[cfg(test)]
impl IpifyOrg {
    fn with_url(url: &str) -> Self {
        Self {
            url: String::from(url),
        }
    }
}

impl PublicIpProvider for IpifyOrg {
    fn url(&self) -> &str {
        &self.url
    }
}

///
/// IpInfoIo represents ipinfo.io
///
/// Rate limit: 1000 req/day
///
/// Test it:
/// ```shell
/// curl -H "Accept: application/json" https://ipinfo.io
/// ```
pub struct IpInfoIo {
    url: String,
}

impl PublicIpProvider for IpInfoIo {
    fn url(&self) -> &str {
        &self.url
    }
}

impl Default for IpInfoIo {
    fn default() -> Self {
        Self {
            url: String::from("https://ipinfo.io"),
        }
    }
}

#[cfg(test)]
impl IpInfoIo {
    fn with_url(url: &str) -> Self {
        Self {
            url: String::from(url),
        }
    }
}

///
/// Represents ip-api.com (TLS not supported in this free API)
///
/// Rate limit: 45 req/minute
///
/// Test it:
/// ```shell
/// curl -H "Accept: application/json" http://ip-api.com/json?fields=query
/// ```
pub struct IpApiCom {
    url: String,
}

impl Default for IpApiCom {
    fn default() -> Self {
        Self {
            url: String::from("http://ip-api.com/json?fields=query"),
        }
    }
}

#[cfg(test)]
impl IpApiCom {
    fn with_url(url: &str) -> Self {
        Self {
            url: String::from(url),
        }
    }
}

impl PublicIpProvider for IpApiCom {
    fn url(&self) -> &str {
        &self.url
    }

    fn ip_field(&self) -> &'static str {
        "query"
    }
}

/// Represents ipwho.is
///
/// Rate Limit: 10_000 req/month
///
/// Test it:
/// ```shell
/// curl -H "Accept: application/json" https://ipwho.is
/// ```
pub struct IpWhoIs {
    url: String,
}

impl PublicIpProvider for IpWhoIs {
    fn url(&self) -> &str {
        &self.url
    }
}

impl Default for IpWhoIs {
    fn default() -> Self {
        Self {
            url: String::from("https://ipwho.is"),
        }
    }
}

#[cfg(test)]
impl IpWhoIs {
    fn with_url(url: &str) -> Self {
        Self {
            url: String::from(url),
        }
    }
}

/// Represents my-ip.io
///
/// Rate limit: unknown
///
/// Test it:
/// ```shell
/// curl -H "Accept: application/json" https://api.my-ip.io/v2/ip.json
/// ```
pub struct MyIpIo {
    url: String,
}

impl Default for MyIpIo {
    fn default() -> Self {
        Self {
            url: String::from("https://api.my-ip.io/v2/ip.json"),
        }
    }
}

impl PublicIpProvider for MyIpIo {
    fn url(&self) -> &str {
        &self.url
    }
}

#[cfg(test)]
impl MyIpIo {
    fn with_url(url: &str) -> Self {
        Self {
            url: String::from(url),
        }
    }
}

// tests

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::{Method::GET, MockServer};

    #[macro_export]
    macro_rules! test_provder {
        ($fn_name:ident, $provider_type:ident, $payload:literal, $expected_ip:literal) => {
            #[tokio::test]
            async fn $fn_name() -> Result<(), Box<dyn std::error::Error>> {
                let mock_server = MockServer::start();
                let _ = mock_server.mock(|when, then| {
                    when.method(GET)
                        .path("/")
                        .header("Accept", "application/json");
                    then.status(200)
                        .header("content-type", "application/json")
                        .body($payload);
                });

                let resolver = PublicIpResolver::new();
                let base_url = mock_server.base_url();
                let provider = $provider_type::with_url(&base_url);
                let public_ip = resolver.lookup_with_provider(&provider).await?;
                let expected_ip = $expected_ip.parse::<Ipv4Addr>()?;

                assert_eq!(public_ip, expected_ip);

                Ok(())
            }
        };
    }

    test_provder!(
        test_ipinfoio_provider,
        IpInfoIo,
        r#"{
            "ip": "93.254.94.16",
            "hostname": "p5dfe5e10.dip0.t-ipconnect.de",
            "city": "Munich",
            "region": "Bavaria",
            "country": "DE",
            "loc": "48.1374,11.5755",
            "org": "AS3320 Deutsche Telekom AG",
            "postal": "80097",
            "timezone": "Europe/Berlin",
            "readme": "https://ipinfo.io/missingauth"
        }"#,
        "93.254.94.16"
    );

    test_provder!(
        test_ifconfigco_provider,
        IfConfigCo,
        r#"{
          "ip": "93.254.94.16",
          "ip_decimal": 1576951312,
          "country": "Germany",
          "country_iso": "DE",
          "country_eu": true,
          "region_name": "Bavaria",
          "region_code": "BY",
          "zip_code": "97074",
          "city": "Würzburg",
          "latitude": 49.7786,
          "longitude": 9.9308,
          "time_zone": "Europe/Berlin",
          "asn": "AS3320",
          "asn_org": "Deutsche Telekom AG",
          "hostname": "p5dfe5e10.dip0.t-ipconnect.de",
          "user_agent": {
            "product": "vpn-killswitch",
            "version": "0.1.0",
            "raw_value": "vpn-killswitch/0.1.0"
          }
        }"#,
        "93.254.94.16"
    );

    test_provder!(
        test_ipifyorg_provider,
        IpifyOrg,
        r#"{
            "ip": "93.254.94.16"
        }"#,
        "93.254.94.16"
    );

    test_provder!(
        test_ipapicom_provider,
        IpApiCom,
        r#"{
            "query":"93.254.94.16"
        }"#,
        "93.254.94.16"
    );

    test_provder!(
        test_ipwhois_provider,
        IpWhoIs,
        r#"{
          "ip": "93.254.94.16",
          "success": true,
          "type": "IPv4",
          "continent": "Europe",
          "continent_code": "EU",
          "country": "Germany",
          "country_code": "DE",
          "region": "Baden-Württemberg",
          "region_code": "BW",
          "city": "Obrigheim",
          "latitude": 49.3517343,
          "longitude": 9.0933321,
          "is_eu": true,
          "postal": "74847",
          "calling_code": "49",
          "capital": "Berlin",
          "borders": "AT,BE,CH,CZ,DK,FR,LU,NL,PL",
          "flag": {
            "img": "https://cdn.ipwhois.io/flags/de.svg",
            "emoji": "🇩🇪",
            "emoji_unicode": "U+1F1E9 U+1F1EA"
          },
          "connection": {
            "asn": 3320,
            "org": "Deutsche Telekom AG",
            "isp": "Deutsche Telekom AG",
            "domain": "telekom.de"
          },
          "timezone": {
            "id": "Europe/Berlin",
            "abbr": "CEST",
            "is_dst": true,
            "offset": 7200,
            "utc": "+02:00"
          }
        }"#,
        "93.254.94.16"
    );

    test_provder!(
        test_myipio_provider,
        MyIpIo,
        r#"{
          "success": true,
          "ip": "93.254.94.16",
          "type": "IPv4",
          "country": {
            "code": "DE",
            "name": "Germany"
          },
          "region": "Bavaria",
          "city": "Werneck",
          "location": {
            "lat": 49.978,
            "lon": 10.1042
          },
          "timeZone": "Europe/Berlin",
          "asn": {
            "number": 3320,
            "name": "Deutsche Telekom AG",
            "network": "93.224.0.0/11"
          }
        }"#,
        "93.254.94.16"
    );
}
