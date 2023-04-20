use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::net::IpAddr;

pub trait IPStore {
    fn request_handled(&mut self, ip_address: IpAddr) -> ();
    fn top100(&self) -> &[IpAddr; 100];
    fn clear(&mut self) -> ();
}

pub struct PulleyIPStore<'a> {
    request_counts: HashMap<IpAddr, &'a u32>,
    top100_set: HashSet<IpAddr>,
    top100_list: [Option<(IpAddr, &'a u32)>; 100],
}

impl<'a> PulleyIPStore<'a> {
    fn new() -> PulleyIPStore<'a> {
        PulleyIPStore {
            request_counts: HashMap::new(),
            top100_set: HashSet::new(),
            top100_list: [None; 100],
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use crate::PulleyIPStore;

    #[test]
    fn it_works() {
        let mut j = 0;
        for _i in 1..20000000 {
            j += 1;
        }
        assert_eq!(j, 4);
    }
}
