static mut _SEED: u32 = 1;

const _MOD: u64 = 2147483647;
const _K: u64 = 16807;

pub fn seed(seed: u32) {
    unsafe {
        _SEED = seed;
    }
}
pub fn next() -> u32 {
    unsafe {
        _SEED = ((_SEED as u64) * _K % _MOD) as u32;
        _SEED
    }
}
pub fn random() -> f64 {
    (next() as f64) / 2147483647.0
}

pub fn randrange(stop: u32) -> u32 {
    next() % stop
}
