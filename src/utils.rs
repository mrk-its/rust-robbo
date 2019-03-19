use cfg_if::cfg_if;
use crate::types::{Direction, Position};

cfg_if! {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;
        pub use self::console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        pub fn set_panic_hook() {}
    }
}

pub fn reverse_direction((kx, ky): Direction) -> Direction {
    (-kx, -ky)
}

pub fn direction_by_index(index: usize) -> Direction {
    assert!(index < 4);
    [(1, 0), (0, 1), (-1, 0), (0, -1)][index]
}

pub fn direction_to_index(dir: Direction) -> usize {
    match dir {
        (1, 0) => 0,
        (0, 1) => 1,
        (-1, 0) => 2,
        (0, -1) => 3,
        _ => panic!()
    }
}

pub fn rotate_clockwise((x, y): Direction) -> Direction {
    (-y, x)
}

pub fn rotate_counter_clockwise((x, y): Direction) -> Direction {
    (y, -x)
}

pub fn dest_coords((x, y): Position, (kx, ky): Direction) -> Position {
    (x + kx, y + ky)
}

pub fn modulo(n: i32, k: i32) -> i32 {
    ((n % k) + k) % k
}