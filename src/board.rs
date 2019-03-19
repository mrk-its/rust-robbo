use std::collections::HashSet;

use crate::random;

use log::log;
use consts::{COLLECTABLE, MOVEABLE, DESTROYABLE, UNDESTROYABLE, DEADLY};
use types::{Kind, Flags, Action, Actions, Direction, Position};
use utils::{direction_to_index, direction_by_index, rotate_clockwise, rotate_counter_clockwise, dest_coords};
use levels::Level;

use crate::items::{
    Item, SimpleItem, Animation, Door, Teleport, Butterfly, Bear, Bird,
    LaserHead, BlastHead, Bullet, Bomb, Capsule, Gun, GunType, Magnet, PushBox,
    ForceField,
};

#[derive(Debug)]
pub struct Inventory {
    pub keys: usize,
    pub bullets: usize,
    pub screws: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Neighbourhood {
    neighbours: [(Kind, Flags);4],
    robbo_dir: Option<Direction>,
    magnetic_force_dir: Option<Direction>,
}

impl Neighbourhood {
    pub fn get_robbo_dir(&self) -> Option<Direction> {
        self.robbo_dir
    }
    pub fn get_magnetic_force_dir(&self) -> Option<Direction> {
        self.magnetic_force_dir
    }
    pub fn get_kind(&self, direction: Direction) -> Kind {
        self.neighbours[direction_to_index(direction)].0
    }
    pub fn get_flags(&self, direction: Direction) -> Flags {
        self.neighbours[direction_to_index(direction)].1
    }
    pub fn is_empty(&self, direction: Direction) -> bool {
        self.get_kind(direction) == Kind::Empty
    }
    pub fn is_deadly(&self) -> bool {
        self.neighbours.iter().any(
            |(_, flags)| flags & DEADLY > 0
        )
    }
}

impl Inventory {
    pub fn new() -> Inventory {
        Inventory {
            keys: 0,
            bullets: 0,
            screws: 0,
        }
    }
    pub fn collect(&mut self, kind: Kind) {
        match kind {
            Kind::Bullets => self.bullets += 9,
            Kind::Key => self.keys += 1,
            Kind::Screw => self.screws += 1,
            _ => (),
        };
        self.show();
    }
    pub fn show(&self) {
        log(&format!("{:?}", self));
    }
}


pub struct Board {
    pub width: i32,
    pub height: i32,
    items: Vec<Option<Box<Item>>>,
    pub missing_screws: usize,
    pub robbo_moving_dir: Option<Direction>,
    pub robbo_shooting_dir: Option<Direction>,
    processed: HashSet<Position>,
    pub inventory: Inventory,
    pub finished: bool,
    pub missing_robbo_ticks: usize,
}

impl Board {
    pub fn from(level: &Level) -> Board {
        let mut items: Vec<Option<Box<Item>>> = Vec::new();
        let mut missing_screws = 0;
        random::seed(2);
        for (y, row) in level.tiles.iter().enumerate() {
            for (x, c) in row.chars().enumerate() {
                let additional = level.additional.get(&(x, y)).map(
                    |v| &v[..]
                );
                let tile: Option<Box<Item>> = match c {
                    'O' => Some(Box::new(SimpleItem::new(Kind::Wall, &[2]).flags(UNDESTROYABLE))),
                    'o' => Some(Box::new(SimpleItem::new(Kind::Wall, &[29]).flags(UNDESTROYABLE))),
                    '-' => Some(Box::new(SimpleItem::new(Kind::Wall, &[19]).flags(UNDESTROYABLE))),
                    'Q' => Some(Box::new(SimpleItem::new(Kind::Wall, &[3]).flags(UNDESTROYABLE))),
                    'q' => Some(Box::new(SimpleItem::new(Kind::Wall, &[21]).flags(UNDESTROYABLE))),
                    'p' => Some(Box::new(SimpleItem::new(Kind::Wall, &[68]).flags(UNDESTROYABLE))),
                    'P' => Some(Box::new(SimpleItem::new(Kind::Wall, &[69]).flags(UNDESTROYABLE))),
                    's' => Some(Box::new(SimpleItem::new(Kind::Wall, &[10]).flags(UNDESTROYABLE))),
                    'S' => Some(Box::new(SimpleItem::new(Kind::Wall, &[22]).flags(UNDESTROYABLE))),
                    'H' => Some(Box::new(SimpleItem::ground())),
                    'T' => {
                        missing_screws += 1;
                        Some(Box::new(SimpleItem::screw()))
                    },
                    '\'' => Some(Box::new(SimpleItem::bullets())),
                    '%' => Some(Box::new(SimpleItem::key())),
                    'D' => Some(Box::new(Door::new())),
                    '#' => Some(Box::new(SimpleItem::new(Kind::ABox, &[20]).flags(MOVEABLE))),
                    '&' => Some(Box::new(Teleport::new(additional.unwrap_or(&[0, 0])))),
                    'R' => Some(Box::new(Animation::spawn_robbo())),
                    '!' => Some(Box::new(Capsule::new())),
                    '~' => Some(Box::new(PushBox::new())),
                    'b' => Some(Box::new(Bomb::new())),
                    '?' => Some(Box::new(SimpleItem::questionmark())),
                    'V' => Some(Box::new(Butterfly::new())),
                    '@' => Some(Box::new(Bear::new(Kind::Bear, additional.unwrap_or(&[0]), &[13, 14]))),
                    '*' => Some(Box::new(Bear::new(Kind::BlackBear, additional.unwrap_or(&[0]), &[30, 31]))),
                    '^' => Some(Box::new(Bird::new(additional.unwrap_or(&[0, 0, 0])))),
                    '}' => Some(Box::new(Gun::new(additional.unwrap_or(&[0, 0, 0, 0, 0, 0])))),
                    'L' => Some(Box::new(SimpleItem::new(Kind::HorizontalLaser, &[53]))),
                    'l' => Some(Box::new(SimpleItem::new(Kind::VerticalLaser, &[53]))),
                    'M' => Some(Box::new(Magnet::new(additional.unwrap_or(&[0])))),
                    '=' => Some(Box::new(ForceField::new(additional.unwrap_or(&[0])))),
                    _ => None,
                };
                items.push(tile);
            }
        };
        let mut board = Board{
            width: level.width,
            height: level.height,
            items,
            missing_screws,
            robbo_moving_dir: None,
            robbo_shooting_dir: None,
            processed: HashSet::new(),
            inventory: Inventory::new(),
            finished: false,
            missing_robbo_ticks: 0,
        };
        if missing_screws == 0 {
            board.repair_capsule()
        }
        board
    }

    pub fn get_neighbourhood(&self, (x, y): Position, robbo_pos: Option<Position>) -> Neighbourhood {
        let up = self.get_kind_and_flags((x, y-1));
        let down = self.get_kind_and_flags((x, y+1));
        let left = self.get_kind_and_flags((x-1, y));
        let right = self.get_kind_and_flags((x+1, y));

        let magnetic_force_dir = robbo_pos.map_or(None, |robbo_pos| {
            (0..4).map(direction_by_index).map(
                |dir| self.get_magnetic_force_dir(robbo_pos, dir)
            ).find(|dir| dir.is_some()).unwrap_or(None)
        });

        let robbo_dir = robbo_pos.map(
            |(robbo_x, robbo_y)| ((robbo_x as i32) - (x as i32), (robbo_y as i32) - (y as i32))
        );

        Neighbourhood {
            neighbours: [right, down, left, up],
            robbo_dir,
            magnetic_force_dir,
        }
    }

    fn get_magnetic_force_dir(&self, robbo_pos: Position, dir: Direction) -> Option<Direction> {
        let mut pos = dest_coords(robbo_pos, dir);
        while self.is_empty(pos) {
            pos = dest_coords(pos, dir);
        };
        self.get_item(pos).as_ref().map_or(None, |item| item.as_magnet()).filter(
            |magnet| magnet.get_magnetic_force_dir() == dir
        ).map(|_magnet| dir)
    }

    pub fn is_pos_valid(&self, (x, y): Position) -> bool {
        x >= 0 && x < self.width && y >=0 && y < self.height
    }

    pub fn get_item(&self, (x, y): Position) -> &Option<Box<Item>> {
        if self.is_pos_valid((x, y)) {
            &self.items[(y * self.width + x) as usize]
        } else {
            &None
        }
    }

    pub fn get_mut_item(&mut self, (x, y): Position) -> Option<&mut Box<Item>> {
        self.items[(y * self.width + x) as usize].as_mut()
    }

    pub fn get_kind(&self, pos: Position) -> Kind {
        if !self.is_pos_valid(pos) {
            return Kind::Wall
        }
        self.get_item(pos).as_ref().map_or(Kind::Empty, |item| item.get_kind())
    }

    pub fn get_kind_and_flags(&self, pos: Position) -> (Kind, Flags) {
        if !self.is_pos_valid(pos) {
            return (Kind::Wall, 0)
        }
        self.get_item(pos).as_ref().map_or((Kind::Empty, 0), |item| (item.get_kind(), item.get_flags()))
    }

    pub fn is_empty(&self, (x, y): Position) -> bool {
        self.is_pos_valid((x, y)) && self.get_item((x, y)).is_none()
    }

    pub fn get_tile(&self, pos: Position, frame_cnt: usize) -> usize {
        self.get_item(pos).as_ref().map_or(95, |item| item.get_tile(frame_cnt))
    }

    pub fn swap(&mut self, (x, y): Position, (dx, dy): Position) {
        self.items.swap((y * self.width + x) as usize, (dy * self.width + dx) as usize);
    }

    pub fn replace(&mut self, (x, y): Position, item: Option<Box<Item>>) {
        self.items[(y * self.width + x) as usize] = item;
    }

    pub fn remove(&mut self, pos: Position) {
        self.replace(pos, None)
    }

    pub fn god_mode(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let pos = (x, y);
                let kind = self.get_kind(pos);
                if kind == Kind::Butterfly || kind == Kind::Bear || kind == Kind::BlackBear || kind == Kind::Bird{
                    self.destroy(pos, true);
                } else if kind == Kind::Gun {
                    match self.get_mut_item(pos) {
                        Some(item) => item.as_gun().unwrap().disabled = !item.as_gun().unwrap().disabled,
                        None => (),
                    }
                }

            }
        }
    }
    pub fn dispatch_actions(&mut self, actions: Actions, pos: Position) {
        match actions {
            Some(actions) => {
                for action in actions {
                    match action {
                        Action::RelMove(direction) => {
                            self.move_if_empty(pos, direction);
                        },
                        Action::LaserHeadMove(direction) => {
                            self.laser_move(pos, direction);
                        },
                        Action::BlastHeadMove(direction) => {
                            self.blast_move(pos, direction);
                        },
                        Action::BlastHeadDestroy => {
                            self.replace(pos, Some(Box::new(Animation::blast_tail())))
                        },
                        Action::AutoRemove => {
                            self.remove(pos);
                        },
                        Action::NextLevel => {
                            self.finished = true;
                        },
                        Action::DestroyBullet => {
                            self.replace(pos, Some(Box::new(Animation::small_explosion())));
                        },
                        Action::RelImpact(direction, force) => {
                            let dest = dest_coords(pos, direction);
                            self.destroy(dest, force);
                            self.mark_as_processed(dest);
                        },
                        Action::CreateBullet(direction) => {
                            self.shot(pos, direction);
                        },
                        Action::CreateLaser(direction) => {
                            self.laser_shot(pos, direction);
                        },
                        Action::CreateBlast(direction) => {
                            self.blaster_shot(pos, direction);
                        },
                        Action::SpawnRobbo => {
                            self.replace(pos, Some(Box::new(SimpleItem::new(Kind::Robbo, &[60, 61, 62, 63, 64, 65, 66, 67]).flags(DESTROYABLE))))
                        },
                        Action::SpawnRandomItem => {
                            // empty field, push box, screw, bullet, key, bomb, ground, butterfly, gun or another questionmark
                            let item:Box<Item> = match random::randrange(10) {
                                1 => Box::new(PushBox::new()),
                                2 => Box::new(SimpleItem::screw()),
                                3 => Box::new(SimpleItem::bullets()),
                                4 => Box::new(SimpleItem::key()),
                                5 => Box::new(Bomb::new()),
                                6 => Box::new(SimpleItem::ground()),
                                7 => Box::new(Butterfly::new()),
                                8 => Box::new(Gun::new(&[0, 0, 0, 0, 0, 1])),
                                9 => Box::new(SimpleItem::questionmark()),
                                _ => Box::new(Animation::small_explosion()),
                            };
                            self.replace(pos, Some(item))
                        },
                        Action::TeleportRobbo(group, position_in_group, direction) => {
                            self.teleport_robbo(group, position_in_group, direction);
                        },
                    }
                }
            },
            None => (),
        }
    }

    pub fn teleport_robbo(&mut self, group: u16, position_in_group: u16, direction: Direction) {
        let dest_teleport_pos = {
            let teleports = self.items.iter().enumerate().map(|(i, item)| match (i, item) {
                (_, Some(it)) => (i, it.as_teleport()),
                _ => (i, None)
            }).filter(|(_, v)| {
                v.is_some() && v.unwrap().group == group
            }).collect::<Vec<(usize, Option<&Teleport>)>>();

            teleports.iter().find(
                |(_, v)| ((v.unwrap().position_in_group as usize % teleports.len()) == ((position_in_group + 1) as usize) % teleports.len())
            ).map(
                |(i, v)| ((*i as i32 % self.width, *i as i32 / self.width), v.unwrap())
            ).map(|(pos, _)| pos)
        };
        let robbo_pos = self.find_robbo();

        match (dest_teleport_pos, robbo_pos) {
            (Some(teleport_pos), Some(robbo_pos)) => {
                let mut dir = direction;
                let cc = dir.0 != 0; // hack for level 16
                for _ in 0..4 {
                    let dest_robbo_pos = dest_coords(teleport_pos, dir);
                    if self.is_empty(dest_robbo_pos) {
                        self.destroy(robbo_pos, true);
                        self.replace(dest_robbo_pos, Some(Box::new(Animation::teleport_robbo())));
                        break;
                    }

                    dir = if cc {rotate_counter_clockwise(dir)} else {rotate_clockwise(dir)}
                }

            },
            _ => {}
        }
    }

    pub fn tick(&mut self) {
        self.processed = HashSet::new();
        let robbo_pos = self.find_robbo();
        match robbo_pos {
            Some(pos) => {
                let neighbours = self.get_neighbourhood(pos, robbo_pos);
                match neighbours.get_magnetic_force_dir() {
                    Some(dir) => {
                        self.robbo_move_or_shot(pos, dir, false);
                        if self.get_kind(dest_coords(pos, dir)) == Kind::Magnet {
                            self.destroy(pos, false)
                        }
                    },
                    None => {
                        if neighbours.is_deadly() {
                            self.destroy(pos, false);
                        } else {
                            let actions = match (self.robbo_shooting_dir, self.robbo_moving_dir) {
                                (Some(direction), _) => {
                                    self.robbo_shooting_dir = None;
                                    self.robbo_move_or_shot(pos, direction, true)
                                },
                                (_, Some(direction)) => self.robbo_move_or_shot(pos, direction, false),
                                _ => None,
                            };
                            self.dispatch_actions(actions, pos);
                        }

                    }
                }
                self.missing_robbo_ticks = 0;
            },
            None => {
                self.missing_robbo_ticks += 1;
                if self.missing_robbo_ticks == 8 {
                    self.explode_all();
                }
            },
        }
        for y in 0..self.height {
            let mut skip_to = 0;
            for x in 0..self.width {
                if x < skip_to {
                    continue;
                }
                let pos = (x, y);
                if self.is_processed(pos) {
                    continue;
                }
                if self.get_kind((x, y)) == Kind::ForceField {
                    skip_to = self.process_force_field(pos);
                }
                let neighbours = self.get_neighbourhood(pos, robbo_pos);
                let actions = {
                    match self.get_mut_item(pos) {
                        Some(it) => {
                            it.tick(&neighbours)
                        },
                        None => None,
                    }
                };
                self.dispatch_actions(actions, pos);
            }
        }
    }

    pub fn process_force_field(&mut self, (x, y): Position) -> i32 {
        let mut wall_x1 = x;
        let mut wall_x2 = x;
        while self.get_kind((wall_x1 - 1, y)) != Kind::Wall {
            wall_x1 -= 1;
        }
        while self.get_kind((wall_x2, y)) != Kind::Wall {
            wall_x2 += 1;
        }
        let ff_dir = self.get_item((x, y)).as_ref().unwrap().as_force_field().unwrap().direction;
        let (mut x, end_x, step) = if ff_dir == 0 {
            (wall_x1, wall_x2, 1)
        } else {
            (wall_x2-1, wall_x1-1, -1)
        };

        let tmp = if self.get_kind((x, y)) == Kind::ForceField {
            self.items[(y * self.width + x) as usize].take()
        } else {
            None
        };
        x += step;

        while x != end_x {
            if self.get_kind((x, y)) == Kind::ForceField {
                self.swap((x - step, y), (x, y));
                self.remove((x, y));
            }
            x += step;
        }

        if tmp.is_some() {
            self.items[(y * self.width + x - step) as usize].replace(tmp.unwrap());
        }
        wall_x2
    }

    pub fn find_robbo(&self) -> Option<Position> {
        self.items.iter().enumerate().find(
            |(_, v)| match v {
                Some(item)=>item.get_kind() == Kind::Robbo,
                None => false,
        }).map(|(i, _)|(i as i32 % self.width, i as i32 / self.width))
    }

    pub fn destroy(&mut self, pos: Position, force: bool) {
        let (x, y) = pos;
        if x<0 || y< 0 || x>=self.width || y>=self.height {
            return
        }
        let (is_destroyable, is_bomb_destroyable, is_question_mark) = {
            match self.get_mut_item(pos) {
                Some(it) => {
                    if it.destroy() {
                        return;
                    }
                    (it.is_destroyable(), !it.is_undestroyable(), it.get_kind() == Kind::Questionmark)
                },
                None => (false, true, false)
            }
        };
        if is_destroyable || force && is_bomb_destroyable {
            let animation = if is_question_mark {
                Animation::question_mark_explosion()
            } else {
                Animation::small_explosion()
            };
            self.replace(pos, Some(Box::new(animation)));
        }
    }

    pub fn mark_as_processed(&mut self, pos: Position) {
        self.processed.insert(pos);
    }

    pub fn is_processed(&self, pos: Position) -> bool {
        self.processed.contains(&pos)
    }

    pub fn move_if_empty(&mut self, pos: Position, direction: Direction) -> bool {
        let dest_pos = dest_coords(pos, direction);
        let is_dst_empty = self.is_empty(dest_pos);
        if is_dst_empty {
            self.swap(pos, dest_pos);
            {
                let item = self.get_mut_item(dest_pos);
                match item {
                    Some(item) => {
                        item.moved(direction);
                    },
                    None => (),
                }
            }
            self.mark_as_processed(dest_pos);
        }
        is_dst_empty
    }

    pub fn laser_move(&mut self, pos: Position, direction: Direction) {
        let dest_pos = dest_coords(pos, direction);
        if self.is_empty(dest_pos) {
            self.swap(pos, dest_pos);
            self.replace(pos, Some(Box::new(SimpleItem::laser_tail(direction))));
            self.mark_as_processed(dest_pos);
        } else if self.get_kind(dest_pos) == Kind::LaserTail {
            self.swap(pos, dest_pos);
            self.replace(pos, None);
            self.mark_as_processed(dest_pos);
        }
    }
    pub fn blast_move(&mut self, pos: Position, direction: Direction) {
        let dest_pos = dest_coords(pos, direction);
        self.swap(pos, dest_pos);
        self.replace(pos, Some(Box::new(Animation::blast_tail())));
        self.mark_as_processed(dest_pos);
        self.mark_as_processed(pos);
    }

    pub fn repair_capsule(&mut self) {
        for item in &mut self.items {
            match item {
                Some(it) => {
                    match it.as_capsule() {
                        Some(capsule) => capsule.repair(),
                        None => (),
                    }
                },
                None => (),
            }
        }
    }

    pub fn robbo_move(&mut self, pos: Position, direction: Direction) -> Actions {
        let dest_pos = dest_coords(pos, direction);
        if !self.move_if_empty(pos, direction) && self.is_pos_valid(dest_pos) {
            let (kind, is_collectable, is_moveable) = {
                let dst = self.get_item(dest_pos);
                match dst {
                    Some(item) => (item.get_kind(), item.is_collectable(), item.is_moveable()),
                    None => (Kind::Empty, false, false),
                }
            };
            if is_collectable {
                self.inventory.collect(kind);
                self.swap(pos, dest_pos);
                self.remove(pos);
                if self.inventory.screws == self.missing_screws {
                    self.repair_capsule();
                }
            } else if is_moveable {
                if self.move_if_empty(dest_pos, direction) {
                    self.move_if_empty(pos, direction);
                }
            } else {
                let (x, y) = dest_pos;
                let item = &mut self.items[(y * self.width + x) as usize];
                match item {
                    Some(it) => return it.enter(&mut self.inventory, direction),
                    None => (),
                }
            }
        };
        None
    }

    pub fn _shot(&mut self, pos: Position, direction: Direction, gun_type: GunType) -> bool {
        let dest = dest_coords(pos, direction);
        let is_dst_empty = self.is_empty(dest);
        if is_dst_empty {
            let bullet:Option<Box<Item>> = match gun_type {
                GunType::Burst => Some(Box::new(Bullet::new(direction))),
                GunType::Solid => Some(Box::new(LaserHead::new(direction))),
                GunType::Blaster => Some(Box::new(BlastHead::new(direction))),
            };
            self.replace(dest, bullet);
            self.mark_as_processed(dest);
        } else {
            self.destroy(dest, false);
        }
        is_dst_empty
    }

    pub fn shot(&mut self, pos: Position, direction: Direction) -> bool {
        self._shot(pos, direction, GunType::Burst)
    }

    pub fn laser_shot(&mut self, pos: Position, direction: Direction) -> bool {
        self._shot(pos, direction, GunType::Solid)
    }

    pub fn blaster_shot(&mut self, pos: Position, direction: Direction) -> bool {
        self._shot(pos, direction, GunType::Blaster)
    }

    pub fn robbo_move_or_shot(&mut self, _pos: Position, direction: Direction, shot: bool) -> Actions {
        let robbo_pos = self.find_robbo();

        match robbo_pos {
            Some(pos) => {
                if shot {
                    if self.inventory.bullets > 0 {
                        self.inventory.bullets -= 1;
                        self.inventory.show();
                        self.shot(pos, direction);
                        None
                    } else {
                        None
                    }
                } else {
                    self.robbo_move(pos, direction)
                }
            }
            None => None
        }
    }

    pub fn kill_robbo(&mut self) {
        self.find_robbo().map(|pos|{
            self.replace(pos, Some(Box::new(Animation::small_explosion())))
        });
    }

    pub fn is_robbo_killed(&self) -> bool {
        return self.missing_robbo_ticks >= 16
    }

    pub fn explode_all(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let pos = (x, y);
                let kind = self.get_kind(pos);
                if kind != Kind::Empty && kind != Kind::Wall {
                    self.replace(pos, Some(Box::new(Animation::small_explosion())));
                }
            }
        }
    }
    pub fn robbo_shot_event(&mut self, dir: Direction) {
        self.robbo_shooting_dir = Some(dir)
    }

    pub fn robbo_move_event(&mut self, dir: Direction) {
        self.robbo_moving_dir = match dir {
            (0, 0) => None,
            (0, _) => Some(dir),
            (_, 0) => Some(dir),
            _ => {
                match self.robbo_moving_dir {
                    Some((_, cur_dy)) => {
                        if cur_dy != 0 {
                            Some((dir.0, 0))
                        } else {
                            Some((0, dir.1))
                        }
                    },
                    None => Some((dir.0, 0))
                }
            },
        }
    }
}
