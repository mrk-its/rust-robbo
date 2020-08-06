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
    Bullets,
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
    RelMove(Direction),
    LaserHeadMove(Direction),
    BlastHeadMove(Direction),
    BlastHeadDestroy,
    RelImpact(Direction, bool),
    CreateBullet(Direction),
    CreateLaser(Direction),
    CreateBlast(Direction),
    DestroyBullet,
    SpawnRobbo(bool),
    SpawnRandomItem,
    TeleportRobbo(u16, u16, Direction),
    NextLevel,
    BombExplosion,
    DoorOpened,
}
pub type Actions = Option<Vec<Action>>;
