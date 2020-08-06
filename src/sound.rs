#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i16)]
pub enum Sound {
    Walk = 1,
    Spawn = 2,
    Ammo = 3,
    Key = 4,
    Screw = 5,
    Bomb = 6,
    Door = 7,
    Shot = 8,
    GunShot = 9,
    Teleport = 10,
}
