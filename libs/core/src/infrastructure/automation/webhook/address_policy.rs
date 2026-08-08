use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Whether the deployment lets webhooks reach addresses that are not public.
///
/// This is **instance** configuration, never an organization setting. Hosted,
/// a tenant supplying `http://169.254.169.254/` would be borrowing the
/// server's network rights to read credentials it has no access to; if the
/// tenant could flip this, the guard would protect nobody. Self-hosted, the
/// private range is the user's own LAN — their Odoo, their NAS — and blocking
/// it removes most of the value while protecting nothing, since they are
/// already root on the machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PrivateNetworkAccess {
    #[default]
    Denied,
    Allowed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressVerdict {
    Allowed,
    /// Carries what made it unacceptable, so a user configuring an endpoint
    /// learns why instead of seeing a generic failure.
    Refused(&'static str),
}

/// Judges an address the server is about to connect to.
///
/// Applied to the **resolved** address, never to the hostname: a name is
/// attacker-controlled and can resolve wherever it likes, including to
/// different answers on two consecutive lookups.
pub fn judge(address: IpAddr, access: PrivateNetworkAccess) -> AddressVerdict {
    if let Some(reason) = always_refused(address) {
        return AddressVerdict::Refused(reason);
    }

    if access == PrivateNetworkAccess::Allowed {
        return AddressVerdict::Allowed;
    }

    match private_reason(address) {
        Some(reason) => AddressVerdict::Refused(reason),
        None => AddressVerdict::Allowed,
    }
}

/// Refused whatever the instance allows: these are never a legitimate webhook
/// target, and the cloud metadata endpoint is the whole point of the guard.
fn always_refused(address: IpAddr) -> Option<&'static str> {
    match address {
        IpAddr::V4(v4) => {
            if v4 == Ipv4Addr::new(169, 254, 169, 254) {
                return Some("the cloud metadata endpoint");
            }
            if v4.is_loopback() {
                return Some("a loopback address");
            }
            if v4.is_unspecified() {
                return Some("an unspecified address");
            }
            if v4.is_broadcast() || v4.is_multicast() {
                return Some("a broadcast or multicast address");
            }
            None
        }
        IpAddr::V6(v6) => {
            if let Some(mapped) = v6.to_ipv4_mapped() {
                // `::ffff:169.254.169.254` reaches the same metadata service.
                return always_refused(IpAddr::V4(mapped));
            }
            if v6.is_loopback() {
                return Some("a loopback address");
            }
            if v6.is_unspecified() {
                return Some("an unspecified address");
            }
            if v6.is_multicast() {
                return Some("a multicast address");
            }
            None
        }
    }
}

fn private_reason(address: IpAddr) -> Option<&'static str> {
    match address {
        IpAddr::V4(v4) => {
            if v4.is_private() {
                return Some("a private address");
            }
            if v4.is_link_local() {
                return Some("a link-local address");
            }
            if is_carrier_grade_nat(v4) {
                return Some("a carrier-grade NAT address");
            }
            None
        }
        IpAddr::V6(v6) => {
            if let Some(mapped) = v6.to_ipv4_mapped() {
                return private_reason(IpAddr::V4(mapped));
            }
            if is_unique_local(v6) {
                return Some("a unique-local address");
            }
            if is_link_local_v6(v6) {
                return Some("a link-local address");
            }
            None
        }
    }
}

/// `100.64.0.0/10`. Hand-rolled because `Ipv4Addr::is_shared` is still
/// unstable, and a webhook target behind a carrier's NAT is not reachable
/// from outside it anyway.
fn is_carrier_grade_nat(address: Ipv4Addr) -> bool {
    let octets = address.octets();
    octets[0] == 100 && (64..128).contains(&octets[1])
}

/// `fc00::/7` — the IPv6 equivalent of RFC 1918.
fn is_unique_local(address: Ipv6Addr) -> bool {
    address.octets()[0] & 0xfe == 0xfc
}

/// `fe80::/10`.
fn is_link_local_v6(address: Ipv6Addr) -> bool {
    let octets = address.octets();
    octets[0] == 0xfe && octets[1] & 0xc0 == 0x80
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v4(address: &str) -> IpAddr {
        address.parse().unwrap()
    }

    fn v6(address: &str) -> IpAddr {
        address.parse().unwrap()
    }

    #[test]
    fn a_public_address_is_allowed() {
        assert_eq!(
            judge(v4("93.184.216.34"), PrivateNetworkAccess::Denied),
            AddressVerdict::Allowed
        );
        assert_eq!(
            judge(v6("2606:2800:220:1::248"), PrivateNetworkAccess::Denied),
            AddressVerdict::Allowed
        );
    }

    #[test]
    fn private_ranges_are_refused_by_default() {
        for address in ["10.0.0.5", "172.16.0.1", "192.168.1.50", "100.64.0.1"] {
            assert!(
                matches!(
                    judge(v4(address), PrivateNetworkAccess::Denied),
                    AddressVerdict::Refused(_)
                ),
                "{address} must be refused"
            );
        }
        assert!(matches!(
            judge(v6("fd00::1"), PrivateNetworkAccess::Denied),
            AddressVerdict::Refused(_)
        ));
        assert!(matches!(
            judge(v6("fe80::1"), PrivateNetworkAccess::Denied),
            AddressVerdict::Refused(_)
        ));
    }

    /// The reason the guard exists. On a hosted instance this address returns
    /// the machine's IAM credentials to whoever can make the server fetch it.
    #[test]
    fn the_cloud_metadata_endpoint_is_refused_even_when_private_access_is_allowed() {
        assert_eq!(
            judge(v4("169.254.169.254"), PrivateNetworkAccess::Allowed),
            AddressVerdict::Refused("the cloud metadata endpoint")
        );
    }

    /// An IPv4-mapped IPv6 address reaches exactly the same host. Judging the
    /// two forms differently is how a guard gets walked around.
    #[test]
    fn an_ipv4_mapped_address_is_judged_as_its_ipv4_form() {
        assert_eq!(
            judge(v6("::ffff:169.254.169.254"), PrivateNetworkAccess::Allowed),
            AddressVerdict::Refused("the cloud metadata endpoint")
        );
        assert!(matches!(
            judge(v6("::ffff:10.0.0.5"), PrivateNetworkAccess::Denied),
            AddressVerdict::Refused(_)
        ));
    }

    #[test]
    fn loopback_is_refused_whatever_the_instance_allows() {
        for access in [PrivateNetworkAccess::Denied, PrivateNetworkAccess::Allowed] {
            assert!(matches!(
                judge(v4("127.0.0.1"), access),
                AddressVerdict::Refused(_)
            ));
            assert!(matches!(
                judge(v6("::1"), access),
                AddressVerdict::Refused(_)
            ));
        }
    }

    /// A self-hosted artisan's integrations live on their own LAN. Blocking it
    /// protects nothing — they are already root on the machine — and removes
    /// most of the value.
    #[test]
    fn a_self_hosted_instance_can_reach_its_own_lan() {
        assert_eq!(
            judge(v4("192.168.1.50"), PrivateNetworkAccess::Allowed),
            AddressVerdict::Allowed
        );
    }

    #[test]
    fn the_default_is_to_refuse() {
        assert_eq!(
            PrivateNetworkAccess::default(),
            PrivateNetworkAccess::Denied
        );
    }
}
