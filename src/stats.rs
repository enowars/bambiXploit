use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

pub struct BambiStats {
    ok_flags: AtomicU64,
}

impl BambiStats {
    pub fn add_ok(&self, diff: u64) {
        self.ok_flags.fetch_add(diff, Ordering::Relaxed);
    }

    pub fn new() -> Self {
        BambiStats {
            ok_flags: AtomicU64::new(0),
        }
    }
}
