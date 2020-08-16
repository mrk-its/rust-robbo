use super::{Item, SimpleItem};
use consts::DESTROYABLE;
use log::log;
use tiles::Tiles;
use types::{Action, Actions, Direction, Kind};
use utils::direction_to_index;
use sound::Sound;

#[derive(Debug)]
pub struct Inventory {
    pub keys: usize,
    pub bullets: usize,
    pub screws: usize,
}

impl Inventory {
    pub fn new() -> Inventory {
        Inventory {
            keys: 0,
            bullets: 0,
            screws: 0,
        }
    }
    pub fn collect(&mut self, kind: Kind) -> Actions {
        match kind {
            Kind::Ammo => {
                self.bullets += 9;
                Actions::single(Action::PlaySound(Sound::Ammo))
            }
            Kind::Key => {
                self.keys += 1;
                Actions::single(Action::PlaySound(Sound::Key))
            }
            Kind::Screw => {
                self.screws += 1;
                Actions::single(Action::PlaySound(Sound::Screw))
            }
            _ => Actions::empty()
        }
    }
    pub fn show(&self) {
        log(&format!("{:?}", self));
    }
}

#[derive(Debug)]
pub struct Robbo {
    simple_item: SimpleItem,
    direction: Direction,
    moving_direction: Option<Direction>,
    shot_direction: Option<Direction>,
    pub inventory: Inventory,
    pub is_hidden: bool,
    pub is_killed: bool,
}
impl Robbo {
    pub fn new() -> Robbo {
        Robbo {
            direction: (-1, 0),
            shot_direction: None,
            moving_direction: None,
            inventory: Inventory::new(),
            simple_item: SimpleItem::new(Kind::Robbo, &[60, 61, 62, 63, 64, 65, 66, 67])
                .flags(DESTROYABLE),
            is_hidden: true,
            is_killed: false,
        }
    }
    pub fn set_direction(&mut self, direction: Direction, shot: bool) {
        if direction != (0, 0) {
            self.direction = direction;
        }
        if shot {
            self.shot_direction = Some(direction)
        } else {
            self.moving_direction = match direction {
                (0, 0) => None,
                (0, _) => Some(direction),
                (_, 0) => Some(direction),
                _ => match self.moving_direction {
                    Some((_, cur_dy)) => {
                        if cur_dy != 0 {
                            Some((direction.0, 0))
                        } else {
                            Some((0, direction.1))
                        }
                    }
                    None => Some((direction.0, 0)),
                },
            }
        }
    }
    pub fn hide(&mut self, tiles: &mut Tiles) {
        self.is_hidden = true;
        tiles.put_empty(self.get_position())
    }
    pub fn show(&mut self, tiles: &mut Tiles) {
        self.is_hidden = false;
        self.put_tile(tiles);
    }
    pub fn kill(&mut self) {
        self.is_killed = true;
    }
}
impl Item for Robbo {
    fn get_simple_item(&self) -> &SimpleItem {
        &self.simple_item
    }
    fn get_simple_item_mut(&mut self) -> &mut SimpleItem {
        &mut self.simple_item
    }
    fn get_tile(&self, frame_cnt: usize) -> usize {
        let index = direction_to_index(self.direction) * 2;
        self.simple_item.tiles[index + frame_cnt / 2 % 2]
    }
    fn tick(&mut self, tiles: &Tiles, _rng: &mut dyn rand::RngCore) -> Actions {
        if self.is_hidden {
            return Actions::empty();
        }
        if let Some(dir) = tiles.magnetic_force_dir {
            return Actions::new(&[Action::RobboMove(dir)]);
        }
        if let Some(direction) = self.shot_direction {
            if self.inventory.bullets > 0 {
                self.shot_direction = None;
                self.inventory.bullets -= 1;
                return Actions::new(&[Action::CreateBullet(direction)]);
            }
        }
        if let Some(dir) = self.moving_direction {
            return Actions::new(&[Action::RobboMove(dir)]);
        }
        return Actions::empty()
    }
}
