#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused_imports)]
#![allow(unused_macros)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::comparison_chain)]
#![allow(clippy::nonminimal_bool)]
#![allow(clippy::neg_multiply)]
#![allow(dead_code)]
use itertools::Itertools;
use rand::Rng;
use std::time::Instant;
use std::{
    cmp::{Ord, Ordering, PartialOrd},
    collections::BinaryHeap,
};
type ScoreType = isize;
const H: usize = 5;
const W: usize = 5;
const END_TURN: usize = 5;
const CHARACTER_N: usize = 3;
const INF: ScoreType = 1e9 as isize;
const dx: [isize; 4] = [1, -1, 0, 0];
const dy: [isize; 4] = [0, 0, 1, -1];

#[derive(Debug, Clone)]
struct TimeKeeper {
    start_time: Instant,
    time_threshold: f64, // us
}

impl TimeKeeper {
    fn new(ms: usize) -> Self {
        TimeKeeper {
            start_time: Instant::now(),
            time_threshold: (ms * 1e3 as usize) as f64,
        }
    }
    #[inline]
    fn isTimeOver(&self) -> bool {
        let elapsed_time = self.start_time.elapsed().as_micros() as f64;
        #[cfg(feature = "local")]
        {
            elapsed_time * 0.5 >= self.time_threshold
        }
        #[cfg(not(feature = "local"))]
        {
            elapsed_time >= self.time_threshold
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
struct Coord {
    x_: isize,
    y_: isize,
}
impl Coord {
    fn new() -> Self {
        Coord { x_: 0, y_: 0 }
    }
}

#[derive(Debug, Clone)]
struct AutoMoveMazeState {
    points_: [[usize; W]; H],
    turn_: usize,
    characters_: [Coord; CHARACTER_N],
    game_score_: usize,
    evaluated_score_: ScoreType,
}

impl AutoMoveMazeState {
    fn new() -> Self {
        #[allow(unused_assignments)]
        let mut rng: rand::rngs::StdRng =
            rand::SeedableRng::seed_from_u64(rand::thread_rng().gen());
        #[cfg(feature = "seed")]
        {
            let seed = 12;
            eprintln!("seed: {}", seed);
            rng = rand::SeedableRng::seed_from_u64(seed);
        }

        let mut points_ = [[0; W]; H];
        for y in 0..H {
            for x in 0..W {
                points_[y][x] = rng.gen_range(1, 10);
            }
        }
        let characters_ = [Coord::new(); CHARACTER_N];
        AutoMoveMazeState {
            points_,
            turn_: 0,
            characters_,
            game_score_: 0,
            evaluated_score_: 0,
        }
    }
    fn setCharacter(&mut self, character_id: usize, y: isize, x: isize) {
        self.characters_[character_id].y_ = y;
        self.characters_[character_id].x_ = x;
    }
    fn isDone(&self) -> bool {
        self.turn_ == END_TURN
    }
    fn movePlayer(&mut self, character_id: usize) {
        let character = self.characters_[character_id];
        let mut best_point = -INF;
        let mut best_action_index = 0;
        for action in 0..4 {
            let ty = character.y_ + dy[action];
            let tx = character.x_ + dx[action];
            if ty >= 0 && ty < H as isize && tx >= 0 && tx < W as isize {
                let point = self.points_[ty as usize][tx as usize] as isize;
                if point > best_point {
                    best_point = point;
                    best_action_index = action;
                }
            }
        }

        self.characters_[character_id].y_ += dy[best_action_index];
        self.characters_[character_id].x_ += dx[best_action_index];
    }
    fn advance(&mut self) {
        for character_id in 0..CHARACTER_N {
            self.movePlayer(character_id);
        }
        for character in self.characters_ {
            let point = self.points_[character.y_ as usize][character.x_ as usize];
            self.game_score_ += point;
            self.points_[character.y_ as usize][character.x_ as usize] = 0;
        }
        self.turn_ += 1;
    }
    fn getScore(&self, is_print: bool) -> usize {
        let mut tmp_state = self.clone();
        for character in self.characters_ {
            tmp_state.points_[character.y_ as usize][character.x_ as usize] = 0;
        }
        if is_print {
            tmp_state.toString();
        }

        while !tmp_state.isDone() {
            tmp_state.advance();
            if is_print {
                tmp_state.toString();
            }
        }
        tmp_state.game_score_
    }
    fn toString(&self) {
        println!("turn: {}", self.turn_);
        println!("score: {}", self.game_score_);
        for y in 0..H {
            for x in 0..W {
                let mut character_vec = vec![];
                for (i, &coord) in self.characters_.iter().enumerate() {
                    let cx = coord.x_ as usize;
                    let cy = coord.y_ as usize;
                    if cx == x && cy == y {
                        character_vec.push((b'A' + i as u8) as char);
                    }
                }
                if !character_vec.is_empty() {
                    print!("{} ", character_vec.iter().join(""));
                } else if self.points_[y][x] > 0 {
                    print!("{}", self.points_[y][x]);
                } else {
                    print!(".");
                }
                for _ in 0..CHARACTER_N - character_vec.len() {
                    print!(" ");
                }
            }
            println!();
        }
    }
}

fn randomAction(state: &AutoMoveMazeState) -> AutoMoveMazeState {
    let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(0);
    let mut now_state = state.clone();
    for character_id in 0..CHARACTER_N {
        let y = rng.gen_range(0, H) as isize;
        let x = rng.gen_range(0, W) as isize;
        now_state.setCharacter(character_id, y, x);
    }
    now_state
}

fn playGame(ai: (&str, AutoMoveMazeState)) {
    let state = ai.1;
    let score = state.getScore(true);
    println!("Score of {}: {}", ai.0, score);
}

fn main() {
    let start = Instant::now();
    let state = AutoMoveMazeState::new();
    let ai = ("randomAction", randomAction(&state));
    playGame(ai);
    println!("Elapsed time: {}sec", start.elapsed().as_secs());
}
