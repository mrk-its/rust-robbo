use consts;
use random;
use tiles::Tiles;
use types::{Action, Actions, Direction, Kind};
use utils::direction_by_index;
use super::{Item, SimpleItem};

#[derive(Debug)]
pub struct Butterfly {
    simple_item: SimpleItem,
}

impl Butterfly {
    const MOVE_PROBABILITY: f64 = 1.0;
    const RANDOM_MOVE_PROBABILITY: f64 = 0.1;

    pub fn new() -> Butterfly {
        Butterfly {
            simple_item: SimpleItem::new(Kind::Butterfly, &[32, 33])
                .flags(consts::DESTROYABLE | consts::DEADLY),
        }
    }
}

impl Item for Butterfly {
    fn get_simple_item(&self) -> &SimpleItem {
        &self.simple_item
    }
    fn get_simple_item_mut(&mut self) -> &mut SimpleItem {
        &mut self.simple_item
    }
    fn tick(&mut self, tiles: &Tiles) -> Actions {
        let neighbours = tiles.get_neighbours(self.get_position());
        if random::random() > Butterfly::MOVE_PROBABILITY {
            return Actions::empty();
        }
        if random::random() < Butterfly::RANDOM_MOVE_PROBABILITY {
            let valid_dirs = (0..4)
                .map(direction_by_index)
                .filter(|dir| neighbours.get(*dir).is_empty())
                .collect::<Vec<Direction>>();
            if !valid_dirs.is_empty() {
                return Actions::new(&[Action::RelMove(
                    valid_dirs[random::randrange(valid_dirs.len() as u32) as usize],
                )]);
            }
        }
        neighbours
            .get_robbo_dir()
            .map(|(dx, dy)| {
                if dx.abs() < dy.abs() {
                    if dx != 0 && neighbours.get((dx.signum(), 0)).is_empty() {
                        (dx.signum(), 0)
                    } else if dy != 0 && neighbours.get((0, dy.signum())).is_empty() {
                        (0, dy.signum())
                    } else {
                        (0, 0)
                    }
                } else if dy != 0 && neighbours.get((0, dy.signum())).is_empty() {
                    (0, dy.signum())
                } else if dx != 0 && neighbours.get((dx.signum(), 0)).is_empty() {
                    (dx.signum(), 0)
                } else {
                    (0, 0)
                }
            })
            .map(|dir| Actions::new(&[Action::RelMove(dir)]))
            .unwrap_or(Actions::empty())
    }
}

