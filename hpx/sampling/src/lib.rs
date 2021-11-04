use bit_set::BitSet;
use rand::Rng;

pub const DEFAULT_RESERVOIR_SIZE: usize = 100;

pub fn random_set(size: usize, base: usize) -> BitSet {
    let mut s = BitSet::new();
    let mut chosen = Vec::with_capacity(base);
    let mut i = 0;
    while i < base {
        chosen.insert(i as usize, i);
        s.insert(i);
        i += 1;
    }
    let mut r = rand::thread_rng();
    while i < size {
        let j = r.gen_range(0..i + 1);
        if j < base {
            s.remove(chosen[j]);
            s.insert(i);
            chosen[j] = i;
        }
        i += 1;
    }

    return s;
}
