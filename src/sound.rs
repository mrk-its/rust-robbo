use std::cell::RefCell;

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
    Capsule = 11,
    Burn = 12,
}

pub struct Sounds {
    sounds: RefCell<Vec<Sound>>
}

impl Sounds {
    pub fn new() -> Sounds {
        Sounds {
            sounds: RefCell::new(Vec::new())
        }
    }
    pub fn play_sound(&self, sound: Sound) {
        let mut sounds = self.sounds.borrow_mut();
        sounds.push(sound)
    }
    pub fn get_sounds(&self) -> Vec<Sound> {
        let mut sounds = self.sounds.borrow_mut();
        sounds.drain(..).collect()
    }
}