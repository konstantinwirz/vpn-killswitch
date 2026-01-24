use std::net::Ipv4Addr;
use std::str::FromStr;
use serde::Deserialize;
use crate::public_ip::error::PublicIpLookupError;
use crate::public_ip::provider::PublicIpLookupProvider;
use crate::public_ip::public_ip::PublicIpLookupResponse;

/// Represents https://ipify.org
///
/// Rate Limit: unlimited
///
/// Test it:
/// ```shell
/// curl -H "Accept: application/json" https://api.ipify.org?format=json
/// ```
#[derive(Clone, Debug)]
pub struct IpifyOrg {
    url: &'static str
}

const URL: &str = "https://api.ipify.org?format=json";

impl IpifyOrg {
    pub fn new() -> Self {
        Self { url: URL }
    }
}

/// Represents API response from https://api.ipify.org
///
/// Example:
/// ```json
/// {"ip":"93.254.94.16"}
/// ```
#[derive(Clone, Debug, Deserialize)]
pub struct IpifyOrgResponse {
    ip: String,
}

impl PublicIpLookupProvider for IpifyOrg {
    type ResponseType = IpifyOrgResponse;

    fn url(&self) -> &'static str {
        self.url
    }
}

impl TryFrom<IpifyOrgResponse> for PublicIpLookupResponse {
    type Error = PublicIpLookupError;

    fn try_from(value: IpifyOrgResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            ip: Ipv4Addr::from_str(&value.ip)?,
            asn: None
        })
    }
}
