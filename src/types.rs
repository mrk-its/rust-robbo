use sound::Sound;
pub type Position = (i32, i32);
pub type Direction = (i32, i32);
pub type Flags = u16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Empty,
    Wall,
    Ground,
    Robbo,
    Bullet,
    LaserTail,
    Ammo,
    Screw,
    Capsule,
    Key,
    Door,
    ABox,
    Bomb,
    Questionmark,
    Teleport,
    Butterfly,
    Bear,
    BlackBear,
    Bird,
    Gun,
    HorizontalLaser,
    VerticalLaser,
    Magnet,
    ForceField,
    Explosion,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    AutoRemove,
    RobboMove(Direction),
    RelMove(Direction),
    ForceRelMove(Direction),  // similar to above but with overriding in front
    CreateLaserTail(Position, Direction),
    CreateBlastTail(Position, Direction),
    RelImpact(Direction, bool),
    CreateBullet(Direction),
    CreateLaser(Direction),
    CreateBlast(Direction),
    SpawnRobbo(bool),
    SpawnRandomItem,
    TeleportRobbo(u16, u16, Direction),
    NextLevel,
    SmallExplosion,
    KillRobbo,
    ExplodeAll,
    ForceField,
    PlaySound(Sound),
}
pub struct Actions {
    actions: Vec<Action>
}

impl Actions {
    pub fn new(actions: &[Action]) -> Actions {
        Actions {actions: actions.iter().cloned().collect()}
    }
    pub fn single(action: Action) -> Actions {
        Actions {actions: vec![action]}
    }
    pub fn empty() -> Actions {
        Actions {actions: vec![]}
    }
    pub fn push(&mut self, action: Action) {
        self.actions.push(action);
    }
    pub fn pop_first(&mut self) -> Option<Action> {
        if !self.actions.is_empty() {
            Some(self.actions.remove(0))
        } else {
            None
        }
    }
    pub fn extend(&mut self, other: &Actions) {
        self.actions.extend_from_slice(&other.actions)
    }
}

impl std::iter::IntoIterator for Actions {
    type Item = Action;
    type IntoIter = std::vec::IntoIter<Action>;
    fn into_iter(self) -> Self::IntoIter {
        self.actions.into_iter()
    }
}
