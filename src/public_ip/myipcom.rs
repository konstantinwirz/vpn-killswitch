use std::net::Ipv4Addr;
use std::str::FromStr;
use serde::Deserialize;
use crate::public_ip::error::PublicIpLookupError;
use crate::public_ip::provider::PublicIpLookupProvider;
use crate::public_ip::public_ip::PublicIpLookupResponse;

/// Represents https://myip.com
///
/// rate limit: unlimited
///
/// Test it:
/// ```shell
/// curl -H "Accept: application/json" https://api.myip.com
/// ```
pub struct MyIpCom {
    url: &'static str
}

const URL: &str = "https://api.myip.com";

impl MyIpCom {
    pub fn new() -> Self {
        Self { url: URL }
    }
}

/// Represents API response
///
/// Example:
/// ```json
/// {
///     "ip":"93.254.94.16",
///     "country":"Germany","cc":"DE"
/// }
/// ```
#[derive(Deserialize, Debug, Clone)]
pub struct MyIpComResponse {
    ip: String,
}

impl PublicIpLookupProvider for MyIpCom {
    type ResponseType = MyIpComResponse;

    #[inline]
    fn url(&self) -> &'static str {
        self.url
    }
}

impl TryFrom<MyIpComResponse> for PublicIpLookupResponse {
    type Error = PublicIpLookupError;

    fn try_from(value: MyIpComResponse) -> Result<Self, Self::Error> {
        Ok(
            Self {
                ip: Ipv4Addr::from_str(&value.ip)?,
                asn: None
            }
        )
    }
}
