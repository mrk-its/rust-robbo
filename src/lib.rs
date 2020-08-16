extern crate cfg_if;
extern crate js_sys;
extern crate wasm_bindgen;
extern crate web_sys;
extern crate rand;

#[macro_use]
mod log;
mod board;
mod consts;
mod items;
mod levels;
mod sound;
mod types;
mod utils;
mod tiles;
use board::Board;
use cfg_if::cfg_if;
use levels::LevelSet;
use log::log;
use utils::{modulo, set_panic_hook};
use wasm_bindgen::prelude::*;

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
    current_level: usize,
    level_set: LevelSet,
    board: Board,
}

/// Public methods, exported to JavaScript.
#[wasm_bindgen]
impl Universe {
    pub fn reload_level(&mut self) {
        let level = &self.level_set.levels[self.current_level];
        log(&format!("{:#?}", level));
        self.board = Board::from(level);
    }

    pub fn kill_robbo(&mut self) {
        self.board.kill_robbo();
    }

    pub fn prev_level(&mut self) {
        self.current_level = modulo(
            self.current_level as i32 - 1,
            self.level_set.size() as i32,
        ) as usize;
        self.reload_level();
    }
    pub fn next_level(&mut self) {
        self.current_level =
            (self.current_level + 1) % self.level_set.size();
        self.reload_level();
    }

    pub fn toggle_god_mode(&mut self) {
        self.board.god_mode();
    }
    pub fn toggle_god_mode2(&mut self) {
        self.board.god_mode2();
    }

    pub fn robbo_move(&mut self, kx: i32, ky: i32) {
        self.board.robbo_move_or_shot((kx, ky), false)
    }

    pub fn robbo_shot(&mut self, kx: i32, ky: i32) {
        self.board.robbo_move_or_shot((kx, ky), true)
    }

    pub fn new(level_data: String, current_level: usize) -> Universe {
        set_panic_hook();
        let level_set = LevelSet::parse(&level_data);
        let board = Board::from(&level_set.levels[current_level]);

        Universe {
            current_level,
            level_set,
            board,
        }
    }

    pub fn get_current_level(&self) -> usize {
        self.current_level
    }

    pub fn get_inventory(&self) -> String {
        let inventory = &self.board.robbo.inventory;
        format!(
            "level: {:02} screws: {:02} keys: {:02} bullets: {:02}",
            self.current_level + 1,
            self.board.missing_screws - inventory.screws,
            inventory.keys,
            inventory.bullets
        )
    }

    pub fn load_next_level(&mut self) {
        self.current_level += 1;
        if self.current_level >= self.level_set.size() {
            self.current_level = 0;
        }
        self.reload_level();
    }

    pub fn tick(&mut self) {
        if self.board.finished {
            self.load_next_level();
            return;
        }
        self.board.tick();
        if self.board.is_robbo_killed() {
            self.reload_level();
        }
    }
    pub fn get_tile(&self, x: i32, y: i32) -> usize {
        self.board.get_tile((x, y))
    }
    pub fn get_board_width(&self) -> i32 {
        self.board.width
    }
    pub fn get_board_height(&self) -> i32 {
        self.board.height
    }
    pub fn get_sounds(&mut self) -> Vec<i16> {
        self.board.get_sounds().iter().map(|v| *v as i16).collect()
    }
}
