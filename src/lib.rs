use std::collections::HashMap;
use std::net::IpAddr;

const IP_COUNT: usize = 20000000;
const TOP_COUNT: usize = 100;

pub trait IPStore {
    fn request_handled(&mut self, ip_address: IpAddr) -> ();
    fn top100(&self) -> [Option<IpAddr>; TOP_COUNT];
    fn clear(&mut self) -> ();
}

struct PulleyIPStore {
    request_counts: HashMap<IpAddr, u32>,
    top100_list: [Option<IpAddr>; TOP_COUNT],
    top100_counts: [u32; TOP_COUNT],
    top100_size: usize,
}

impl PulleyIPStore {
    pub fn new() -> PulleyIPStore {
        PulleyIPStore {
            request_counts: HashMap::with_capacity(IP_COUNT),
            top100_list: [None; TOP_COUNT],
            top100_counts: [0; TOP_COUNT],
            top100_size: 0,
        }
    }
}

impl IPStore for PulleyIPStore {
    fn request_handled(&mut self, ip_address: IpAddr) -> () {
        let count = self.request_counts.entry(ip_address).or_insert(0);

        let threshold = self.top100_counts[TOP_COUNT - 1];
        let was_in_top100 = *count > threshold;
        let was_on_threshold = *count == threshold;
        *count += 1;
        if was_in_top100 {
            let previous_index = self
                .top100_list
                .iter()
                .position(|&x| x == Some(ip_address))
                .unwrap();
            let next_index = self
                .top100_counts
                .iter()
                .rev()
                .position(|&x| x < *count)
                .unwrap_or(0);
            self.top100_list[previous_index] = self.top100_list[next_index];
            self.top100_counts[previous_index] = self.top100_counts[next_index];
            self.top100_list[next_index] = Some(ip_address);
            self.top100_counts[next_index] = *count;
        } else if self.top100_size < TOP_COUNT {
            // If we are still finding first 100 IP addreses, just need to add
            self.top100_list[self.top100_size] = Some(ip_address);
            self.top100_counts[self.top100_size] = 1;
            self.top100_size += 1;
        } else if was_on_threshold {
        }
    }

    fn top100(&self) -> [Option<IpAddr>; TOP_COUNT] {
        println!("{:?}", self.top100_counts);
        self.top100_list
    }

    fn clear(&mut self) -> () {
        self.request_counts = HashMap::new();
        self.top100_list = [None; TOP_COUNT];
        self.top100_counts = [0; TOP_COUNT];
        self.top100_size = 0;
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
        assert_eq!(ip_store.top100()[2], None, "Third element should be None");
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
        assert_eq!(ip_store.top100()[2], None, "Third element should be None");
    }

    // Helper function - sample from array according to a triangle distribution with mode 0
    fn sample_triangularly<X>(arr: &[X]) -> &X {
        let mut rng = thread_rng();
        let size = arr.len() as f64;
        let index = size - size * f64::sqrt(1.0 - rng.gen::<f64>());
        &arr[index as usize]
    }

    #[test]
    fn it_handles_40m_inserts() {
        let mut ip_store = PulleyIPStore::new();

        let mut rng = thread_rng();
        let mut ipaddrs = vec![Ipv4Addr::new(0, 0, 0, 0); 20000000];
        for ip in &mut ipaddrs {
            *ip = Ipv4Addr::from(rng.gen::<[u8; 4]>());
        }
        for _ in 0..40000000 {
            ip_store.request_handled(IpAddr::V4(*sample_triangularly(&ipaddrs)));
        }
        ip_store.top100();
    }
}
