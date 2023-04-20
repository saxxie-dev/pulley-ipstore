use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::net::IpAddr;

pub trait IPStore {
    fn request_handled(&mut self, ip_address: IpAddr) -> ();
    fn top100(&self) -> [Option<IpAddr>; 100];
    fn clear(&mut self) -> ();
}

struct PulleyIPStore<'a> {
    request_counts: HashMap<IpAddr, &'a u32>,
    top100_set: HashSet<IpAddr>,
    top100_list: [Option<(IpAddr, &'a u32)>; 100],
}

impl<'a> PulleyIPStore<'a> {
    pub fn new() -> PulleyIPStore<'a> {
        PulleyIPStore {
            request_counts: HashMap::new(),
            top100_set: HashSet::new(),
            top100_list: [None; 100],
        }
    }
}

impl<'a> IPStore for PulleyIPStore<'a> {
    fn request_handled(&mut self, ip_address: IpAddr) -> () {
        let count = &1;
        self.request_counts.insert(ip_address, count);
        self.top100_list[0] = Some((ip_address, count));
        self.top100_set.insert(ip_address);
    }

    fn top100(&self) -> [Option<IpAddr>; 100] {
        self.top100_list
            .map(|x| -> Option<IpAddr> { x.map(|y| -> IpAddr { y.0 }) })
    }

    fn clear(&mut self) -> () {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use rand::{thread_rng, Rng};
    use std::net::{IpAddr, Ipv4Addr};

    use crate::{IPStore, PulleyIPStore};

    // Helper function - generate random IPv4 address
    fn random_ip() -> IpAddr {
        let mut rng = thread_rng();
        IpAddr::V4(Ipv4Addr::new(
            rng.gen_range(0..255),
            rng.gen_range(0..255),
            rng.gen_range(0..255),
            rng.gen_range(0..255),
        ))
    }

    #[test]
    fn it_handles_a_single_request() {
        let mut ip_store = PulleyIPStore::new();
        let my_ip: IpAddr = random_ip();
        ip_store.request_handled(my_ip);
        assert_eq!(
            ip_store.top100()[0],
            Some(my_ip),
            "First element should be the ip address which was added"
        );
        assert_eq!(ip_store.top100()[1], None, "Second element should be None");
    }

    #[test]
    fn it_successfully_clears() {
        let mut ip_store = PulleyIPStore::new();
        let my_ip: IpAddr = random_ip();
        ip_store.request_handled(my_ip);
        ip_store.clear();
        assert_eq!(
            ip_store.top100()[0],
            None,
            "First element should be None after clearing"
        );
    }

    #[test]
    fn it_handles_multiple_request() {
        let mut ip_store = PulleyIPStore::new();
        let ip1: IpAddr = random_ip();
        let ip2: IpAddr = random_ip();
        ip_store.request_handled(ip1);
        ip_store.request_handled(ip2);

        assert_eq!(
            ip_store.top100()[0],
            Some(ip1),
            "First element should be the first ip address which was added"
        );
        assert_eq!(
            ip_store.top100()[1],
            Some(ip2),
            "Second element should be second ip address which was added"
        );
        assert_eq!(ip_store.top100()[1], None, "Third element should be None");
    }

    #[test]
    fn it_handles_reordering_after_multiple_request() {
        let mut ip_store = PulleyIPStore::new();
        let ip1: IpAddr = random_ip();
        let ip2: IpAddr = random_ip();
        ip_store.request_handled(ip1);
        ip_store.request_handled(ip2);
        ip_store.request_handled(ip2);

        assert_eq!(
            ip_store.top100()[0],
            Some(ip2),
            "First element should be the second ip address, which was handled twice"
        );
        assert_eq!(
            ip_store.top100()[1],
            Some(ip1),
            "Second element should be first ip address, which was handled once"
        );
        assert_eq!(ip_store.top100()[1], None, "Third element should be None");
    }
}
