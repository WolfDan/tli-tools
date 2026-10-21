use super::types::EventKind;

#[inline]
pub const fn hash_seed(text: &str) -> u32 {
    let bytes = text.as_bytes();
    let mut h: u32 = 2166136261;
    let mut i = 0;
    while i < bytes.len() {
        h ^= bytes[i] as u32;
        h = h.wrapping_mul(16777619);
        i += 1;
    }
    h
}

#[inline]
pub const fn refit_seed(slot: u8) -> u32 {
    let base = hash_seed("ANALYZER|refit|");
    let d = (b'0' + (slot % 10)) as u32;
    let h1 = (base ^ d).wrapping_mul(16777619);
    let h2 = (h1 ^ (b'|' as u32)).wrapping_mul(16777619);
    (h2 ^ (b'0' as u32)).wrapping_mul(16777619)
}

#[inline]
fn with_rng<R>(rng: &mut u32, f: impl FnOnce(&mut rand::rngs::SmallRng) -> R) -> R {
    use rand::{RngCore, SeedableRng};
    let mut r = rand::rngs::SmallRng::seed_from_u64(*rng as u64);
    let out = f(&mut r);
    *rng = r.next_u32();
    out
}

#[inline]
pub fn next_rand(rng: &mut u32) -> f64 {
    use rand::Rng;
    with_rng(rng, |r| r.r#gen::<f64>())
}

#[inline]
pub fn pick_index(rng: &mut u32, weights: &[f64]) -> usize {
    use rand::distributions::{Distribution, WeightedIndex};
    with_rng(rng, |r| {
        WeightedIndex::new(weights.iter().map(|w| w.max(0.0)))
            .map(|dist| dist.sample(r))
            .unwrap_or(0)
    })
}

#[inline]
pub fn pick_one<T: Copy>(rng: &mut u32, list: &[T]) -> T {
    use rand::seq::SliceRandom;
    with_rng(rng, |r| *list.choose(r).expect("pick_one on empty slice"))
}

#[inline]
pub fn roll_event_from_pool(rng: &mut u32, pool: &[EventKind]) -> EventKind {
    use rand::distributions::{Distribution, WeightedIndex};
    use rand::seq::SliceRandom;
    if pool.is_empty() {
        return EventKind::Chain;
    }
    with_rng(rng, |r| {
        if let Ok(dist) = WeightedIndex::new(pool.iter().map(|ev| ev.weight())) {
            pool[dist.sample(r)]
        } else {
            *pool.choose(r).unwrap()
        }
    })
}

#[inline]
pub fn shuffle_slice<T: Copy>(rng: &mut u32, slice: &mut [T]) {
    use rand::seq::SliceRandom;
    if slice.len() <= 1 {
        return;
    }
    with_rng(rng, |r| slice.shuffle(r));
}
