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
use std::collections::BinaryHeap;
use std::time::Instant;

type ScoreType = isize;
const H: usize = 30;
const W: usize = 30;
const END_TURN: usize = 100;
const INF: ScoreType = 1e9 as isize;
const dx: [isize; 4] = [1, -1, 0, 0];
const dy: [isize; 4] = [0, 0, 1, -1];

#[derive(Debug, Clone)]
struct TimeKeeper {
    start_time: std::time::Instant,
    time_threshold: f64,
}

impl TimeKeeper {
    fn new(time_threshold: f64) -> Self {
        TimeKeeper {
            start_time: std::time::Instant::now(),
            time_threshold,
        }
    }
    #[inline]
    fn isTimeOver(&self) -> bool {
        let elapsed_time = self.start_time.elapsed().as_nanos() as f64 * 1e-9;
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

#[derive(Debug, Clone, Eq, PartialEq)]
struct Coord {
    x_: isize,
    y_: isize,
}
impl Coord {
    fn new() -> Self {
        Coord { x_: 0, y_: 0 }
    }
}

#[derive(Debug, Clone, Eq)]
struct MazeState {
    points_: [[usize; W]; H],
    turn_: usize,
    character_: Coord,
    game_score_: usize,
    evaluated_score_: ScoreType,
    first_action_: isize,
}

impl MazeState {
    fn new() -> Self {
        #[allow(unused_assignments)]
        let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(rand::thread_rng().gen());
        #[cfg(feature = "seed")]
        {
            let seed = 12;
            eprintln!("seed: {}", seed);
            rng = rand::SeedableRng::seed_from_u64(seed);
        }

        let mut character_ = Coord::new();
        character_.x_ = rng.gen_range(0, W) as isize;
        character_.y_ = rng.gen_range(0, H) as isize;
        let mut points_ = [[0; W]; H];
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
            first_action_: -1,
        }
    }
    fn isDone(&self) -> bool {
        self.turn_ == END_TURN
    }
    fn advance(&mut self, action: usize) {
        self.character_.x_ += dx[action];
        self.character_.y_ += dy[action];
        let point = &mut self.points_[self.character_.y_ as usize][self.character_.x_ as usize] as *mut usize;
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
            let ty = self.character_.y_ + dy[action];
            let tx = self.character_.x_ + dx[action];
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

impl std::cmp::PartialEq for MazeState {
    fn eq(&self, other: &Self) -> bool {
        self.evaluated_score_ == other.evaluated_score_
    }
}

impl std::cmp::PartialOrd for MazeState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.evaluated_score_ == other.evaluated_score_ {
            Some(std::cmp::Ordering::Equal)
        } else if self.evaluated_score_ > other.evaluated_score_ {
            Some(std::cmp::Ordering::Greater)
        } else if self.evaluated_score_ < other.evaluated_score_ {
            Some(std::cmp::Ordering::Less)
        } else {
            None
        }
    }
}

impl std::cmp::Ord for MazeState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.evaluated_score_ == other.evaluated_score_ {
            std::cmp::Ordering::Equal
        } else if self.evaluated_score_ > other.evaluated_score_ {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Less
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

fn beamSearchAction(state: &MazeState, beam_width: usize, beam_depth: usize) -> usize {
    let mut now_beam = BinaryHeap::new();
    let mut best_state = &MazeState::new(); // initialize
    now_beam.push(state.clone());

    for t in 0..beam_depth {
        let mut next_beam = BinaryHeap::new();
        for _ in 0..beam_width {
            if now_beam.is_empty() {
                break;
            }
            let now_state = now_beam.pop().unwrap();
            let legal_actions = now_state.legalActions();
            for &action in &legal_actions {
                let mut next_state = now_state.clone();
                next_state.advance(action);
                next_state.evaluateScore();
                if t == 0 {
                    next_state.first_action_ = action as isize;
                }
                next_beam.push(next_state);
            }
        }

        now_beam = next_beam;
        best_state = now_beam.peek().unwrap();
        if best_state.isDone() {
            break;
        }
    }
    best_state.first_action_ as usize
}

fn beamSearchActionWithTimeThreshold(state: &MazeState, beam_width: usize, time_threshold: f64) -> usize {
    let mut now_beam = BinaryHeap::new();
    let mut best_state = &MazeState::new(); // initialize
    now_beam.push(state.clone());
    let time_keeper = TimeKeeper::new(time_threshold);

    for t in 0.. {
        let mut next_beam = BinaryHeap::new();
        for _ in 0..beam_width {
            if now_beam.is_empty() {
                break;
            }
            let now_state = now_beam.pop().unwrap();
            let legal_actions = now_state.legalActions();
            for &action in &legal_actions {
                let mut next_state = now_state.clone();
                next_state.advance(action);
                next_state.evaluateScore();
                if t == 0 {
                    next_state.first_action_ = action as isize;
                }
                next_beam.push(next_state);
            }
        }

        now_beam = next_beam;
        best_state = now_beam.peek().unwrap();
        if best_state.isDone() || time_keeper.isTimeOver() {
            break;
        }
    }
    best_state.first_action_ as usize
}

fn chokudaiSearchAction(state: &MazeState, beam_width: usize, beam_depth: usize, beam_number: usize) -> isize {
    let mut beam = vec![BinaryHeap::new(); beam_depth + 1];
    beam[0].push(state.clone());

    for _ in 0..beam_number {
        for t in 0..beam_depth {
            for _ in 0..beam_width {
                if beam[t].is_empty() {
                    break;
                }
                let now_state = beam[t].peek().unwrap().clone();
                if now_state.isDone() {
                    break;
                }
                beam[t].pop();

                let legal_actions = now_state.legalActions();
                for &action in &legal_actions {
                    let mut next_state = now_state.clone();
                    next_state.advance(action);
                    next_state.evaluateScore();
                    if t == 0 {
                        next_state.first_action_ = action as isize;
                    }
                    beam[t + 1].push(next_state);
                }
            }
        }
    }
    for t in (0..=beam_depth).rev() {
        let now_beam = &beam[t];
        if !now_beam.is_empty() {
            return now_beam.peek().unwrap().first_action_;
        }
    }
    -1
}

fn chokudaiSearchActionWithTimeThreshold(
    state: &MazeState,
    beam_width: usize,
    beam_depth: usize,
    time_threshold: f64,
) -> isize {
    let mut beam = vec![BinaryHeap::new(); beam_depth + 1];
    beam[0].push(state.clone());
    let time_keeper = TimeKeeper::new(time_threshold);

    loop {
        for t in 0..beam_depth {
            for _ in 0..beam_width {
                if beam[t].is_empty() {
                    break;
                }
                let now_state = beam[t].peek().unwrap().clone();
                if now_state.isDone() {
                    break;
                }
                beam[t].pop();

                let legal_actions = now_state.legalActions();
                for &action in &legal_actions {
                    let mut next_state = now_state.clone();
                    next_state.advance(action);
                    next_state.evaluateScore();
                    if t == 0 {
                        next_state.first_action_ = action as isize;
                    }
                    beam[t + 1].push(next_state);
                }
            }
        }
        if time_keeper.isTimeOver() {
            break;
        }
    }
    for t in (0..=beam_depth).rev() {
        let now_beam = &beam[t];
        if !now_beam.is_empty() {
            return now_beam.peek().unwrap().first_action_;
        }
    }
    -1
}

fn playGame() -> usize {
    let mut state = MazeState::new();
    // [ms]
    let time_threshold = 10.0;
    // state.toString();
    while !state.isDone() {
        // state.advance(randomAction(&state));
        // state.advance(greedyAction(&state));
        // (state, beam_width, beam_depth)
        // state.advance(beamSearchAction(&state, 5, 3));
        // (state, beam_width, time_threshold[s])
        // state.advance(beamSearchActionWithTimeThreshold(&state, 5, time_threshold * 1e-3));
        // (state, beam_width, beam_depth, beam_number)
        // state.advance(chokudaiSearchAction(&state, 1, 3, 1) as usize);
        // (state, beam_width,beam_depth, time_threshold[s])
        state.advance(chokudaiSearchActionWithTimeThreshold(&state, 1, END_TURN, time_threshold * 1e-3) as usize);
        // state.toString();
    }
    state.game_score_
}

fn testApiScore(game_number: usize) {
    let mut score_mean = 0.0;
    for _ in 0..game_number {
        score_mean += playGame() as f64;
    }
    score_mean /= game_number as f64;
    println!("Score: {:.2}", score_mean);
}

fn main() {
    let start = Instant::now();
    testApiScore(100);
    println!("Elapsed time: {}sec", start.elapsed().as_secs());
}
