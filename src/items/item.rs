use super::{Robbo, Gun, Teleport, Capsule, Magnet, ForceField};
use consts;
use tiles::{Neighbourhood, Tile, Tiles};
use types::{Actions, Direction, Flags, Kind, Position};
use utils::dest_coords;

pub trait Item: std::fmt::Debug {
    fn put_tile(&self, tiles: &mut Tiles) {
        let kind = self.get_kind();
        let flags = self.get_flags();
        let tile = self.get_tile(tiles.frame_cnt);
        tiles.put(self.get_position(), Tile::new(kind, flags, tile));
    }
    fn set_position(&mut self, pos: Position) {
        self.get_simple_item_mut().pos = pos
    }
    fn get_position(&self) -> Position {
        self.get_simple_item().pos
    }
    fn _get_neighbours<'tiles>(&self, tiles: &'tiles Tiles) -> Neighbourhood<'tiles> {
        return Neighbourhood::new(tiles, self.get_position());
    }
    fn get_simple_item(&self) -> &SimpleItem;
    fn get_simple_item_mut(&mut self) -> &mut SimpleItem;
    fn get_tile(&self, frame_cnt: usize) -> usize {
        self.get_simple_item().get_tile(frame_cnt)
    }
    fn tick(&mut self, _tiles: &Tiles, _rng: &mut dyn rand::RngCore) -> Actions {
        Actions::empty()
    }
    fn _mv(&mut self, dir: Direction, tiles: &mut Tiles) -> bool {
        let pos = self.get_position();
        let dst = dest_coords(pos, dir);
        if tiles.is_empty(dst) {
            tiles.put_empty(pos);
            self.set_position(dst);
            self.put_tile(tiles);
            true
        } else {
            false
        }
    }
    fn get_kind(&self) -> Kind {
        self.get_simple_item().get_kind()
    }
    fn get_flags(&self) -> Flags {
        self.get_simple_item().get_flags()
    }
    fn enter(&mut self, _robbo: &mut Robbo, _direction: Direction) -> Actions {
        Actions::empty()
    }
    fn pushed(&mut self, _direction: Direction) {}
    fn destroy(&mut self) -> bool {
        false
    }
    fn as_teleport(&self) -> Option<&Teleport> {
        None
    }
    fn as_mut_capsule(&mut self) -> Option<&mut Capsule> {
        None
    }
    fn as_mut_gun(&mut self) -> Option<&mut Gun> {
        None
    }
    fn as_gun(&self) -> Option<&Gun> {
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
    pub tiles: &'static [usize],
    flags: Flags,
    pub pos: Position,
}
impl SimpleItem {
    pub fn new(kind: Kind, tiles: &'static [usize]) -> SimpleItem {
        SimpleItem {
            kind,
            tiles,
            flags: 0,
            pos: (0, 0),
        }
    }
    pub fn flags(&self, flags: Flags) -> SimpleItem {
        SimpleItem { flags, ..*self }
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
    pub fn abox() -> SimpleItem {
        SimpleItem::new(Kind::ABox, &[20]).flags(consts::MOVEABLE)
    }
    pub fn horizontal_laser() -> SimpleItem {
        SimpleItem::new(Kind::HorizontalLaser, &[53])
    }
    pub fn vertical_laser() -> SimpleItem {
        SimpleItem::new(Kind::VerticalLaser, &[53])
    }
    pub fn laser_tail((dx, _dy): Direction) -> SimpleItem {
        SimpleItem::new(Kind::LaserTail, if dx != 0 { &[36, 37] } else { &[38, 39] })
            .flags(consts::UNDESTROYABLE)
    }
    pub fn questionmark() -> SimpleItem {
        SimpleItem::new(Kind::Questionmark, &[12]).flags(consts::DESTROYABLE | consts::MOVEABLE)
    }
}

impl Item for SimpleItem {
    fn get_simple_item(&self) -> &SimpleItem {
        self
    }
    fn get_simple_item_mut(&mut self) -> &mut SimpleItem {
        self
    }
}

