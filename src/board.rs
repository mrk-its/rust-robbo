use std::collections::HashSet;

use crate::items::{
    Animation, Bear, Bird, BlastHead, Bomb, Bullet, Butterfly, Capsule, Door, ForceField, Gun,
    GunType, Item, LaserHead, Magnet, PushBox, Robbo, SimpleItem, Teleport,
};
use crate::random;
use levels::Level;
use log;
use sound::{Sound, Sounds};
use std::collections::HashMap;
use tiles::{Tile, Tiles};
use types::{Action, Actions, Direction, Kind, Position};
use utils::{dest_coords, direction_by_index};
type Processed = HashSet<Position>;

pub struct Items {
    items: HashMap<Position, Box<dyn Item>>,
    processed: Processed,
}

impl Items {
    pub fn new(items: Vec<Box<dyn Item>>) -> Items {
        let mut hashmap = HashMap::new();
        for item in items {
            hashmap.insert(item.get_position(), item);
        }
        Items {
            items: hashmap,
            processed: HashSet::new(),
        }
    }
    fn init(&mut self) {
        self.processed = HashSet::new();
    }
    pub fn get_items(&self, kind: Kind) -> Vec<&Box<dyn Item>> {
        self.items
            .values()
            .filter(|v| v.get_kind() == kind)
            .collect()
    }
    fn mut_item_at(&mut self, pos: Position) -> Option<&mut Box<dyn Item>> {
        self.items.get_mut(&pos)
    }
    pub fn item_at(&self, pos: Position) -> Option<&Box<dyn Item>> {
        self.items.get(&pos)
    }
    fn iter_mut(
        &mut self,
    ) -> std::collections::hash_map::ValuesMut<'_, (i32, i32), Box<(dyn Item)>> {
        self.items.values_mut()
    }
    fn item_indices_to_process(&self) -> Vec<Position> {
        let mut keys: Vec<Position> = self.items.keys().cloned().collect();
        keys.sort();
        keys
    }
    fn is_processed(&self, pos: Position) -> bool {
        self.processed.contains(&pos)
    }
    pub fn push(&mut self, item: Box<dyn Item>) {
        assert!(
            !self.items.contains_key(&item.get_position()),
            "item: {:?} already contains: {:?}",
            item, self.items.get(&item.get_position())
        );
        let pos = item.get_position();
        self.items.insert(pos, item);
        self.processed.insert(pos);
    }
    fn get_mut(&mut self, pos: Position) -> Option<&mut Box<dyn Item>> {
        self.items.get_mut(&pos)
    }
    fn get(&mut self, pos: Position) -> Option<&Box<dyn Item>> {
        self.items.get(&pos)
    }
    pub fn remove(&mut self, pos: Position) -> Option<Box<dyn Item>> {
        self.items.remove(&pos)
    }
}

pub struct Board {
    pub width: i32,
    pub height: i32,
    pub items: Items,
    pub tiles: Tiles,
    pub robbo: Robbo,
    pub missing_screws: usize,
    pub missing_robbo_ticks: usize,
    pub robbo_moving_dir: Option<Direction>,
    pub robbo_shooting_dir: Option<Direction>,
    pub finished: bool,
    sounds: Sounds,
}

impl Board {
    pub fn from(level: &Level) -> Board {
        let mut items = Vec::new();
        let mut tiles: Tiles = Tiles::new(level.width, level.height);
        let mut missing_screws = 0;
        let mut robbo = Robbo::new();
        random::seed(2);
        for (y, row) in level.tiles.iter().enumerate() {
            for (x, c) in row.chars().enumerate() {
                let pos: Position = (x as i32, y as i32);
                let additional = level.additional.get(&(x, y)).map(|v| &v[..]);

                let tile_map: HashMap<char, Tile> = [
                    ('O', Tile::wall(2)),
                    ('o', Tile::wall(29)),
                    ('-', Tile::wall(19)),
                    ('Q', Tile::wall(3)),
                    ('q', Tile::wall(21)),
                    ('p', Tile::wall(68)),
                    ('P', Tile::wall(69)),
                    ('s', Tile::wall(10)),
                    ('S', Tile::wall(22)),
                    ('H', Tile::ground()),
                    ('T', Tile::screw()),
                    ('\'', Tile::ammo()),
                    ('%', Tile::key()),
                ]
                .iter()
                .cloned()
                .collect();
                let tile = tile_map.get(&c);
                if let Some(tile) = tile {
                    if tile.get_kind() == Kind::Screw {
                        missing_screws += 1
                    }
                    tiles.put(pos, tile.clone());
                    continue;
                }
                let mut item: Box<dyn Item> = match c {
                    'D' => Box::new(Door::new()),
                    '#' => Box::new(SimpleItem::abox()),
                    '&' => Box::new(Teleport::new(additional.unwrap_or(&[0, 0]))),
                    'R' => {
                        robbo.set_position(pos);
                        Box::new(Animation::spawn_robbo())
                    }
                    '!' => Box::new(Capsule::new()),
                    '~' => Box::new(PushBox::new()),
                    'b' => Box::new(Bomb::new()),
                    '?' => Box::new(SimpleItem::questionmark()),
                    'V' => Box::new(Butterfly::new()),
                    '@' => Box::new(Bear::new(Kind::Bear, additional.unwrap_or(&[0]), &[13, 14])),
                    '*' => Box::new(Bear::new(
                        Kind::BlackBear,
                        additional.unwrap_or(&[0]),
                        &[30, 31],
                    )),
                    '^' => Box::new(Bird::new(additional.unwrap_or(&[0, 0, 0]))),
                    '}' => Box::new(Gun::new(additional.unwrap_or(&[0, 0, 0, 0, 0, 0]))),
                    'L' => Box::new(SimpleItem::horizontal_laser()),
                    'l' => Box::new(SimpleItem::vertical_laser()),
                    'M' => Box::new(Magnet::new(additional.unwrap_or(&[0]))),
                    '=' => Box::new(ForceField::new(additional.unwrap_or(&[0]))),
                    _ => continue,
                };
                item.set_position(pos);
                item.put_tile(&mut tiles);
                items.push(item);
            }
        }
        Board {
            width: level.width,
            height: level.height,
            items: Items::new(items),
            robbo: robbo,
            tiles: tiles,
            missing_screws,
            robbo_moving_dir: None,
            robbo_shooting_dir: None,
            finished: false,
            sounds: Sounds::new(),
            missing_robbo_ticks: 0,
        }
    }

    fn get_magnetic_force_dir(&self, robbo_pos: Position, dir: Direction) -> Option<Direction> {
        let mut pos = dest_coords(robbo_pos, dir);
        while self.tiles.is_empty(pos) {
            pos = dest_coords(pos, dir);
        }
        self.items
            .item_at(pos)
            .and_then(|item| item.as_magnet())
            .filter(|magnet| magnet.get_magnetic_force_dir() == dir)
            .map(|_magnet| dir)
    }

    pub fn get_tile(&self, pos: Position) -> usize {
        self.tiles.get(pos).map(|x| x.get_tile()).unwrap_or(0)
    }

    pub fn god_mode2(&mut self) {
        self.robbo.inventory.bullets = 99999;
        self.play_sound(Sound::Bomb)
    }

    // pub fn god_mode(&mut self) {
    //     for y in 0..self.height {
    //         for x in 0..self.width {
    //             let pos = (x, y);
    //             let kind = self.get_kind(pos);
    //             match kind {
    //                 Kind::Butterfly | Kind::Bear | Kind::BlackBear | Kind::Bird => {
    //                     self.destroy(pos, true)
    //                 }
    //                 Kind::Gun => {
    //                     if let Some(item) = self.get_mut_item(pos) {
    //                         item.as_gun().unwrap().disabled = !item.as_gun().unwrap().disabled
    //                     }
    //                 }
    //                 Kind::Capsule => {
    //                     if let Some(item) = self.get_mut_item(pos) {
    //                         item.as_capsule().unwrap().repair()
    //                     }
    //                 }
    //                 _ => (),
    //             }
    //         }
    //     }
    //     self.play_sound(Sound::Bomb)
    // }

    pub fn remove_at(&mut self, pos: Position) -> Option<Box<dyn Item>> {
        if pos == self.robbo.get_position() {
            self.robbo.hide(&mut self.tiles);
            // return None;
        }
        let item = self.items.remove(pos);
        self.tiles.put_empty(pos);
        item
    }

    pub fn mv(&mut self, pos: Position, dir: Direction) -> Option<Position> {
        let x = self.items.remove(pos);
        if x.is_some() {
            let mut item = x.unwrap();
            item._mv(dir, &mut self.tiles);
            let pos = item.get_position();
            self.items.push(item);
            Some(pos)
        } else {
            None
        }
    }

    pub fn add_item(&mut self, pos: Position, mut item: Box<dyn Item>) {
        self.remove_at(pos);
        item.set_position(pos);
        item.put_tile(&mut self.tiles);
        self.items.push(item);
    }

    pub fn destroy(&mut self, pos: Position, force: bool) {
        let tile = self.tiles.get_or_wall(pos);
        if tile.get_kind() == Kind::Robbo {
            self.robbo.hide(&mut self.tiles);
            self.add_item(pos, Box::new(Animation::kill_robbo()));
            return;
        }
        let is_bomb_destroyable = !tile.is_undestroyable();
        if tile.is_destroyable() || force && is_bomb_destroyable {
            if tile.get_kind() == Kind::Bomb {
                if let Some(item) = self.items.mut_item_at(pos) {
                    item.destroy();
                }
            } else {
                let animation = if tile.get_kind() == Kind::Questionmark {
                    Animation::question_mark_explosion()
                } else {
                    Animation::small_explosion()
                };
                self.add_item(pos, Box::new(animation));
            }
        }
    }

    pub fn dispatch_actions(&mut self, mut actions: Actions, pos: Position) {
        while let Some(action) = actions.pop_first() {
            match action {
                Action::PlaySound(sound) => self.play_sound(sound),
                Action::RelMove(direction) => {
                    self.mv(pos, direction);
                }
                Action::ForceRelMove(dir) => {
                    let dest_pos = dest_coords(pos, dir);
                    self.remove_at(dest_pos);
                    self.mv(pos, dir);
                }
                Action::RobboMove(dir) => {
                    let dst = dest_coords(pos, dir);
                    if let Some(dst_tile) = self.tiles.get(dst) {
                        if dst_tile.is_collectable() {
                            actions.extend(&self.robbo.inventory.collect(dst_tile.get_kind()));
                            self.remove_at(dst);
                        }
                    }
                    if let Some(dst_tile) = self.tiles.get(dst) {
                        if dst_tile.is_empty() {
                            if self.robbo._mv(dir, &mut self.tiles) {
                                self.play_sound(Sound::Walk);
                            };
                        } else if dst_tile.is_moveable() {
                            if let Some(dest_pos) = self.mv(dst, dir) {
                                let item = self.items.get_mut(dest_pos).unwrap();
                                item.pushed(dir);
                                if self.robbo._mv(dir, &mut self.tiles) {
                                    self.play_sound(Sound::Walk);
                                }
                            };
                        } else {
                            if let Some(item) = self.items.mut_item_at(dst) {
                                actions.extend(&item.enter(&mut self.robbo, dir))
                            }
                        }
                    }
                }
                Action::AutoRemove => {
                    self.remove_at(pos);
                }
                Action::NextLevel => {
                    self.finished = true;
                    self.play_sound(Sound::Capsule);
                }
                Action::RelImpact(direction, force) => {
                    let dest = dest_coords(pos, direction);
                    self.destroy(dest, force);
                }
                Action::CreateBullet(direction) => self._shot(pos, direction, GunType::Burst),
                Action::CreateLaser(direction) => self._shot(pos, direction, GunType::Solid),
                Action::CreateBlast(direction) => self._shot(pos, direction, GunType::Blaster),
                Action::CreateLaserTail(pos, dir) => {
                    self.add_item(pos, Box::new(SimpleItem::laser_tail(dir)))
                }
                Action::CreateBlastTail(pos, _dir) => {
                    self.add_item(pos, Box::new(Animation::blast_tail()))
                }
                Action::SmallExplosion => {
                    self.add_item(pos, Box::new(Animation::small_explosion()));
                    self.play_sound(Sound::GunShot);
                }
                Action::SpawnRobbo(initial) => {
                    self.robbo.show(&mut self.tiles);
                    if initial {
                        self.play_sound(Sound::Spawn);
                    }
                }
                Action::KillRobbo => self.kill_robbo(),
                Action::ExplodeAll => self.robbo.kill(),
                Action::SpawnRandomItem => {
                    // empty field, push box, screw, bullet, key, bomb, ground, butterfly, gun or another questionmark
                    match random::randrange(10) {
                        1 => self.tiles.put(pos, Tile::screw()),
                        2 => self.tiles.put(pos, Tile::ammo()),
                        3 => self.tiles.put(pos, Tile::key()),
                        4 => self.tiles.put(pos, Tile::ground()),
                        5 => self.add_item(pos, Box::new(PushBox::new())),
                        6 => self.add_item(pos, Box::new(Bomb::new())),
                        7 => self.add_item(pos, Box::new(Butterfly::new())),
                        8 => self.add_item(pos, Box::new(Gun::new(&[0, 0, 0, 0, 0, 1]))),
                        9 => self.add_item(pos, Box::new(SimpleItem::questionmark())),
                        _ => self.add_item(pos, Box::new(Animation::small_explosion())),
                    };
                }
                Action::TeleportRobbo(group, position_in_group, direction) => {
                    Teleport::teleport_robbo(self, group, position_in_group, direction)
                }
                Action::ForceField => {
                    ForceField::process_force_field(self, pos);
                }
            }
        }
    }
    pub fn tick(&mut self) {
        self.items.init();
        self.tiles.magnetic_force_dir = (0..4)
            .map(direction_by_index)
            .map(|dir| self.get_magnetic_force_dir(self.robbo.get_position(), dir))
            .find(|dir| dir.is_some())
            .unwrap_or(None);

        let actions = self.robbo.tick(&mut self.tiles);
        self.dispatch_actions(actions, self.robbo.get_position());

        self.tiles.robbo_pos = Some(self.robbo.get_position());

        for pos in self.items.item_indices_to_process() {
            if self.items.is_processed(pos) {
                continue;
            }
            if let Some(item) = self.items.get_mut(pos) {
                let actions = item.tick(&mut self.tiles);
                item.put_tile(&mut self.tiles);
                self.dispatch_actions(actions, pos);
            }
        }

        if self.robbo.is_hidden {
            self.missing_robbo_ticks += 1;
            if self.missing_robbo_ticks == 8 {
                self.explode_all();
            }
        } else {
            self.missing_robbo_ticks = 0;
        }

        let robbo_neighbours = self.tiles.get_neighbours(self.robbo.get_position());
        if robbo_neighbours.is_deadly() {
            self.dispatch_actions(
                Actions::single(Action::KillRobbo),
                self.robbo.get_position(),
            );
        }

        if !self.robbo.is_hidden && self.robbo.inventory.screws >= self.missing_screws {
            self.repair_capsule()
        }

        self.tiles.frame_cnt += 1;
    }

    pub fn repair_capsule(&mut self) {
        let repaired = self
            .items
            .iter_mut()
            .find(|item| item.get_kind() == Kind::Capsule)
            .map(|item| item.as_capsule())
            .flatten()
            .map(|c| c.repair());
        if let Some(true) = repaired {
            self.play_sound(Sound::Bomb)
        }
    }

    pub fn _shot(&mut self, pos: Position, direction: Direction, gun_type: GunType) {
        let dest = dest_coords(pos, direction);
        let is_dst_empty = self.tiles.is_empty(dest);
        if is_dst_empty {
            let bullet: Box<dyn Item> = match gun_type {
                GunType::Burst => Box::new(Bullet::new(direction)),
                GunType::Solid => Box::new(LaserHead::new(direction)),
                GunType::Blaster => Box::new(BlastHead::new(direction)),
            };
            self.add_item(dest, bullet);
        } else {
            self.destroy(dest, false);
        }
        if let GunType::Burst = gun_type {
            self.play_sound(Sound::Shot)
        }
    }

    pub fn kill_robbo(&mut self) {
        self.destroy(self.robbo.get_position(), false);
    }

    pub fn is_robbo_killed(&self) -> bool {
        self.missing_robbo_ticks > 20
    }

    pub fn explode_all(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let pos = (x, y);
                let kind = self.tiles.get_kind(pos);
                if kind != Kind::Empty && kind != Kind::Wall {
                    self.add_item(pos, Box::new(Animation::small_explosion()));
                }
            }
        }
        self.play_sound(Sound::Bomb)
    }

    pub fn robbo_move_or_shot(&mut self, dir: Direction, shot: bool) {
        self.robbo.set_direction(dir, shot)
    }
    pub fn play_sound(&self, sound: Sound) {
        self.sounds.play_sound(sound);
    }

    pub fn get_sounds(&self) -> Vec<Sound> {
        self.sounds.get_sounds()
    }
}
