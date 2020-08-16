use consts;
use types::{Direction, Flags, Kind, Position};
use utils::dest_coords;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tile {
    kind: Kind,
    flags: Flags,
    tile: usize,
}
impl Tile {
    pub fn new(kind: Kind, flags: Flags, tile: usize) -> Tile {
        Tile { kind, flags, tile }
    }
    pub fn wall(tile: usize) -> Tile {
        Tile::new(Kind::Wall, consts::UNDESTROYABLE, tile)
    }
    pub fn ammo() -> Tile {
        Tile::new(Kind::Ammo, consts::COLLECTABLE, 5)
    }
    pub fn screw() -> Tile {
        Tile::new(Kind::Screw, consts::COLLECTABLE, 4)
    }
    pub fn key() -> Tile {
        Tile::new(Kind::Key, consts::COLLECTABLE, 42)
    }
    pub fn ground() -> Tile {
        Tile::new(Kind::Ground, consts::DESTROYABLE, 77)
    }
    fn is_flag_set(&self, flags: Flags) -> bool {
        (self.flags & flags) > 0
    }
    pub fn is_collectable(&self) -> bool {
        self.is_flag_set(consts::COLLECTABLE)
    }
    pub fn is_moveable(&self) -> bool {
        self.is_flag_set(consts::MOVEABLE)
    }
    pub fn is_destroyable(&self) -> bool {
        self.is_flag_set(consts::DESTROYABLE)
    }
    pub fn is_deadly(&self) -> bool {
        self.is_flag_set(consts::DEADLY)
    }
    pub fn is_undestroyable(&self) -> bool {
        self.is_flag_set(consts::UNDESTROYABLE)
    }
    pub fn is_empty(&self) -> bool {
        self.kind == Kind::Empty
    }
    pub fn get_kind(&self) -> Kind {
        self.kind
    }
    pub fn get_tile(&self) -> usize {
        self.tile
    }
}

const EMPTY: Tile = Tile {
    kind: Kind::Empty,
    flags: 0,
    tile: 95,
};

const WALL: Tile = Tile {
    kind: Kind::Wall,
    flags: 0,
    tile: 0,
};

pub struct Tiles {
    width: i32,
    tiles: Vec<Tile>,
    pub frame_cnt: usize,
    pub robbo_pos: Option<Position>,
    pub magnetic_force_dir: Option<Direction>,
}

impl Tiles {
    pub fn new(width: i32, height: i32) -> Tiles {
        let mut tiles = Vec::with_capacity((width * height) as usize);
        for _ in 0..width * height {
            tiles.push(EMPTY)
        }
        Tiles {
            width,
            tiles,
            frame_cnt: 0,
            robbo_pos: Some((0, 0)),
            magnetic_force_dir: None,
        }
    }
    pub fn put(&mut self, pos: Position, tile: Tile) {
        self.tiles[(pos.0 + pos.1 * self.width) as usize] = tile;
    }
    pub fn put_empty(&mut self, pos: Position) {
        self.put(pos, EMPTY)
    }
    pub fn get(&self, pos: Position) -> Option<&Tile> {
        self.tiles.get((pos.0 + pos.1 * self.width) as usize)
    }
    pub fn get_or_wall(&self, pos: Position) -> Tile {
        *self.tiles.get((pos.0 + pos.1 * self.width) as usize).unwrap_or(&WALL)
    }
    pub fn get_kind(&self, pos: Position) -> Kind {
        self.get(pos).map(|v| v.kind).unwrap_or(Kind::Wall)
    }
    pub fn is_empty(&self, pos: Position) -> bool {
        self.get_kind(pos) == Kind::Empty
    }
    pub fn get_neighbours(&self, pos: Position) -> Neighbourhood {
        return Neighbourhood::new(
            &self,
            pos,
        )
    }
}

#[derive(Clone, Copy)]
pub struct Neighbourhood<'tiles> {
    pos: Position,
    tiles: &'tiles Tiles,
    robbo_dir: Option<Direction>,
}

impl<'tiles> Neighbourhood<'tiles> {
    pub fn new(tiles: &'tiles Tiles, pos: Position) -> Neighbourhood<'tiles> {
        let robbo_dir = tiles.robbo_pos.map(|(robbo_x, robbo_y)| {
            (
                (robbo_x as i32) - (pos.0 as i32),
                (robbo_y as i32) - (pos.1 as i32),
            )
        });

        Neighbourhood {
            pos,
            tiles,
            robbo_dir,
        }
    }
    pub fn get_robbo_dir(&self) -> Option<Direction> {
        self.robbo_dir
    }
    pub fn get(&self, direction: Direction) -> &Tile {
        let dst = dest_coords(self.pos, direction);
        self.tiles.get(dst).unwrap_or(&WALL)
    }
    pub fn get_kind(&self, direction: Direction) -> Kind {
        self.get(direction).kind
    }
    pub fn is_deadly(&self) -> bool {
        consts::ALL_DIRS
            .iter()
            .map(|i| self.get(*i))
            .any(|t| t.is_deadly())
    }
}
