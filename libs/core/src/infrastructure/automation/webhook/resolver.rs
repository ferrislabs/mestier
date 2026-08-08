use std::{net::SocketAddr, sync::Arc};

use reqwest::dns::{Addrs, Name, Resolve, Resolving};

use super::address_policy::{AddressVerdict, PrivateNetworkAccess, judge};

/// A DNS resolver that only ever hands back addresses the policy allows.
///
/// Checking the address separately and then letting the HTTP client resolve
/// the name again leaves a window: the second lookup can answer differently
/// from the first, and an attacker controlling the zone only needs the
/// verification and the connection to disagree once. Filtering inside the
/// resolver removes the window — the client connects to exactly what was
/// judged, because there is no second lookup.
pub struct GuardedResolver {
    access: PrivateNetworkAccess,
}

impl GuardedResolver {
    pub fn new(access: PrivateNetworkAccess) -> Arc<Self> {
        Arc::new(Self { access })
    }
}

impl Resolve for GuardedResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let access = self.access;

        Box::pin(async move {
            let host = name.as_str().to_owned();
            let resolved = tokio::net::lookup_host((host.as_str(), 0)).await?;

            let mut allowed: Vec<SocketAddr> = Vec::new();
            let mut refusal: Option<&'static str> = None;

            for address in resolved {
                match judge(address.ip(), access) {
                    AddressVerdict::Allowed => allowed.push(address),
                    AddressVerdict::Refused(reason) => refusal = Some(reason),
                }
            }

            if allowed.is_empty() {
                let reason = refusal.unwrap_or("no address at all");
                return Err(format!("`{host}` resolves to {reason}").into());
            }

            Ok(Box::new(allowed.into_iter()) as Addrs)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn resolve(host: &str, access: PrivateNetworkAccess) -> Result<Vec<SocketAddr>, String> {
        let resolver = GuardedResolver::new(access);
        let name: Name = host.parse().expect("a parseable host");

        resolver
            .resolve(name)
            .await
            .map(|addrs| addrs.collect())
            .map_err(|error| error.to_string())
    }

    #[tokio::test]
    async fn a_name_resolving_to_loopback_is_refused() {
        let error = resolve("localhost", PrivateNetworkAccess::Denied)
            .await
            .expect_err("localhost must not be reachable");

        assert!(error.contains("loopback"), "{error}");
    }

    /// Loopback stays refused even for a self-hosted instance that opened its
    /// LAN: reaching the Mestier process itself is never a webhook.
    #[tokio::test]
    async fn loopback_stays_refused_when_the_lan_is_open() {
        assert!(
            resolve("localhost", PrivateNetworkAccess::Allowed)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn a_name_that_does_not_resolve_is_an_error_not_an_empty_allowance() {
        assert!(
            resolve(
                "this-host-does-not-exist.invalid",
                PrivateNetworkAccess::Denied
            )
            .await
            .is_err()
        );
    }

    /// The refusal names what was wrong, so someone configuring an endpoint
    /// learns why instead of reading a generic connection failure.
    #[tokio::test]
    async fn the_refusal_says_what_was_wrong() {
        let error = resolve("localhost", PrivateNetworkAccess::Denied)
            .await
            .expect_err("refused");

        assert!(error.contains("localhost"), "{error}");
    }
}
