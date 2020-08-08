use types;

pub const MOVEABLE: u16 = 1;
pub const DESTROYABLE: u16 = 2;
pub const UNDESTROYABLE: u16 = 4;
pub const COLLECTABLE: u16 = 8;
pub const DEADLY: u16 = 16;

pub const ALL_DIRS: &[types::Direction] = &[(1, 0), (0, 1), (-1, 0), (0, -1)];
