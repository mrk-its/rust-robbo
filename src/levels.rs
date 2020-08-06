use log::log;
use std::collections::HashMap;

type AdditionalMap = HashMap<(usize, usize), Vec<u16>>;

pub struct LevelSet {
    pub name: String,
    pub levels: Vec<Level>,
}

impl LevelSet {
    pub fn parse(data: &str) -> LevelSet {
        let mut levels: Vec<Level> = Vec::new();
        let mut level_set_name: Option<&str> = None;
        let mut default_level_color: String = String::from("000000");
        let mut collecting_data: bool = false;
        let mut current_level = Level::new();
        let mut lines = data.split("\n");
        loop {
            let line = lines.next();
            if line.is_none() {
                break;
            }
            let line = line.unwrap();
            if line.starts_with("[") {
                collecting_data = false;
            }
            match line {
                "[level]" => current_level.number = lines.next().unwrap().parse().unwrap(),
                "[name]" => {
                    level_set_name = Some(lines.next().unwrap());
                }
                "[colour]" => {
                    current_level.color = String::from(lines.next().unwrap());
                }
                "[default_level_colour]" => {
                    default_level_color = String::from(lines.next().unwrap());
                }
                "[size]" => {
                    let mut it = lines
                        .next()
                        .unwrap()
                        .split('.')
                        .map(|v| v.parse::<i32>().unwrap());
                    current_level.width = it.next().unwrap();
                    current_level.height = it.next().unwrap();
                }
                "[data]" => {
                    collecting_data = true;
                }
                "[additional]" => {
                    let cnt = lines.next().unwrap().parse::<usize>().unwrap();
                    for _ in 0..cnt {
                        let line = lines.next().unwrap();
                        let parts = line.split('.').collect::<Vec<&str>>();
                        let x = parts[0].parse::<usize>().unwrap();
                        let y = parts[1].parse::<usize>().unwrap();
                        let c = parts[2].chars().next().unwrap();
                        if c != current_level.tiles[y].chars().nth(x).unwrap() {
                            log!(
                                "level:{} additional data mismatch: {}.{}.{}",
                                current_level.number,
                                x,
                                y,
                                c
                            );
                        };
                        let params = parts[3..]
                            .iter()
                            .map(|v| v.parse::<u16>().unwrap())
                            .collect::<Vec<u16>>();
                        current_level.additional.insert((x, y), params);
                    }
                }
                "[end]" => {
                    if current_level.color.len() == 0 {
                        current_level.color = default_level_color.clone();
                    }
                    levels.push(current_level);
                    current_level = Level::new();
                }
                _ => {
                    if collecting_data {
                        current_level.tiles.push(String::from(line));
                    }
                }
            }
        }

        LevelSet {
            name: String::from(level_set_name.unwrap()),
            levels: levels,
        }
    }
    pub fn size(&self) -> usize {
        self.levels.len()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Level {
    pub number: usize,
    pub width: i32,
    pub height: i32,
    pub color: String,
    pub tiles: Vec<String>,
    pub additional: AdditionalMap,
}

impl Level {
    pub fn new() -> Level {
        Level {
            number: 0,
            width: 16,
            height: 35,
            color: String::from("000000"),
            tiles: vec![],
            additional: AdditionalMap::new(),
        }
    }
}
