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

    fn swap(&mut self, a: usize, b: usize) -> () {
        let temp_ip = self.top100_list[a];
        let temp_count = self.top100_counts[a];
        self.top100_list[a] = self.top100_list[b];
        self.top100_counts[a] = self.top100_counts[b];
        self.top100_list[b] = temp_ip;
        self.top100_counts[b] = temp_count;
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
            // If already contained in top 100, we only need to reorder
            let previous_index = self
                .top100_list
                .iter()
                .position(|&x| x == Some(ip_address))
                .unwrap();
            let next_index = self.top100_counts.iter().position(|&x| x < *count).unwrap();
            self.top100_counts[previous_index] = *count;
            self.swap(previous_index, next_index);
        } else if self.top100_size < TOP_COUNT {
            // If we are still finding first 100 IP addreses, just need to add
            self.top100_list[self.top100_size] = Some(ip_address);
            self.top100_counts[self.top100_size] = 1;
            self.top100_size += 1;
        } else if was_on_threshold {
            // If we just crossed the threshold, and may need to move something from outside of the top 100 to inside the top 100
            let opt_previous_index = self.top100_list.iter().position(|&x| x == Some(ip_address));
            let previous_index = match opt_previous_index {
                None => {
                    self.top100_list[TOP_COUNT - 1] = Some(ip_address);
                    self.top100_counts[TOP_COUNT - 1] = *count - 1;
                    TOP_COUNT - 1
                }
                Some(p) => p,
            };

            let next_index = self.top100_counts.iter().position(|&x| x < *count).unwrap();
            self.top100_counts[previous_index] = *count;
            self.swap(previous_index, next_index);
        }
    }

    fn top100(&self) -> [Option<IpAddr>; TOP_COUNT] {
        // println!("{:?}", self.top100_counts);
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
    use rand::{distributions::Uniform, thread_rng, Rng};
    use std::{
        net::{IpAddr, Ipv4Addr},
        time::{Duration, Instant},
    };

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

    // Helper function - sample from array according to a triangle distribution with mode+min 0
    fn sample_triangularly<const N: usize>(size: usize) -> Vec<usize> {
        let mut rng = thread_rng();
        let uniform_distro = Uniform::from(0.0..1.0);
        let uniform_sample: Vec<f64> = rng.sample_iter(uniform_distro).take(N).collect();
        let mut result = Vec::with_capacity(N);
        for y in uniform_sample {
            result.push((size as f64 * (1.0 - f64::sqrt(1.0 - y))) as usize);
        }
        result.try_into().unwrap()
    }

    #[test]
    fn it_handles_40m_inserts_of_20m_ips() {
        let mut ip_store = PulleyIPStore::new();

        let mut rng = thread_rng();
        let mut ipaddrs = vec![Ipv4Addr::new(0, 0, 0, 0); 20000000];
        for ip in &mut ipaddrs {
            *ip = Ipv4Addr::from(rng.gen::<[u8; 4]>());
        }

        let sample_indices: Vec<usize> = sample_triangularly::<40000000>(ipaddrs.len() - 1);

        // Measure insertion perf
        let insertion_start_time = Instant::now();
        for i in sample_indices {
            ip_store.request_handled(IpAddr::V4(ipaddrs[i]));
        }
        let insertion_duration = insertion_start_time.elapsed();
        let amortized_insertion_duration = insertion_duration / 40000000;
        println!(
            "Insertion duration: {:?} total, or {:?} each",
            insertion_duration, amortized_insertion_duration
        );
        assert!(
            amortized_insertion_duration < Duration::from_millis(1),
            "Should take at most 1ms to insert"
        );

        // Measure extracting top 100 perf
        let extraction_start_time = Instant::now();
        ip_store.top100();
        let extraction_duration = extraction_start_time.elapsed();
        println!("Extraction duration: {:?}", extraction_duration);
        assert!(
            amortized_insertion_duration < Duration::from_millis(300),
            "Should take at most 300 ms to extract top 100"
        );

        // Sanity check
        assert!(
            ip_store.request_counts.len() > 15000000,
            "Should have inserted at least 15000000 of the 20000000 values"
        );

        // Sanity check
        let total_insertions = ip_store.request_counts.values().fold(0, |acc, v| acc + v);
        assert_eq!(
            total_insertions, 40000000,
            "Should have inserted every value"
        );
    }
}
