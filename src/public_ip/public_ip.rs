use crate::public_ip::error::PublicIpLookupError;
use serde::{Deserialize, de::DeserializeOwned};
use std::net::Ipv4Addr;

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct PublicIpLookupResponse {
    pub ip: Ipv4Addr,
    pub asn: Option<String>,
}

use crate::public_ip::provider::PublicIpLookupProvider;

pub async fn lookup<T>(provider: T) -> Result<PublicIpLookupResponse, PublicIpLookupError>
where
    T: PublicIpLookupProvider,
    T::ResponseType: DeserializeOwned,
    PublicIpLookupResponse: TryFrom<T::ResponseType, Error = PublicIpLookupError>,
{
    let http_client = reqwest::Client::new();
    let response = http_client
        .get(provider.url())
        .headers(provider.headers())
        .send()
        .await?;

    let body: T::ResponseType = response.json().await?;
    body.try_into()
}

#[cfg(test)]
mod tests {
    use tokio::task::JoinSet;
    use super::*;
    use crate::public_ip::ifconfigco::IfConfigCo;
    use crate::public_ip::ipapico::IpApiCo;
    use crate::public_ip::ipapicom::IpApiCom;
    use crate::public_ip::ipifyorg::IpifyOrg;
    use crate::public_ip::ipwhois::IpWhoIs;
    use crate::public_ip::myipcom::MyIpCom;
    use crate::public_ip::myipio::MyIpIo;

    #[tokio::test]
    async fn test_lookup() -> Result<(), PublicIpLookupError> {
        let mut join_set = JoinSet::new();
        join_set.spawn(async { lookup(IfConfigCo::new()).await });
        join_set.spawn(async { lookup(MyIpIo::new()).await });
        join_set.spawn(async { lookup(IpApiCom::new()).await });
        join_set.spawn(async { lookup(IpWhoIs::new()).await });
        join_set.spawn(async { lookup(IpApiCo::new()).await });
        join_set.spawn(async { lookup(MyIpCom::new()).await });
        join_set.spawn(async { lookup(IpifyOrg::new()).await });

        let mut last_response: Option<PublicIpLookupResponse> = None;
        while let Some(result) =join_set.join_next().await {
            let response = result.unwrap()?;
            dbg!(&response);

            if last_response.is_none() { // first call
                last_response.replace(response);
                continue;
            }

            if response.asn.is_some() && last_response.as_ref().unwrap().asn.is_some() {
                assert_eq!(response, *last_response.as_ref().unwrap());
            } else { // compare only ips
                assert_eq!(response.ip, last_response.as_ref().unwrap().ip);
            }

            last_response.replace(response);
        }

        Ok(())
    }
}
