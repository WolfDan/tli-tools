use super::types::EventKind;

const HASH_TEXT_LEN_MAX: usize = 65_536;
const FNV_OFFSET_BASIS: u32 = 2_166_136_261;
const FNV_PRIME: u32 = 16_777_619;

const _: () = {
    assert!(HASH_TEXT_LEN_MAX > 0);
    assert!(FNV_PRIME > 0);
};

#[inline]
pub const fn hash_seed(text: &str) -> u32 {
    let bytes = text.as_bytes();
    assert!(bytes.len() <= HASH_TEXT_LEN_MAX);
    let mut h: u32 = FNV_OFFSET_BASIS;
    let mut i = 0;
    while i < bytes.len() && i < HASH_TEXT_LEN_MAX {
        h ^= bytes[i] as u32;
        h = h.wrapping_mul(FNV_PRIME);
        i += 1;
    }
    assert!(i == bytes.len());
    h
}

#[inline]
pub const fn refit_seed(slot: u8) -> u32 {
    let base = hash_seed("ANALYZER|refit|");
    let digit_offset = slot % 10;
    assert!(digit_offset < 10);
    let d = (b'0' + digit_offset) as u32;
    assert!(d >= (b'0' as u32));
    assert!(d <= (b'9' as u32));
    let h1 = (base ^ d).wrapping_mul(FNV_PRIME);
    let h2 = (h1 ^ (b'|' as u32)).wrapping_mul(FNV_PRIME);
    (h2 ^ (b'0' as u32)).wrapping_mul(FNV_PRIME)
}

#[inline]
fn with_rng<R>(rng: &mut u32, f: impl FnOnce(&mut rand::rngs::SmallRng) -> R) -> R {
    use rand::{RngCore, SeedableRng};
    let seed_u64 = *rng as u64;
    assert!(seed_u64 <= u32::MAX as u64);
    let mut r = rand::rngs::SmallRng::seed_from_u64(seed_u64);
    let out = f(&mut r);
    let next_seed = r.next_u32();
    *rng = next_seed;
    assert_eq!(*rng, next_seed);
    out
}

#[inline]
pub fn next_rand(rng: &mut u32) -> f64 {
    use rand::Rng;
    let val = with_rng(rng, |r| r.r#gen::<f64>());
    assert!(val.is_finite());
    assert!((0.0..1.0).contains(&val));
    val
}

#[inline]
pub fn pick_index(rng: &mut u32, weights: &[f64]) -> usize {
    use rand::distributions::{Distribution, WeightedIndex};
    assert!(!weights.is_empty());
    assert!(weights.iter().all(|w| w.is_finite()));
    let idx = with_rng(rng, |r| {
        WeightedIndex::new(weights.iter().map(|w| w.max(0.0)))
            .map(|dist| dist.sample(r))
            .unwrap_or(0)
    });
    assert!(idx < weights.len());
    idx
}

#[inline]
pub fn pick_one<T: Copy>(rng: &mut u32, list: &[T]) -> T {
    use rand::seq::SliceRandom;
    assert!(!list.is_empty());
    let len = list.len();
    assert!(len >= 1);
    with_rng(rng, |r| *list.choose(r).expect("pick_one on empty slice"))
}

#[inline]
pub fn roll_event_from_pool(rng: &mut u32, pool: &[EventKind]) -> EventKind {
    use rand::distributions::{Distribution, WeightedIndex};
    use rand::seq::SliceRandom;
    assert!(pool.len() <= 32);
    if pool.is_empty() {
        return EventKind::Chain;
    }
    let chosen = with_rng(rng, |r| {
        if let Ok(dist) = WeightedIndex::new(pool.iter().map(|ev| ev.weight())) {
            pool[dist.sample(r)]
        } else {
            *pool.choose(r).unwrap()
        }
    });
    assert!(chosen.weight() > 0);
    chosen
}

#[inline]
pub fn shuffle_slice<T: Copy>(rng: &mut u32, slice: &mut [T]) {
    use rand::seq::SliceRandom;
    let len_before = slice.len();
    assert!(len_before <= 64);
    if len_before <= 1 {
        return;
    }
    with_rng(rng, |r| slice.shuffle(r));
    assert_eq!(slice.len(), len_before);
}
