#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused_imports)]
#![allow(unused_macros)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::comparison_chain)]
#![allow(clippy::nonminimal_bool)]
#![allow(clippy::neg_multiply)]
#![allow(dead_code)]
use rand::Rng;
use std::cmp::{Ordering, PartialOrd};
type ScoreType = isize;
const H: usize = 3;
const W: usize = 4;
const END_TURN: usize = 4;
const INF: ScoreType = 1e9 as isize;

#[derive(Debug, Clone, PartialEq)]
struct Coord {
    x_: isize,
    y_: isize,
}
impl Coord {
    fn new() -> Self {
        Coord { x_: 0, y_: 0 }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct MazeState {
    points_: Vec<Vec<usize>>,
    turn_: usize,
    character_: Coord,
    game_score_: usize,
    evaluated_score_: ScoreType,
    first_action: isize,
}

impl MazeState {
    const dx: [isize; 4] = [1, -1, 0, 0];
    const dy: [isize; 4] = [0, 0, 1, -1];
    fn new(seed: Option<u64>) -> Self {
        let mut rng: rand::rngs::StdRng =
            rand::SeedableRng::seed_from_u64(rand::thread_rng().gen());
        if let Some(s) = seed {
            rng = rand::SeedableRng::seed_from_u64(s)
        }

        let mut character_ = Coord::new();
        character_.x_ = rng.gen_range(0, W) as isize;
        character_.y_ = rng.gen_range(0, H) as isize;
        let mut points_ = vec![vec![0; W]; H];
        for y in 0..H {
            for x in 0..W {
                if x as isize == character_.x_ && y as isize == character_.y_ {
                    continue;
                }
                points_[y][x] = rng.gen_range(0, 10);
            }
        }
        MazeState {
            points_,
            turn_: 0,
            character_,
            game_score_: 0,
            evaluated_score_: 0,
            first_action: -1,
        }
    }
    fn isDone(&self) -> bool {
        self.turn_ == END_TURN
    }
    fn advance(&mut self, action: usize) {
        self.character_.x_ += Self::dx[action];
        self.character_.y_ += Self::dy[action];
        let point = &mut self.points_[self.character_.y_ as usize][self.character_.x_ as usize]
            as *mut usize;
        unsafe {
            if *point > 0 {
                self.game_score_ += *point;
                *point = 0;
            }
        }
        self.turn_ += 1;
    }
    fn legalActions(&self) -> Vec<usize> {
        let mut actions = vec![];
        for action in 0..4 {
            let ty = self.character_.y_ + Self::dy[action];
            let tx = self.character_.x_ + Self::dx[action];
            if 0 <= ty && ty < H as isize && 0 <= tx && tx < W as isize {
                actions.push(action);
            }
        }
        actions
    }
    fn toString(&self) {
        println!("turn: {}", self.turn_);
        println!("score: {}", self.game_score_);
        for y in 0..H {
            for x in 0..W {
                if self.character_.y_ == y as isize && self.character_.x_ == x as isize {
                    print!("@");
                } else if self.points_[y][x] > 0 {
                    print!("{}", self.points_[y][x]);
                } else {
                    print!(".");
                }
            }
            println!();
        }
    }
    fn evaluateScore(&mut self) {
        self.evaluated_score_ = self.game_score_ as isize;
    }
}

impl PartialOrd for MazeState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.evaluated_score_ == other.evaluated_score_ {
            Some(Ordering::Equal)
        } else if self.evaluated_score_ > other.evaluated_score_ {
            Some(Ordering::Greater)
        } else if self.evaluated_score_ < other.evaluated_score_ {
            Some(Ordering::Less)
        } else {
            None
        }
    }
}

fn randomAction(state: &MazeState) -> usize {
    let legal_actions = state.legalActions();
    let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(0);
    legal_actions[rng.gen_range(0, legal_actions.len())]
}

fn greedyAction(state: &MazeState) -> usize {
    let legal_actions = state.legalActions();
    let mut best_score = -INF;
    let mut best_action = -1_isize;
    for &action in &legal_actions {
        let mut now_state = state.clone();
        now_state.advance(action);
        now_state.evaluateScore();
        if now_state.evaluated_score_ > best_score {
            best_score = now_state.evaluated_score_;
            best_action = action as isize;
        }
    }
    best_action as usize
}

fn playGame(seed: Option<u64>) -> usize {
    let mut state = MazeState::new(seed);
    state.toString();
    while !state.isDone() {
        // state.advance(randomAction(&state));
        state.advance(greedyAction(&state));
        state.toString();
    }
    state.game_score_
}

fn testApiScore(game_number: usize) {
    let mut score_mean = 0.0;
    for _ in 0..game_number {
        score_mean += playGame(None) as f64;
        // score_mean += playGame(Some(12)) as f64;
    }
    score_mean /= game_number as f64;
    println!("Score: {:.2}", score_mean);
}

fn main() {
    testApiScore(100);
}
