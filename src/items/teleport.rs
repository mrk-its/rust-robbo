use super::{Animation, Item, Robbo, SimpleItem};
use crate::board::Board;
use crate::types::{Action, Actions, Direction, Kind, Position};
use sound::Sound;
use utils::{dest_coords, rotate_clockwise, rotate_counter_clockwise};

#[derive(Debug)]
pub struct Teleport {
    simple_item: SimpleItem,
    pub group: u16,
    pub position_in_group: u16,
}

impl Teleport {
    pub fn new(params: &[u16]) -> Teleport {
        Teleport {
            simple_item: SimpleItem::new(Kind::Teleport, &[48, 49]),
            group: params[0],
            position_in_group: params[1],
        }
    }
    pub fn teleport_robbo(
        board: &mut Board,
        group: u16,
        position_in_group: u16,
        direction: Direction,
    ) {
        let dest_teleport_positions = {
            let mut teleports = {
                board
                    .items
                    .get_items(Kind::Teleport)
                    .iter()
                    .map(|item| item.as_teleport().unwrap())
                    .filter(|v| v.group == group)
                    .collect::<Vec<&Teleport>>()
            };
            teleports.sort_by_key(|t| t.position_in_group);
            let len = teleports.len();
            let index = teleports
                .iter()
                .enumerate()
                .find(|(_, v)| {
                    (v.position_in_group as usize % len) == (position_in_group as usize % len)
                })
                .map(|(i, _)| i)
                .unwrap();
            teleports.rotate_left(index + 1);
            teleports
                .iter()
                .map(|t| t.get_position())
                .collect::<Vec<Position>>()
        };

        for teleport_pos in dest_teleport_positions {
            let mut dir = direction;
            let cc = dir.0 != 0; // hack for level 16
            for _ in 0..4 {
                let dest_robbo_pos = dest_coords(teleport_pos, dir);
                if board
                    .tiles
                    .get(dest_robbo_pos)
                    .map(|v| v.is_empty())
                    .unwrap_or(false)
                {
                    board.robbo.hide(&mut board.tiles);
                    board.robbo.set_position(dest_robbo_pos);
                    board.play_sound(Sound::Teleport);
                    board.add_item(dest_robbo_pos, Box::new(Animation::teleport_robbo()));
                    return;
                }
                dir = if cc {
                    rotate_counter_clockwise(dir)
                } else {
                    rotate_clockwise(dir)
                }
            }
        }
    }
}

impl Item for Teleport {
    fn get_simple_item(&self) -> &SimpleItem {
        &self.simple_item
    }
    fn get_simple_item_mut(&mut self) -> &mut SimpleItem {
        &mut self.simple_item
    }
    fn enter(&mut self, _robbo: &mut Robbo, direction: Direction) -> Actions {
        Actions::new(&[Action::TeleportRobbo(
            self.group,
            self.position_in_group,
            direction,
        )])
    }
    fn as_teleport(&self) -> Option<&Teleport> {
        Some(self)
    }
}
