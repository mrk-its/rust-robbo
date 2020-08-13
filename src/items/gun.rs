use consts;
use random;
use tiles::Tiles;
use types::{Action, Actions, Direction, Kind};
use utils::{direction_by_index, direction_to_index, rotate_clockwise};
use super::{Item, SimpleItem};

#[repr(u16)]
#[derive(Debug)]
pub enum GunType {
    Burst = 0,
    Solid = 1,
    Blaster = 2,
}

#[derive(Debug)]
pub struct Gun {
    simple_item: SimpleItem,
    shooting_dir: Direction,
    moving_dir: Direction,
    gun_type: GunType,
    is_moveable: bool,
    is_rotateable: bool,
    is_random_rotatable: bool,
    pub disabled: bool,
}
impl Gun {
    const SHOOTING_PROPABILITY: f64 = 0.075;
    const ROTATE_PROBABILITY: f64 = 0.25;

    pub fn new(params: &[u16]) -> Gun {
        let is_moveable = params[3] > 0;
        Gun {
            simple_item: SimpleItem::new(Kind::Gun, &[53, 54, 55, 56]).flags(if is_moveable {
                consts::MOVEABLE
            } else {
                0
            }),
            shooting_dir: direction_by_index(params[0] as usize),
            moving_dir: direction_by_index(params[1] as usize),
            gun_type: match params[2] {
                1 => GunType::Solid,
                2 => GunType::Blaster,
                _ => GunType::Burst,
            },
            is_moveable,
            is_rotateable: *params.get(4).unwrap_or(&0) > 0,
            is_random_rotatable: *params.get(5).unwrap_or(&0) > 0,
            disabled: false,
        }
    }
}
impl Item for Gun {
    fn get_simple_item(&self) -> &SimpleItem {
        &self.simple_item
    }
    fn get_simple_item_mut(&mut self) -> &mut SimpleItem {
        &mut self.simple_item
    }
    fn get_tile(&self, _frame_cnt: usize) -> usize {
        self.simple_item.tiles[direction_to_index(self.shooting_dir)]
    }
    fn tick(&mut self, tiles: &Tiles) -> Actions {
        let mut actions: Vec<Action> = Vec::new();
        let neighbours = tiles.get_neighbours(self.get_position());
        if self.is_moveable {
            if neighbours.get(self.moving_dir).is_empty() {
                actions.push(Action::RelMove(self.moving_dir))
            } else {
                let (dx, dy) = self.moving_dir;
                self.moving_dir = (-dx, -dy);
            }
        }
        if (self.is_random_rotatable || self.is_rotateable)
            && random::random() < Gun::ROTATE_PROBABILITY
        {
            if self.is_random_rotatable {
                self.shooting_dir = direction_by_index(random::randrange(4) as usize);
            } else {
                self.shooting_dir = rotate_clockwise(self.shooting_dir);
            }
        }
        if !self.disabled && random::random() < Gun::SHOOTING_PROPABILITY {
            actions.push(match self.gun_type {
                GunType::Solid => Action::CreateLaser(self.shooting_dir),
                GunType::Blaster => Action::CreateBlast(self.shooting_dir),
                _ => Action::CreateBullet(self.shooting_dir),
            });
        }
        Actions::new(&actions)
    }
    fn as_gun(&mut self) -> Option<&mut Gun> {
        Some(self)
    }
}

