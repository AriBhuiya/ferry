use crate::discovery::FerryService;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

/// IPV4 Ips are preferred over ipv6
pub(crate) fn sort_addrs_by_preference(service: &mut FerryService) {
    service.addrs.sort_by_key(score_ip);
}

pub(crate) fn get_best_addr(service: &mut FerryService) -> Option<SocketAddr> {
    service.addrs.iter().min_by_key(|ip| score_ip(ip)).cloned()
}

fn score_ip(sa: &SocketAddr) -> i32 {
    match sa.ip() {
        IpAddr::V4(v4) => score_ipv4(&v4),
        IpAddr::V6(v6) => score_ipv6(&v6),
    }
}

fn score_ipv4(addr: &Ipv4Addr) -> i32 {
    let [a, b, c, _] = addr.octets();
    // Lower score is better
    // Hard demotions first
    if a == 127 {
        return 100;
    } // loopback
    if a == 10 && b == 255 && c == 255 {
        return 80;
    } // WSL-ish 10.255.255.*
    if a == 172 && (b == 17 || b == 18 || b == 19) {
        return 70;
    } // Docker bridges typically
    if a == 172 && b == 31 {
        return 60;
    } // WSL NAT side

    // Common Ips
    if a == 192 && b == 168 {
        return 0;
    }
    if a == 172 && (16..=31).contains(&b) {
        return 5;
    }
    if a == 10 {
        return 10;
    }

    if a == 169 && b == 254 {
        return 20;
    } // APIPA

    // Everything else (public/CGNAT/etc.)
    30
}

fn score_ipv6(addr: &Ipv6Addr) -> i32 {
    let seg0 = addr.segments()[0];
    // Link-local fe80::
    if (seg0 & 0xffc0) == 0xfe80 {
        return 220;
    }
    // Unique-local fc00::/7 (private)
    if (seg0 & 0xfe00) == 0xfc00 {
        return 210;
    }
    // Loopback ::1
    if addr.is_loopback() {
        return 230;
    }
    // Global unicast
    200
}

// inline tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::net::{SocketAddrV4, SocketAddrV6};
    #[test]
    fn test_score_ipv4() {
        let cases = vec![
            (Ipv4Addr::new(127, 0, 5, 1), 100),
            (Ipv4Addr::new(127, 0, 0, 1), 100),
            (Ipv4Addr::new(172, 0, 0, 2), 30),
            (Ipv4Addr::new(172, 16, 0, 2), 5),
            (Ipv4Addr::new(172, 18, 0, 2), 70),
            (Ipv4Addr::new(10, 15, 3, 7), 10),
            (Ipv4Addr::new(192, 168, 10, 0), 0),
            (Ipv4Addr::new(192, 170, 15, 10), 30),
        ];
        for (c, e) in &cases {
            // c: &Ipv4Addr, e: &i32
            assert_eq!(score_ipv4(c), *e, "failed for {c:?}");
        }
    }

    #[test]
    fn test_score_ipv6() {
        let cases = vec![
            // Loopback ::1
            (Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), 230),
            // Link-local fe80::/10 (your mask checks (seg0 & 0xffc0) == 0xfe80)
            (Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1), 220),
            (Ipv6Addr::new(0xfebf, 0, 0, 0, 0, 0, 0, 2), 220),
            // Unique-local fc00::/7 (private)
            (Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 0), 210),
            (Ipv6Addr::new(0xfd12, 0x3456, 0, 0, 0, 0, 0, 9), 210),
            // Everything else falls through to "global unicast" bucket (per function)
            (Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0, 0, 0, 1), 200),
            (Ipv6Addr::new(0x2a00, 0, 0, 0, 0, 0, 0, 0x1234), 200),
            (Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0, 1), 200),
        ];

        for (c, e) in &cases {
            assert_eq!(score_ipv6(c), *e, "failed for {c}");
        }
    }

    fn v4(a: u8, b: u8, c: u8, d: u8, port: u16) -> SocketAddr {
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(a, b, c, d), port))
    }

    #[allow(clippy::too_many_arguments)]
    fn v6(
        s0: u16,
        s1: u16,
        s2: u16,
        s3: u16,
        s4: u16,
        s5: u16,
        s6: u16,
        s7: u16,
        port: u16,
    ) -> SocketAddr {
        SocketAddr::V6(SocketAddrV6::new(
            Ipv6Addr::new(s0, s1, s2, s3, s4, s5, s6, s7),
            port,
            0,
            0,
        ))
    }

    #[test]
    fn sort_addrs_orders_by_score_ipv4_then_ipv6() {
        // one exemplar per distinct score to avoid tie-order assumptions
        let port = 1234;
        let addr_0 = v4(192, 168, 1, 10, port); // score 0
        let addr_5 = v4(172, 16, 0, 1, port); // score 5
        let addr_10 = v4(10, 1, 2, 3, port); // score 10
        let addr_20 = v4(169, 254, 8, 8, port); // score 20
        let addr_30 = v4(8, 8, 4, 4, port); // score 30 (public)
        let addr_60 = v4(172, 31, 0, 1, port); // score 60
        let addr_70 = v4(172, 18, 0, 1, port); // score 70
        let addr_80 = v4(10, 255, 255, 1, port); // score 80
        let addr_100 = v4(127, 0, 0, 1, port); // score 100

        let addr_200 = v6(0x2001, 0x0db8, 0, 0, 0, 0, 0, 1, port); // score 200 (global)
        let addr_210 = v6(0xfd00, 0, 0, 0, 0, 0, 0, 1, port); // score 210 (unique-local)
        let addr_220 = v6(0xfe80, 0, 0, 0, 0, 0, 0, 1, port); // score 220 (link-local)
        let addr_230 = v6(0, 0, 0, 0, 0, 0, 0, 1, port); // score 230 (loopback)

        let addrs = vec![
            addr_220, addr_5, addr_80, addr_30, addr_210, addr_0, addr_230, addr_60, addr_100,
            addr_200, addr_20, addr_10, addr_70,
        ];
        let mut svc = new_ferry_service(addrs);

        // Act
        sort_addrs_by_preference(&mut svc);

        let expected = vec![
            addr_0,   // 0
            addr_5,   // 5
            addr_10,  // 10
            addr_20,  // 20
            addr_30,  // 30
            addr_60,  // 60
            addr_70,  // 70
            addr_80,  // 80
            addr_100, // 100
            addr_200, // 200
            addr_210, // 210
            addr_220, // 220
            addr_230, // 230
        ];

        assert_eq!(svc.addrs, expected, "addresses not sorted by preference");
    }

    #[test]
    fn sort_addrs_handles_empty_and_singleton() {
        // Empty
        let mut empty = FerryService {
            instance: String::new(),
            fullname: String::new(),
            host: String::new(),
            port: 0,
            addrs: vec![],
            txt: HashMap::new(),
        };
        sort_addrs_by_preference(&mut empty);
        assert!(empty.addrs.is_empty());

        let only = v4(192, 168, 0, 2, 5353);
        let mut one = FerryService {
            addrs: vec![only],
            ..empty.clone()
        };
        sort_addrs_by_preference(&mut one);
        assert_eq!(one.addrs, vec![only]);
    }

    fn new_ferry_service(addrs: Vec<SocketAddr>) -> FerryService {
        FerryService {
            instance: "svc".into(),
            fullname: "svc._tcp.local.".into(),
            host: "host.local.".into(),
            port: 1234,
            addrs,
            txt: HashMap::new(),
        }
    }
    #[test]
    fn get_best_addr_returns_none_when_empty() {
        let mut svc = new_ferry_service(vec![]);
        assert!(get_best_addr(&mut svc).is_none());
        assert!(svc.addrs.is_empty(), "function should not mutate addrs");
    }

    #[test]
    fn get_best_addr_picks_lowest_score_ipv4() {
        // 192.168.*.* => 0 (best), 172.16.*.* => 5, 8.8.8.8 => 30, 127.0.0.1 => 100
        let best = v4(192, 168, 1, 10, 1234); // 0
        let others = [
            v4(172, 16, 0, 1, 1234), // 5
            v4(8, 8, 8, 8, 1234),    // 30
            v4(127, 0, 0, 1, 1234),  // 100
        ];
        let mut svc = new_ferry_service({
            let v = vec![others[0], best, others[1], others[2]];
            v
        });

        let chosen = get_best_addr(&mut svc);
        assert_eq!(chosen, Some(best));
        assert_eq!(
            svc.addrs,
            vec![others[0], best, others[1], others[2]],
            "get_best_addr should not reorder addrs"
        );
    }

    #[test]
    fn get_best_addr_ipv4_beats_ipv6() {
        // IPv4 10.x => 10; IPv6 globals => 200+
        let v4_ok = v4(10, 0, 0, 1, 5555); // score 10
        let v6_gua = v6(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1, 5555); // score 200
        let mut svc = new_ferry_service(vec![v6_gua, v4_ok]);
        let chosen = get_best_addr(&mut svc);
        assert_eq!(chosen, Some(v4_ok));
    }

    #[test]
    fn get_best_addr_breaks_ties_by_first_occurrence() {
        // Two addresses with equal best score (0): first one should be chosen.
        let first = v4(192, 168, 0, 2, 6000); // score 0
        let second = v4(192, 168, 0, 3, 6000); // score 0
        let mut svc = new_ferry_service(vec![first, second]);

        let chosen = get_best_addr(&mut svc);
        assert_eq!(
            chosen,
            Some(first),
            "min_by_key returns the first minimum encountered"
        );
    }

    #[test]
    fn get_best_addr_handles_ipv6_link_local_vs_unique_local() {
        // link-local => 220, unique-local => 210; unique-local should win
        let ll = v6(0xfe80, 0, 0, 0, 0, 0, 0, 1, 7000); // 220
        let ul = v6(0xfd00, 0, 0, 0, 0, 0, 0, 1, 7000); // 210
        let mut svc = new_ferry_service(vec![ll, ul]);

        let chosen = get_best_addr(&mut svc);
        assert_eq!(chosen, Some(ul));
    }
}
