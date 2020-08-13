mod butterfly;
mod gun;
mod item;
mod robbo;
mod teleport;

pub use self::butterfly::Butterfly;
pub use self::gun::{Gun, GunType};
pub use self::item::{Item, SimpleItem};
pub use self::robbo::{Inventory, Robbo};
pub use self::teleport::Teleport;

use sound::Sound;
use consts;
use random;
use tiles::Tiles;
use types::{Action, Actions, Direction, Kind};
use utils::{
    dest_coords, direction_by_index, reverse_direction, rotate_clockwise, rotate_counter_clockwise,
};

#[derive(Debug)]
pub struct Capsule {
    simple_item: SimpleItem,
    is_working: bool,
}

impl Capsule {
    pub fn new() -> Capsule {
        Capsule {
            simple_item: SimpleItem::new(Kind::Capsule, &[17, 18])
                .flags(consts::MOVEABLE | consts::UNDESTROYABLE),
            is_working: false,
        }
    }
    pub fn repair(&mut self) -> bool {
        let was_not_working = !self.is_working;
        self.is_working = true;
        was_not_working
    }
}

impl Item for Capsule {
    fn get_simple_item(&self) -> &SimpleItem {
        &self.simple_item
    }
    fn get_simple_item_mut(&mut self) -> &mut SimpleItem {
        &mut self.simple_item
    }
    fn get_tile(&self, frame_cnt: usize) -> usize {
        if self.is_working {
            self.simple_item.get_tile(frame_cnt)
        } else {
            self.simple_item.tiles[0]
        }
    }
    fn get_flags(&self) -> u16 {
        let mut flags = self.get_simple_item().get_flags();
        if self.is_working {
            flags = flags & !consts::MOVEABLE;
        }
        flags
    }
    fn as_capsule(&mut self) -> Option<&mut Capsule> {
        Some(self)
    }
    fn enter(&mut self, _robbo: &mut Robbo, _direction: Direction) -> Actions {
        if self.is_working {
            Actions::new(&[Action::NextLevel])
        } else {
            Actions::empty()
        }
    }
}

#[derive(Debug)]
pub struct ForceField {
    simple_item: SimpleItem,
    pub direction: u16,
}

impl ForceField {
    pub fn new(params: &[u16]) -> ForceField {
        ForceField {
            simple_item: SimpleItem::new(Kind::ForceField, &[45, 57]).flags(consts::DESTROYABLE),
            direction: params[0],
        }
    }
}

impl Item for ForceField {
    fn get_simple_item(&self) -> &SimpleItem {
        &self.simple_item
    }
    fn get_simple_item_mut(&mut self) -> &mut SimpleItem {
        &mut self.simple_item
    }
    fn as_force_field(&self) -> Option<&ForceField> {
        Some(self)
    }
}

#[derive(Debug)]
pub struct Magnet {
    simple_item: SimpleItem,
    dir: usize,
}
impl Magnet {
    pub fn new(params: &[u16]) -> Magnet {
        Magnet {
            simple_item: SimpleItem::new(Kind::Magnet, &[0, 72, 1, 73]),
            dir: params[0] as usize,
        }
    }
    pub fn get_magnetic_force_dir(&self) -> Direction {
        reverse_direction(direction_by_index(self.dir))
    }
}
impl Item for Magnet {
    fn get_simple_item(&self) -> &SimpleItem {
        &self.simple_item
    }
    fn get_simple_item_mut(&mut self) -> &mut SimpleItem {
        &mut self.simple_item
    }
    fn get_tile(&self, _frame_cnt: usize) -> usize {
        self.simple_item.tiles[self.dir]
    }
    fn as_magnet(&self) -> Option<&Magnet> {
        Some(self)
    }
    fn enter(&mut self, _robbo: &mut Robbo, dir: Direction) -> Actions {
        if reverse_direction(direction_by_index(self.dir)) == dir {
            Actions::single(Action::KillRobbo)
        } else {
            Actions::empty()
        }
    }
}

#[derive(Debug)]
pub struct Bird {
    simple_item: SimpleItem,
    moving_dir: Direction,
    shoting_dir: Direction,
    pub is_shooting: bool,
}
impl Bird {
    pub fn new(params: &[u16]) -> Bird {
        Bird {
            simple_item: SimpleItem::new(Kind::Bird, &[15, 16])
                .flags(consts::DESTROYABLE | consts::DEADLY),
            moving_dir: direction_by_index(params[0] as usize),
            shoting_dir: direction_by_index(params[1] as usize),
            is_shooting: params[2] > 0,
        }
    }
}

impl Item for Bird {
    fn get_simple_item(&self) -> &SimpleItem {
        &self.simple_item
    }
    fn get_simple_item_mut(&mut self) -> &mut SimpleItem {
        &mut self.simple_item
    }
    fn tick(&mut self, tiles: &Tiles) -> Actions {
        let neighbours = tiles.get_neighbours(self.get_position());
        let mut actions = Actions::empty();
        if neighbours.get(self.moving_dir).is_empty() {
            return Actions::single(Action::RelMove(self.moving_dir));
        } else {
            let (dx, dy) = self.moving_dir;
            self.moving_dir = (-dx, -dy);
        }
        if self.is_shooting && random::random() < 0.1 {
            actions.push(Action::CreateBullet(self.shoting_dir));
        }
        actions
    }
}

#[derive(Debug)]
pub struct Bear {
    simple_item: SimpleItem,
    moving_dir: Direction,
}
impl Bear {
    pub fn new(kind: Kind, params: &[u16], tiles: &'static [usize]) -> Bear {
        Bear {
            simple_item: SimpleItem::new(kind, tiles).flags(consts::DESTROYABLE | consts::DEADLY),
            moving_dir: direction_by_index(params[0] as usize),
        }
    }
}

impl Item for Bear {
    fn get_simple_item(&self) -> &SimpleItem {
        &self.simple_item
    }
    fn get_simple_item_mut(&mut self) -> &mut SimpleItem {
        &mut self.simple_item
    }
    fn tick(&mut self, tiles: &Tiles) -> Actions {
        let neighbours = tiles.get_neighbours(self.get_position());
        type RotateFn = dyn Fn(Direction) -> Direction;
        let (r1, r2): (&RotateFn, &RotateFn) = if self.simple_item.get_kind() == Kind::Bear {
            (&rotate_counter_clockwise, &rotate_clockwise)
        } else {
            (&rotate_clockwise, &rotate_counter_clockwise)
        };
        let new_dir = r1(self.moving_dir);
        let new_dir2 = r2(self.moving_dir);
        let new_dir3 = r2(new_dir2);
        if neighbours.get(new_dir).is_empty() {
            self.moving_dir = new_dir;
            return Actions::single(Action::RelMove(new_dir));
        } else if neighbours.get(self.moving_dir).is_empty() {
            return Actions::single(Action::RelMove(self.moving_dir));
        } else if neighbours.get(new_dir2).is_empty() {
            self.moving_dir = new_dir2;
        } else if neighbours.get(new_dir3).is_empty() {
            self.moving_dir = new_dir3;
        }
        Actions::empty()
    }
}

#[derive(Debug)]
pub struct Door {
    simple_item: SimpleItem,
    open: bool,
}
impl Door {
    pub fn new() -> Door {
        Door {
            open: false,
            simple_item: SimpleItem::new(Kind::Door, &[9]),
        }
    }
}
impl Item for Door {
    fn get_simple_item(&self) -> &SimpleItem {
        &self.simple_item
    }
    fn get_simple_item_mut(&mut self) -> &mut SimpleItem {
        &mut self.simple_item
    }
    fn enter(&mut self, robbo: &mut Robbo, _direction: Direction) -> Actions {
        if robbo.inventory.keys > 0 {
            robbo.inventory.keys -= 1;
            robbo.inventory.show();
            self.open = true;
            return Actions::single(Action::PlaySound(Sound::Door));
        }
        Actions::empty()
    }
    fn tick(&mut self, _tiles: &Tiles) -> Actions {
        if self.open {
            Actions::new(&[Action::AutoRemove])
        } else {
            Actions::empty()
        }
    }
}

#[derive(Debug)]
pub struct Bullet {
    simple_item: SimpleItem,
    direction: Direction,
}
impl Bullet {
    pub fn new(direction: Direction) -> Bullet {
        Bullet {
            direction,
            simple_item: SimpleItem::new(Kind::Bullet, &[36, 37, 38, 39])
                .flags(consts::UNDESTROYABLE),
        }
    }
}
impl Item for Bullet {
    fn get_simple_item(&self) -> &SimpleItem {
        &self.simple_item
    }
    fn get_simple_item_mut(&mut self) -> &mut SimpleItem {
        &mut self.simple_item
    }
    fn get_tile(&self, frame_cnt: usize) -> usize {
        let (kx, _ky) = self.direction;
        self.simple_item.tiles[(if kx != 0 { 0 } else { 2 }) + (frame_cnt % 2)]
    }
    fn tick(&mut self, tiles: &Tiles) -> Actions {
        let neighbours = tiles.get_neighbours(self.get_position());
        if neighbours.get(self.direction).is_empty() {
            Actions::single(Action::RelMove(self.direction))
        } else if neighbours.get(self.direction).is_destroyable() {
            Actions::new(&[Action::AutoRemove, Action::RelImpact(self.direction, false)])
        } else {
            Actions::new(&[
                Action::AutoRemove,
                Action::SmallExplosion,
                Action::RelImpact(self.direction, false),
            ])
        }
    }
}

#[derive(Debug)]
pub struct PushBox {
    simple_item: SimpleItem,
    direction: Direction,
}
impl PushBox {
    pub fn new() -> PushBox {
        PushBox {
            direction: (0, 0),
            simple_item: SimpleItem::new(Kind::ABox, &[6]).flags(consts::MOVEABLE),
        }
    }
}
impl Item for PushBox {
    fn get_simple_item(&self) -> &SimpleItem {
        &self.simple_item
    }
    fn get_simple_item_mut(&mut self) -> &mut SimpleItem {
        &mut self.simple_item
    }
    fn tick(&mut self, tiles: &Tiles) -> Actions {
        let neighbours = tiles.get_neighbours(self.get_position());
        if self.direction != (0, 0) {
            if neighbours.get(self.direction).is_empty() {
                Actions::new(&[Action::RelMove(self.direction)])
            } else {
                let dir = self.direction;
                self.direction = (0, 0);
                Actions::new(&[Action::RelImpact(dir, false)])
            }
        } else {
            Actions::empty()
        }
    }
    fn pushed(&mut self, direction: Direction) {
        self.direction = direction;
    }
}

#[derive(Debug)]
pub struct LaserHead {
    simple_item: SimpleItem,
    direction: Direction,
    moving_back: bool,
}
impl LaserHead {
    pub fn new(direction: Direction) -> LaserHead {
        LaserHead {
            simple_item: SimpleItem::new(Kind::Bullet, &[36, 37, 38, 39])
                .flags(consts::UNDESTROYABLE),
            direction,
            moving_back: false,
        }
    }
}
impl Item for LaserHead {
    fn get_simple_item(&self) -> &SimpleItem {
        &self.simple_item
    }
    fn get_simple_item_mut(&mut self) -> &mut SimpleItem {
        &mut self.simple_item
    }
    fn get_tile(&self, frame_cnt: usize) -> usize {
        let (kx, _ky) = self.direction;
        self.simple_item.tiles[(if kx != 0 { 0 } else { 2 }) + (frame_cnt % 2)]
    }
    fn tick(&mut self, tiles: &Tiles) -> Actions {
        let neighbours = tiles.get_neighbours(self.get_position());
        if neighbours.get(self.direction).is_empty() {
            let pos = self.get_position();
            Actions::new(&[
                Action::RelMove(self.direction),
                Action::CreateLaserTail(pos, self.direction),
            ])
        } else if self.moving_back && neighbours.get_kind(self.direction) == Kind::LaserTail {
            Actions::new(&[Action::ForceRelMove(self.direction)])
        } else if !self.moving_back {
            self.moving_back = true;
            let dir = self.direction;
            self.direction = reverse_direction(self.direction);
            Actions::new(&[Action::RelImpact(dir, false)])
        } else {
            Actions::new(&[
                Action::AutoRemove,
                Action::SmallExplosion,
                // Action::RelImpact(self.direction, false),
            ])
        }
    }
}

#[derive(Debug)]
pub struct BlastHead {
    simple_item: SimpleItem,
    direction: Direction,
}
impl BlastHead {
    pub fn new(direction: Direction) -> BlastHead {
        BlastHead {
            simple_item: SimpleItem::new(Kind::Bullet, &[84]).flags(consts::UNDESTROYABLE),
            direction,
        }
    }
}
impl Item for BlastHead {
    fn get_simple_item(&self) -> &SimpleItem {
        &self.simple_item
    }
    fn get_simple_item_mut(&mut self) -> &mut SimpleItem {
        &mut self.simple_item
    }
    fn tick(&mut self, tiles: &Tiles) -> Actions {
        let pos = self.get_position();
        let dst_pos = dest_coords(pos, self.direction);
        if let Some(tile) = tiles.get(dst_pos) {
            if tile.is_empty() || tile.is_destroyable() {
                return Actions::new(&[
                    Action::ForceRelMove(self.direction),
                    Action::CreateBlastTail(pos, self.direction),
                ])
            }
        }
        Actions::new(&[
            Action::AutoRemove,
            Action::CreateBlastTail(self.get_position(), self.direction),
        ])
    }
}

#[derive(Debug)]
pub struct Animation {
    simple_item: SimpleItem,
    frame: usize,
    final_action: Action,
}

impl Animation {
    pub fn new(kind: Kind, tiles: &'static [usize], final_action: Action) -> Animation {
        Animation {
            simple_item: SimpleItem::new(kind, tiles).flags(consts::UNDESTROYABLE),
            frame: 0,
            final_action,
        }
    }
    pub fn small_explosion() -> Animation {
        Animation::new(Kind::Explosion, &[52, 51, 50], Action::AutoRemove)
    }
    pub fn spawn_robbo() -> Animation {
        Animation::new(
            Kind::Explosion,
            &[17, 18, 17, 18, 50, 51, 52],
            Action::SpawnRobbo(true),
        )
    }
    pub fn kill_robbo() -> Animation {
        Animation::new(Kind::Explosion, &[52, 51, 50], Action::ExplodeAll)
    }
    pub fn teleport_robbo() -> Animation {
        Animation::new(Kind::Explosion, &[50, 51, 52], Action::SpawnRobbo(false))
    }
    pub fn question_mark_explosion() -> Animation {
        Animation::new(Kind::Explosion, &[50, 51, 52], Action::SpawnRandomItem)
    }
    pub fn blast_tail() -> Animation {
        Animation::new(Kind::Bullet, &[85, 86, 86, 86, 85, 84], Action::AutoRemove)
    }
}

impl Item for Animation {
    fn get_simple_item(&self) -> &SimpleItem {
        &self.simple_item
    }
    fn get_simple_item_mut(&mut self) -> &mut SimpleItem {
        &mut self.simple_item
    }
    fn get_tile(&self, _frame_cnt: usize) -> usize {
        self.simple_item.tiles[self.frame]
    }
    fn tick(&mut self, _tiles: &Tiles) -> Actions {
        let actions = if self.frame < self.simple_item.tiles.len() - 1 {
            self.frame += 1;
            Actions::empty()
        } else {
            Actions::new(&[Action::AutoRemove, self.final_action])
        };
        actions
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BombState {
    Ready,
    Ignited,
    Exploded,
    Final,
}

#[derive(Debug)]
pub struct Bomb {
    simple_item: SimpleItem,
    state: BombState,
}

impl Bomb {
    pub fn new() -> Bomb {
        Bomb {
            simple_item: SimpleItem::new(Kind::Bomb, &[8])
                .flags(consts::DESTROYABLE | consts::MOVEABLE),
            state: BombState::Ready,
        }
    }
}

impl Item for Bomb {
    fn get_simple_item(&self) -> &SimpleItem {
        &self.simple_item
    }
    fn get_simple_item_mut(&mut self) -> &mut SimpleItem {
        &mut self.simple_item
    }
    fn tick(&mut self, _tiles: &Tiles) -> Actions {
        match self.state {
            BombState::Ignited => {
                self.state = BombState::Exploded;
                Actions::new(&[
                    Action::PlaySound(Sound::Bomb),
                    Action::RelImpact((1, -1), true),
                    Action::RelImpact((1, 1), true),
                    Action::RelImpact((0, 1), true),
                    Action::RelImpact((-1, 1), true),
                ])
            }
            BombState::Exploded => {
                self.state = BombState::Final;
                Actions::new(&[
                    Action::RelImpact((0, -1), true),
                    Action::RelImpact((-1, -1), true),
                    Action::RelImpact((-1, 0), true),
                    Action::RelImpact((1, 0), true),
                    Action::AutoRemove,
                ])
            }
            _ => Actions::empty(),
        }
    }
    fn destroy(&mut self) -> bool {
        if self.state == BombState::Ready {
            self.state = BombState::Ignited;
        }
        true
    }
}
