use board::{Inventory, Neighbourhood};
use consts::{COLLECTABLE, DEADLY, DESTROYABLE, MOVEABLE, UNDESTROYABLE};
use random;
use types::{Action, Actions, Direction, Flags, Kind};
use utils::{
    direction_by_index, direction_to_index, reverse_direction, rotate_clockwise,
    rotate_counter_clockwise,
};

pub trait Item {
    fn get_simple_item(&self) -> &SimpleItem;
    fn get_tile(&self, frame_cnt: usize) -> usize {
        self.get_simple_item().get_tile(frame_cnt)
    }
    fn tick(&mut self, _neighbours: &Neighbourhood) -> Actions {
        None
    }
    fn get_kind(&self) -> Kind {
        self.get_simple_item().get_kind()
    }
    fn get_flags(&self) -> Flags {
        self.get_simple_item().get_flags()
    }
    fn enter(&mut self, _inventory: &mut Inventory, _direction: Direction) -> Actions {
        None
    }
    fn moved(&mut self, _direction: Direction) {}
    fn destroy(&mut self) -> bool {
        false
    }
    fn is_collectable(&self) -> bool {
        (self.get_flags() & COLLECTABLE) != 0
    }
    fn is_moveable(&self) -> bool {
        (self.get_flags() & MOVEABLE) != 0
    }
    fn is_destroyable(&self) -> bool {
        (self.get_flags() & DESTROYABLE) != 0
    }
    fn is_undestroyable(&self) -> bool {
        (self.get_flags() & UNDESTROYABLE) != 0
    }
    fn is_deadly(&self) -> bool {
        (self.get_flags() & DEADLY) != 0
    }

    fn as_robbo(&mut self) -> Option<&mut Robbo> {
        None
    }
    fn as_teleport(&self) -> Option<&Teleport> {
        None
    }
    fn as_capsule(&mut self) -> Option<&mut Capsule> {
        None
    }
    fn as_gun(&mut self) -> Option<&mut Gun> {
        None
    }
    fn as_magnet(&self) -> Option<&Magnet> {
        None
    }
    fn as_force_field(&self) -> Option<&ForceField> {
        None
    }
}

#[derive(Debug)]
pub struct SimpleItem {
    kind: Kind,
    tiles: &'static [usize],
    flags: Flags,
}
impl SimpleItem {
    pub fn new(kind: Kind, tiles: &'static [usize]) -> SimpleItem {
        SimpleItem {
            kind,
            tiles,
            flags: 0,
        }
    }
    pub fn flags(&self, flags: Flags) -> SimpleItem {
        SimpleItem {
            flags,
            ..*self
        }
    }
    fn get_tile(&self, frame_cnt: usize) -> usize {
        self.tiles[frame_cnt / 2 % self.tiles.len()]
    }
    fn get_kind(&self) -> Kind {
        self.kind
    }
    fn get_flags(&self) -> Flags {
        self.flags
    }
    pub fn wall(tiles: &'static [usize]) -> SimpleItem {
        SimpleItem::new(Kind::Wall, tiles).flags(UNDESTROYABLE)
    }
    pub fn abox() -> SimpleItem {
        SimpleItem::new(Kind::ABox, &[20]).flags(MOVEABLE)
    }
    pub fn horizontal_laser() -> SimpleItem {
        SimpleItem::new(Kind::HorizontalLaser, &[53])
    }
    pub fn vertical_laser() -> SimpleItem {
        SimpleItem::new(Kind::VerticalLaser, &[53])
    }
    pub fn laser_tail((dx, _dy): Direction) -> SimpleItem {
        SimpleItem::new(Kind::LaserTail, if dx != 0 { &[36, 37] } else { &[38, 39] })
            .flags(UNDESTROYABLE)
    }
    pub fn screw() -> SimpleItem {
        SimpleItem::new(Kind::Screw, &[4]).flags(COLLECTABLE | MOVEABLE)
    }
    pub fn questionmark() -> SimpleItem {
        SimpleItem::new(Kind::Questionmark, &[12]).flags(DESTROYABLE | MOVEABLE)
    }
    pub fn bullets() -> SimpleItem {
        SimpleItem::new(Kind::Bullets, &[5]).flags(DESTROYABLE | COLLECTABLE)
    }
    pub fn key() -> SimpleItem {
        SimpleItem::new(Kind::Key, &[42]).flags(COLLECTABLE)
    }
    pub fn ground() -> SimpleItem {
        SimpleItem::new(Kind::Ground, &[77]).flags(DESTROYABLE)
    }
}

impl Item for SimpleItem {
    fn get_simple_item(&self) -> &SimpleItem {
        self
    }
}

#[derive(Debug)]
pub struct Capsule {
    simple_item: SimpleItem,
    is_working: bool,
}

impl Capsule {
    pub fn new() -> Capsule {
        Capsule {
            simple_item: SimpleItem::new(Kind::Capsule, &[17, 18]).flags(MOVEABLE | UNDESTROYABLE),
            is_working: false,
        }
    }
    pub fn repair(&mut self) {
        self.is_working = true;
    }
}

impl Item for Capsule {
    fn get_simple_item(&self) -> &SimpleItem {
        &self.simple_item
    }
    fn get_tile(&self, frame_cnt: usize) -> usize {
        if self.is_working {
            self.simple_item.get_tile(frame_cnt)
        } else {
            self.simple_item.tiles[0]
        }
    }
    fn is_moveable(&self) -> bool {
        !self.is_working
    }
    fn as_capsule(&mut self) -> Option<&mut Capsule> {
        Some(self)
    }
    fn enter(&mut self, _inventory: &mut Inventory, _direction: Direction) -> Actions {
        if self.is_working {
            Some(vec![Action::NextLevel])
        } else {
            None
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
            simple_item: SimpleItem::new(Kind::ForceField, &[45, 57]).flags(DESTROYABLE),
            direction: params[0],
        }
    }
}

impl Item for ForceField {
    fn get_simple_item(&self) -> &SimpleItem {
        &self.simple_item
    }
    fn as_force_field(&self) -> Option<&ForceField> {
        Some(self)
    }
}

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
                MOVEABLE
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
    fn get_tile(&self, _frame_cnt: usize) -> usize {
        self.simple_item.tiles[direction_to_index(self.shooting_dir)]
    }
    fn tick(&mut self, neighbours: &Neighbourhood) -> Actions {
        let mut actions: Vec<Action> = Vec::new();
        if self.is_moveable {
            if neighbours.is_empty(self.moving_dir) {
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
        Some(actions)
    }
    fn as_gun(&mut self) -> Option<&mut Gun> {
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
    fn get_tile(&self, _frame_cnt: usize) -> usize {
        self.simple_item.tiles[self.dir]
    }
    fn as_magnet(&self) -> Option<&Magnet> {
        Some(self)
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
            simple_item: SimpleItem::new(Kind::Bird, &[15, 16]).flags(DESTROYABLE | DEADLY),
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
    fn tick(&mut self, neighbours: &Neighbourhood) -> Actions {
        let mut actions = vec![];
        if neighbours.is_empty(self.moving_dir) {
            actions.push(Action::RelMove(self.moving_dir))
        } else {
            let (dx, dy) = self.moving_dir;
            self.moving_dir = (-dx, -dy);
        }
        if self.is_shooting && random::random() < 0.1 {
            actions.push(Action::CreateBullet(self.shoting_dir));
        }
        Some(actions)
    }
}

#[derive(Debug)]
pub struct Butterfly {
    simple_item: SimpleItem,
}

impl Butterfly {
    const MOVE_PROBABILITY: f64 = 1.0;
    const RANDOM_MOVE_PROBABILITY: f64 = 0.1;

    pub fn new() -> Butterfly {
        Butterfly {
            simple_item: SimpleItem::new(Kind::Butterfly, &[32, 33]).flags(DESTROYABLE | DEADLY),
        }
    }
}

impl Item for Butterfly {
    fn get_simple_item(&self) -> &SimpleItem {
        &self.simple_item
    }
    fn tick(&mut self, neighbours: &Neighbourhood) -> Actions {
        if random::random() > Butterfly::MOVE_PROBABILITY {
            return None;
        }
        if random::random() < Butterfly::RANDOM_MOVE_PROBABILITY {
            let valid_dirs = (0..4)
                .map(direction_by_index)
                .filter(|dir| neighbours.is_empty(*dir))
                .collect::<Vec<Direction>>();
            if valid_dirs.len() > 0 {
                return Some(vec![Action::RelMove(
                    valid_dirs[random::randrange(valid_dirs.len() as u32) as usize],
                )]);
            }
        }
        neighbours
            .get_robbo_dir()
            .map(|(dx, dy)| {
                if dx.abs() < dy.abs() {
                    if dx != 0 && neighbours.is_empty((dx.signum(), 0)) {
                        (dx.signum(), 0)
                    } else if dy != 0 && neighbours.is_empty((0, dy.signum())) {
                        (0, dy.signum())
                    } else {
                        (0, 0)
                    }
                } else {
                    if dy != 0 && neighbours.is_empty((0, dy.signum())) {
                        (0, dy.signum())
                    } else if dx != 0 && neighbours.is_empty((dx.signum(), 0)) {
                        (dx.signum(), 0)
                    } else {
                        (0, 0)
                    }
                }
            })
            .map(|dir| vec![Action::RelMove(dir)])
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
            simple_item: SimpleItem::new(kind, tiles).flags(DESTROYABLE | DEADLY),
            moving_dir: direction_by_index(params[0] as usize),
        }
    }
}

impl Item for Bear {
    fn get_simple_item(&self) -> &SimpleItem {
        &self.simple_item
    }
    fn tick(&mut self, neighbours: &Neighbourhood) -> Actions {
        let mut actions = vec![];

        type RotateFn = dyn Fn(Direction) -> Direction;
        let (r1, r2): (&RotateFn, &RotateFn) = if self.simple_item.get_kind() == Kind::Bear {
            (&rotate_counter_clockwise, &rotate_clockwise)
        } else {
            (&rotate_clockwise, &rotate_counter_clockwise)
        };
        let new_dir = r1(self.moving_dir);
        let new_dir2 = r2(self.moving_dir);
        let new_dir3 = r2(new_dir2);
        if neighbours.is_empty(new_dir) {
            self.moving_dir = new_dir;
            actions.push(Action::RelMove(self.moving_dir));
        } else if neighbours.is_empty(self.moving_dir) {
            actions.push(Action::RelMove(self.moving_dir));
        } else if neighbours.is_empty(new_dir2) {
            self.moving_dir = new_dir2;
        } else if neighbours.is_empty(new_dir3) {
            self.moving_dir = new_dir3;
        }
        Some(actions)
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
    fn enter(&mut self, inventory: &mut Inventory, _direction: Direction) -> Actions {
        if inventory.keys > 0 {
            inventory.keys -= 1;
            inventory.show();
            self.open = true;
            return Some(vec![Action::DoorOpened]);
        }
        None
    }
    fn tick(&mut self, _neighbours: &Neighbourhood) -> Actions {
        if self.open {
            Some(vec![Action::AutoRemove])
        } else {
            None
        }
    }
}

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
}

impl Item for Teleport {
    fn get_simple_item(&self) -> &SimpleItem {
        &self.simple_item
    }
    fn enter(&mut self, _inventory: &mut Inventory, direction: Direction) -> Actions {
        Some(vec![Action::TeleportRobbo(
            self.group,
            self.position_in_group,
            direction,
        )])
    }
    fn as_teleport(&self) -> Option<&Teleport> {
        Some(self)
    }
}

pub struct Bullet {
    simple_item: SimpleItem,
    direction: Direction,
}
impl Bullet {
    pub fn new(direction: Direction) -> Bullet {
        Bullet {
            direction,
            simple_item: SimpleItem::new(Kind::Bullet, &[36, 37, 38, 39]).flags(UNDESTROYABLE),
        }
    }
}
impl Item for Bullet {
    fn get_simple_item(&self) -> &SimpleItem {
        &self.simple_item
    }
    fn get_tile(&self, frame_cnt: usize) -> usize {
        let (kx, _ky) = self.direction;
        self.simple_item.tiles[(if kx != 0 { 0 } else { 2 }) + (frame_cnt % 2)]
    }
    fn tick(&mut self, neighbours: &Neighbourhood) -> Actions {
        if neighbours.is_empty(self.direction) {
            Some(vec![Action::RelMove(self.direction)])
        } else {
            Some(vec![
                Action::DestroyBullet,
                Action::RelImpact(self.direction, false),
            ])
        }
    }
}

pub struct PushBox {
    simple_item: SimpleItem,
    direction: Direction,
}
impl PushBox {
    pub fn new() -> PushBox {
        PushBox {
            direction: (0, 0),
            simple_item: SimpleItem::new(Kind::ABox, &[6]).flags(MOVEABLE),
        }
    }
}
impl Item for PushBox {
    fn get_simple_item(&self) -> &SimpleItem {
        &self.simple_item
    }
    fn tick(&mut self, neighbours: &Neighbourhood) -> Actions {
        if self.direction != (0, 0) {
            if neighbours.is_empty(self.direction) {
                Some(vec![Action::RelMove(self.direction)])
            } else {
                let dir = self.direction;
                self.direction = (0, 0);
                Some(vec![Action::RelImpact(dir, false)])
            }
        } else {
            None
        }
    }
    fn moved(&mut self, direction: Direction) {
        self.direction = direction;
    }
}

pub struct LaserHead {
    simple_item: SimpleItem,
    direction: Direction,
    moving_back: bool,
}
impl LaserHead {
    pub fn new(direction: Direction) -> LaserHead {
        LaserHead {
            simple_item: SimpleItem::new(Kind::Bullet, &[36, 37, 38, 39]).flags(UNDESTROYABLE),
            direction,
            moving_back: false,
        }
    }
}
impl Item for LaserHead {
    fn get_simple_item(&self) -> &SimpleItem {
        &self.simple_item
    }
    fn get_tile(&self, frame_cnt: usize) -> usize {
        let (kx, _ky) = self.direction;
        self.simple_item.tiles[(if kx != 0 { 0 } else { 2 }) + (frame_cnt % 2)]
    }
    fn tick(&mut self, neighbours: &Neighbourhood) -> Actions {
        if neighbours.is_empty(self.direction)
            || self.moving_back && neighbours.get_kind(self.direction) == Kind::LaserTail
        {
            Some(vec![Action::LaserHeadMove(self.direction)])
        } else {
            if !self.moving_back {
                self.moving_back = true;
                let dir = self.direction;
                self.direction = reverse_direction(self.direction);
                Some(vec![Action::RelImpact(dir, false)])
            } else {
                Some(vec![
                    Action::DestroyBullet,
                    Action::RelImpact(self.direction, false),
                ])
            }
        }
    }
}

pub struct BlastHead {
    simple_item: SimpleItem,
    direction: Direction,
}
impl BlastHead {
    pub fn new(direction: Direction) -> BlastHead {
        BlastHead {
            simple_item: SimpleItem::new(Kind::Bullet, &[84]).flags(UNDESTROYABLE),
            direction,
        }
    }
}
impl Item for BlastHead {
    fn get_simple_item(&self) -> &SimpleItem {
        &self.simple_item
    }
    fn tick(&mut self, neighbours: &Neighbourhood) -> Actions {
        if neighbours.is_empty(self.direction)
            || (neighbours.get_flags(self.direction) & DESTROYABLE) > 0
                && (neighbours.get_kind(self.direction) != Kind::Bomb)
        {
            Some(vec![Action::BlastHeadMove(self.direction)])
        } else {
            Some(vec![
                Action::BlastHeadDestroy,
                Action::RelImpact(self.direction, false),
            ])
        }
    }
}

pub struct Robbo {
    simple_item: SimpleItem,
    direction: Direction,
}
impl Robbo {
    pub fn new() -> Robbo {
        Robbo {
            direction: (0, 1),
            simple_item: SimpleItem::new(Kind::Robbo, &[60, 61, 62, 63, 64, 65, 66, 67])
                .flags(DESTROYABLE),
        }
    }
    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }
}
impl Item for Robbo {
    fn get_simple_item(&self) -> &SimpleItem {
        &self.simple_item
    }
    fn get_tile(&self, frame_cnt: usize) -> usize {
        let index = direction_to_index(self.direction) * 2;
        self.simple_item.tiles[index + frame_cnt / 2 % 2]
    }
    fn as_robbo(&mut self) -> Option<&mut Robbo> {
        Some(self)
    }
}

pub struct Animation {
    simple_item: SimpleItem,
    frame: usize,
    final_action: Action,
}

impl Animation {
    pub fn new(kind: Kind, tiles: &'static [usize], final_action: Action) -> Animation {
        Animation {
            simple_item: SimpleItem::new(kind, tiles).flags(UNDESTROYABLE),
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
    fn get_tile(&self, _frame_cnt: usize) -> usize {
        self.simple_item.tiles[self.frame]
    }
    fn tick(&mut self, _neighbours: &Neighbourhood) -> Actions {
        if self.frame < self.simple_item.tiles.len() - 1 {
            self.frame += 1;
            None
        } else {
            Some(vec![self.final_action])
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BombState {
    Ready,
    Ignited,
    Exploded,
    Final,
}

pub struct Bomb {
    simple_item: SimpleItem,
    state: BombState,
}

impl Bomb {
    pub fn new() -> Bomb {
        Bomb {
            simple_item: SimpleItem::new(Kind::Bomb, &[8]).flags(DESTROYABLE | MOVEABLE),
            state: BombState::Ready,
        }
    }
}

impl Item for Bomb {
    fn get_simple_item(&self) -> &SimpleItem {
        &self.simple_item
    }
    fn tick(&mut self, _neighbours: &Neighbourhood) -> Actions {
        match self.state {
            BombState::Ignited => {
                self.state = BombState::Exploded;
                Some(vec![
                    Action::BombExplosion,
                    Action::RelImpact((1, -1), true),
                    Action::RelImpact((1, 1), true),
                    Action::RelImpact((0, 1), true),
                    Action::RelImpact((-1, 1), true),
                ])
            }
            BombState::Exploded => {
                self.state = BombState::Final;
                Some(vec![
                    Action::RelImpact((0, -1), true),
                    Action::RelImpact((-1, -1), true),
                    Action::RelImpact((-1, 0), true),
                    Action::RelImpact((1, 0), true),
                    Action::AutoRemove,
                ])
            }
            _ => None,
        }
    }
    fn destroy(&mut self) -> bool {
        if self.state == BombState::Ready {
            self.state = BombState::Ignited;
        }
        true
    }
}
