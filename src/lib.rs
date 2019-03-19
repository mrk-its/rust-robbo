extern crate cfg_if;
extern crate wasm_bindgen;
extern crate web_sys;
extern crate js_sys;

#[macro_use]
mod log;
mod utils;
mod levels;
mod types;
mod consts;
mod items;
mod board;
mod original_level_data;
mod forever_level_data;
mod random;

use types::Direction;
use std::collections::HashMap;
use utils::{rotate_clockwise, modulo, set_panic_hook};
use board::Board;
use std::cmp::min;
use log::log;
use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;
use web_sys::{ImageData, CanvasRenderingContext2d, KeyboardEvent, console};
use levels::LevelSet;


cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct ArrowKeys {
    right: i32,
    down: i32,
    left: i32,
    up: i32,
}

#[wasm_bindgen]
pub struct Universe {
    frame_cnt: usize,
    current_level: usize,
    current_levelset: usize,
    skin_image_data: ImageData,
    level_sets: Vec<LevelSet>,
    board: Board,
    last_chars: String,
    tile_map: HashMap<usize,usize>,
    is_rotated: bool,
    arrow_keys: ArrowKeys,
}

/// Public methods, exported to JavaScript.
#[wasm_bindgen]
impl Universe {

    pub fn reload_level(&mut self) {
        let level = &self.level_sets[self.current_levelset].levels[self.current_level];
        log(&format!("{:#?}", level));
        self.board = Board::from(level);
    }

    fn rotate(&self, dir: Direction) -> Direction {
        if self.is_rotated {
            rotate_clockwise(dir)
        } else {
            dir
        }
    }

    pub fn on_keyboard_event(&mut self, event: &KeyboardEvent, is_keydown: bool) -> bool {
        let is_shift = event.shift_key();
        if is_shift && is_keydown {
            let dir = match event.code().as_ref() {
                "ArrowLeft" => (-1, 0),
                "ArrowRight" => (1, 0),
                "ArrowUp" => (0, -1),
                "ArrowDown" => (0, 1),
                _ => (0, 0),
            };
            if dir != (0, 0) {
                let dir = self.rotate(dir);
                self.board.robbo_shot_event(dir);
                return true;
            }

        }

        if is_keydown {
            match event.code().as_ref() {
                "Escape" => {self.board.kill_robbo(); return true}
                "Backslash" => {self.is_rotated = !self.is_rotated; return true}
                "BracketLeft" => {
                    if is_shift {
                        self.current_levelset = modulo(
                            self.current_levelset as i32 - 1,
                            self.level_sets.len() as i32
                        ) as usize;
                        self.current_level = 0;
                    } else {
                        self.current_level = modulo(
                            self.current_level as i32 -1,
                            self.level_sets[self.current_levelset].size() as i32
                        ) as usize;
                    }
                    self.reload_level();
                    return true;
                },
                "BracketRight" => {
                    if is_shift {
                        self.current_levelset = (self.current_levelset + 1) % self.level_sets.len();
                        self.current_level = 0;
                    } else {
                        self.current_level = (self.current_level + 1) % self.level_sets[self.current_levelset].size();
                    }
                    self.reload_level();
                    return true;
                },
                _ => {
                    self.last_chars.insert_str(0, &event.key());
                    self.last_chars = self.last_chars[..min(16, self.last_chars.len())].to_lowercase();
                    if self.last_chars.len() >= 3 && self.last_chars[0..3].eq("alo") {
                        self.board.god_mode();
                    }
                }
            };
        }

        let pressed = is_keydown as i32;
        let is_handled = match event.code().as_ref() {
            "ArrowLeft" => {self.arrow_keys.left = pressed; true},
            "ArrowRight" => {self.arrow_keys.right = pressed; true},
            "ArrowUp" => {self.arrow_keys.up = pressed; true},
            "ArrowDown" => {self.arrow_keys.down = pressed; true},
            _ => false,
        };


        if is_handled {
            let kx = self.arrow_keys.right - self.arrow_keys.left;
            let ky = self.arrow_keys.down - self.arrow_keys.up;
            let dir = self.rotate((kx, ky));
            self.board.robbo_move_event(dir);
        }
        is_handled
    }

    pub fn new(skin_image_data: &ImageData, current_levelset: usize, current_level: usize) -> Universe {
        console::log_1(&JsValue::from_str("hello from wasm!"));
        set_panic_hook();

        let skin_image_data = skin_image_data.clone();
        let level_sets = vec![
            LevelSet::parse(original_level_data::LEVEL_DATA),
            LevelSet::parse(forever_level_data::LEVEL_DATA),
        ];
        let board = Board::from(&level_sets[current_levelset].levels[current_level]);
        let tile_mappings = vec![
            (36, 38), // bullet / laser
            (37, 39),

            (53, 54), // gun
            (54, 55),
            (55, 56),
            (56, 53),

            (1, 72),  //
            (0, 73),

            (89, 90),
        ];
        let reverse: Vec<(usize, usize)> = tile_mappings.iter().map(|(k, v)| (*v, *k)).collect();
        let tile_map: HashMap<usize, usize> = tile_mappings.iter().chain(&reverse).cloned().collect();

        Universe {
            frame_cnt: 0,
            current_level,
            current_levelset,
            skin_image_data,
            level_sets,
            board,
            last_chars: String::from(""),
            tile_map,
            is_rotated: true,
            arrow_keys: ArrowKeys {up: 0, down: 0, left: 0, right: 0}
        }
    }

    pub fn width(&self) -> usize {
        (if !self.is_rotated {self.board.width} else {self.board.height} * 32) as usize
    }

    pub fn height(&self) -> usize {
        (if !self.is_rotated {self.board.height} else {self.board.width} * 32) as usize
    }

    pub fn get_current_level(&self) -> usize {
        self.current_level
    }
    pub fn get_current_levelset(&self) -> usize {
        self.current_levelset
    }

    pub fn get_inventory(&self) -> String {
        String::from(format!(
            "level: {:02} screws: {:02} keys: {:02} bullets: {:02}",
            self.current_level + 1,
            self.board.missing_screws - self.board.inventory.screws,
            self.board.inventory.keys,
            self.board.inventory.bullets))
    }

    pub fn get_missing_robbo_ticks(&self) -> usize {
        return self.board.missing_robbo_ticks;
    }
    pub fn draw_tile(&self, ctx: &CanvasRenderingContext2d, n:usize, dx: usize, dy: usize) {
        let (dx, dy) = if self.is_rotated {
            (dy, 15-dx)
        } else {
            (dx, dy)
        };
        let n = if self.is_rotated {
            *self.tile_map.get(&n).unwrap_or(&n)
        } else {
            n
        };
        let sx = n % 12;
        let sy = n / 12;
        let dirtyx = (sx * 34 + 2) as f64;
        let dirtyy = (sy * 34 + 2) as f64;
        ctx.put_image_data_with_dirty_x_and_dirty_y_and_dirty_width_and_dirty_height(
            &self.skin_image_data,
            (dx * 32) as f64 - dirtyx,
            (dy * 32) as f64 - dirtyy,
            dirtyx,
            dirtyy,
            32.0,
            32.0
        ).unwrap();
    }

    pub fn load_next_level(&mut self) {
        self.current_level += 1;
        if self.current_level >= self.level_sets[self.current_levelset].size() {
            self.current_level = 0;
            self.current_levelset = (self.current_levelset + 1) % self.level_sets.len()
        }
        self.reload_level();
    }

    pub fn draw(&mut self, ctx: &CanvasRenderingContext2d) {
//        let image_data = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&mut self.image_data), (self.width * 4) as u32, (self.height * 4) as u32).unwrap();
/*        if !self.tick() {
            return
        }*/
        if self.frame_cnt % 8 == 0 {
            if self.board.finished {
                self.load_next_level();
                return
            }
            self.board.tick();
            if self.board.is_robbo_killed() {
                self.reload_level();
                return
            }
            for y in 0..self.board.height {
                for x in 0..self.board.width {
                    let tile = self.board.get_tile((x, y), self.frame_cnt / 8);
                    self.draw_tile(ctx, tile, x as usize, y as usize);
                }
            }
        }
        self.frame_cnt += 1;
    }
}
