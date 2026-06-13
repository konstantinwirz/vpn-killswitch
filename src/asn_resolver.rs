use std::net::Ipv4Addr;

use hickory_resolver::{
    Resolver,
    config::{CLOUDFLARE, ResolverConfig},
    net::runtime::TokioRuntimeProvider,
};
use log::{debug, error, info};

#[derive(Debug, Clone)]
pub struct AsnResolver {
    base_url: String,
    dns_resolver: Resolver<TokioRuntimeProvider>,
}

/// creates a dns resolver
///
/// panics if the resolver couldn't be created, which is ok,
/// the app cannot work without it
fn create_dns_resolver() -> Resolver<TokioRuntimeProvider> {
    Resolver::builder_with_config(
        ResolverConfig::udp_and_tcp(&CLOUDFLARE),
        TokioRuntimeProvider::default(),
    )
    .build()
    .expect("resolver must exist")
}

impl Default for AsnResolver {
    fn default() -> Self {
        Self {
            base_url: String::from("origin.asn.cymru.com"),
            dns_resolver: create_dns_resolver(),
        }
    }
}

impl AsnResolver {
    #[cfg(test)]
    fn with_url(url: &str) -> Self {
        Self {
            base_url: String::from(url),
            dns_resolver: create_dns_resolver(),
        }
    }

    pub async fn resolve(&self, ip: &Ipv4Addr) -> anyhow::Result<String> {
        debug!("about to resolve an asn for {ip}");

        let ip_octets = ip.octets();
        let query = format!(
            "{}.{}.{}.{}.{}",
            ip_octets[3], ip_octets[2], ip_octets[1], ip_octets[0], self.base_url
        );

        debug!("about to resolve an asn for {ip} using query {query}");

        let response = self.dns_resolver.txt_lookup(query).await?;

        debug!("got response {:?}", &response);

        for record in response.answers() {
            if let Some((asn, _)) = record.data.to_string().split_once("|") {
                let asn = asn.trim();
                info!("asn successfully resolved: {asn}");
                return Ok(String::from(asn));
            }
        }

        error!("failed to resolve an asn for {}", &ip);

        anyhow::bail!("couldn't resolve asn for {}", ip)
    }
}
