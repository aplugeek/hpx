use bit_set::BitSet;
use std::sync::atomic::AtomicUsize;

pub struct ContextState {
    pub sampling: BitSet,
    pub counter: AtomicUsize,
}

impl ContextState {
    pub fn should_sampling(&self, sn: usize) -> bool {
        self.sampling.contains(sn)
    }
}
