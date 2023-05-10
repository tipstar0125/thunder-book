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
use rand::rngs::StdRng;
use rand::Rng;
use std::collections::BinaryHeap;
use std::time::Instant;

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
            elapsed_time * 0.85 >= self.time_threshold
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
    fn new(rng: &mut StdRng) -> Self {
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
    fn init(&mut self, rng: &mut StdRng) {
        for character in self.characters_.iter_mut() {
            character.y_ = rng.gen_range(0, H) as isize;
            character.x_ = rng.gen_range(0, W) as isize;
        }
    }
    fn transition(&mut self, rng: &mut StdRng) {
        let character_id = rng.gen_range(0, CHARACTER_N);
        self.characters_[character_id].y_ = rng.gen_range(0, H) as isize;
        self.characters_[character_id].x_ = rng.gen_range(0, W) as isize;
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

fn randomAction(state: &AutoMoveMazeState, _number: usize, rng: &mut StdRng) -> AutoMoveMazeState {
    let mut now_state = state.clone();
    for character_id in 0..CHARACTER_N {
        let y = rng.gen_range(0, H) as isize;
        let x = rng.gen_range(0, W) as isize;
        now_state.setCharacter(character_id, y, x);
    }

    now_state
}

fn hillClimb(state: &AutoMoveMazeState, number: usize, rng: &mut StdRng) -> AutoMoveMazeState {
    let mut now_state = state.clone();
    now_state.init(rng);
    let mut best_score = now_state.getScore(false);

    for _ in 0..number {
        let mut next_state = now_state.clone();
        next_state.transition(rng);
        let next_score = next_state.getScore(false);
        if next_score > best_score {
            best_score = next_score;
            now_state = next_state;
        }
    }
    now_state
}

fn simulatedAnnealing(
    state: &AutoMoveMazeState,
    number: usize,
    rng: &mut StdRng,
    start_temp: f64,
    end_temp: f64,
) -> AutoMoveMazeState {
    let mut now_state = state.clone();
    now_state.init(rng);
    let mut best_score = now_state.getScore(false);
    let mut now_score = best_score;
    let mut best_state = now_state.clone();

    for i in 0..number {
        let mut next_state = now_state.clone();
        next_state.transition(rng);
        let next_score = next_state.getScore(false);

        let temp = start_temp + (end_temp - start_temp) * (i as f64 / number as f64);
        // next_score >= now_score => next_score - now_score >= 0 => good
        let probability = ((next_score as f64 - now_score as f64) / temp).exp();
        // 0 <= rng.gen::<f64>() <= 1
        if rng.gen::<f64>() < probability {
            now_score = next_score;
            now_state = next_state.clone();
        }
        if next_score > best_score {
            best_score = next_score;
            best_state = next_state;
        }
    }
    best_state
}

type Handler = Box<dyn FnMut(&AutoMoveMazeState, usize, &mut StdRng) -> AutoMoveMazeState>;

fn playGame(ai: &mut (&str, Handler), simulate_number: usize, rng_action: &mut StdRng) {
    #[allow(unused_mut, unused_assignments)]
    let mut seed = rand::thread_rng().gen();
    #[cfg(feature = "seed")]
    {
        seed = 11216848234635351618;
    }
    println!("constructor seed: {}", seed);
    let mut rng_constructor: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(seed);

    let mut state = AutoMoveMazeState::new(&mut rng_constructor);
    state = ai.1(&state, simulate_number, rng_action);
    let score = state.getScore(false);
    println!("Score of {}: {}", ai.0, score);
}

fn testAiScore(
    ai: &mut (&str, Handler),
    game_number: usize,
    constructor_seed: usize,
    simulate_number: usize,
    rng_action: &mut StdRng,
) {
    println!("constructor seed: {}", constructor_seed);
    let mut rng_constructor: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(constructor_seed as u64);
    let mut score_mean = 0.0;

    for _ in 0..game_number {
        let mut state = AutoMoveMazeState::new(&mut rng_constructor);
        state = ai.1(&state, simulate_number, rng_action);
        score_mean += state.getScore(false) as f64;
    }
    score_mean /= game_number as f64;
    println!("Score of {}: {}", ai.0, score_mean);
}

fn single_play(simulate_number: usize, rng_action: &mut StdRng) {
    let start = Instant::now();

    let name = "randomAction";
    let f = |state: &AutoMoveMazeState, simulate_number: usize, rng: &mut StdRng| -> AutoMoveMazeState {
        randomAction(state, simulate_number, rng)
    };

    // let name = "hillClimb";
    // let f = |state: &AutoMoveMazeState, simulate_number: usize, rng: &mut StdRng| -> AutoMoveMazeState {
    //     hillClimb(state, simulate_number, rng)
    // };

    // let name = "simulatedAnnealing";
    // let f = |state: &AutoMoveMazeState, simulate_number: usize, rng: &mut StdRng| -> AutoMoveMazeState {
    //     simulatedAnnealing(state, simulate_number, rng, 500.0, 10.0)
    // };

    playGame(&mut (name, Box::new(f)), simulate_number, rng_action);
    println!(
        "Elapsed time: {}sec",
        start.elapsed().as_millis() as f64 / 1000.0
    );
}

fn repeat_play(simulate_number: usize, rng_action: &mut StdRng) {
    #[allow(unused_mut, unused_assignments)]
    let mut constructor_seed = rand::thread_rng().gen();
    #[cfg(feature = "seed")]
    {
        constructor_seed = 11216848234635351618;
    }

    let game_number = 1000;

    let f_randomAction = |state: &AutoMoveMazeState, simulate_number: usize, rng: &mut StdRng| -> AutoMoveMazeState {
        randomAction(state, simulate_number, rng)
    };
    let f_hillClimb = |state: &AutoMoveMazeState, simulate_number: usize, rng: &mut StdRng| -> AutoMoveMazeState {
        hillClimb(state, simulate_number, rng)
    };
    let f_simulatedAnnealing =
        |state: &AutoMoveMazeState, simulate_number: usize, rng: &mut StdRng| -> AutoMoveMazeState {
            simulatedAnnealing(state, simulate_number, rng, 500.0, 10.0)
        };

    let mut ais: Vec<(&str, Handler)> = vec![
        ("randomAction", Box::new(f_randomAction)),
        ("hillClimb", Box::new(f_hillClimb)),
        ("simulatedAnnealing", Box::new(f_simulatedAnnealing)),
    ];

    for ai in ais.iter_mut() {
        let start = Instant::now();

        testAiScore(
            ai,
            game_number,
            constructor_seed,
            simulate_number,
            rng_action,
        );

        println!(
            "Elapsed time: {}sec",
            start.elapsed().as_millis() as f64 / 1000.0
        );
    }
}

fn main() {
    #[allow(unused_mut, unused_assignments)]
    let mut seed = rand::thread_rng().gen();
    #[cfg(feature = "seed")]
    {
        seed = 11216848234635351618;
    }
    println!("action seed: {}", seed);
    let mut rng_action: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(seed);
    let simulate_number = 10000;

    // single_play(simulate_number, &mut rng_action);
    repeat_play(simulate_number, &mut rng_action);
}
