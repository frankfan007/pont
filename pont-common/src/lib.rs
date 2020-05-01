use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

use rand::thread_rng;
use rand::seq::SliceRandom;

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ClientMessage {
    CreateRoom(String),
    JoinRoom(String, String),
    Chat(String),
    Play(Vec<(Piece, i32, i32)>),

    /*
    Swap(Vec<Piece>),
    */

    Disconnected,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ServerMessage {
    JoinedRoom {
        room_name: String,
        players: Vec<(String, u32, bool)>,
        active_player: usize,
        board: Vec<((i32, i32), Piece)>,
        pieces: Vec<Piece>,
        remaining: usize,
    },
    UnknownRoom(String),
    Chat {
        from: String,
        message: String,
    },
    Information(String),
    NewPlayer(String),
    PlayerDisconnected(usize),
    PlayerTurn(usize, usize),
    Played(Vec<(Piece, i32, i32)>),
    MoveAccepted(Vec<Piece>),
    MoveRejected,
    PlayerScore {
        index: usize,
        delta: u32,
        total: u32,
    },


    /*
    Players {
        players: Vec<(String, usize)>,
        turn: usize,
    },
    YourTurn,
    NotYourTurn,
    Board(Board), // Used to send the initial board
    Draw(Vec<Piece>),
    InvalidMove(String),
    */
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Shape {
    Clover,
    Star,
    Square,
    Diamond,
    Cross,
    Circle,
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Color {
    Orange,
    Yellow,
    Green,
    Red,
    Blue,
    Purple,
}

pub type Piece = (Shape, Color);

#[derive(Debug, Deserialize, Serialize)]
pub struct Game {
    pub board: HashMap<(i32, i32), Piece>,
    pub bag: Vec<Piece>,
}

impl Game {
    pub fn play(&mut self, ps: &[(Piece, i32, i32)]) -> Option<u32> {
        let mut score = 0;
        for (p, x, y) in ps {
            if self.board.contains_key(&(*x, *y)) {
                return None;
            } else {
                self.board.insert((*x, *y), *p);
                score += 1;
            }
        }
        Some(score)
    }

    pub fn new() -> Game {
        use Color::*;
        use Shape::*;
        let mut bag = Vec::new();
        for c in &[Orange, Yellow, Green, Red, Blue, Purple] {
            for s in &[Clover, Star, Square, Diamond, Cross, Circle] {
                for _ in 0..3 {
                    bag.push((*s, *c));
                }
            }
        }
        bag.shuffle(&mut thread_rng());

        Game {
            board: HashMap::new(), bag
        }
    }

    pub fn deal(&mut self, n: usize) -> HashMap<Piece, usize> {
        let mut out = HashMap::new();
        for _ in 0..n {
            if let Some(p) = self.bag.pop() {
                *out.entry(p).or_insert(0) += 1;
            }
        }
        out
    }

    pub fn exchange(&mut self, pieces: Vec<Piece>) -> Option<Vec<Piece>> {
        if pieces.len() <= self.bag.len() {
            let mut out = Vec::new();
            for _ in 0..pieces.len() {
                out.push(self.bag.pop().unwrap());
            }
            for p in pieces.into_iter() {
                self.bag.push(p);
            }
            self.bag.shuffle(&mut thread_rng());
            Some(out)
        } else {
            None
        }
    }

    fn connected(board: &HashMap<(i32, i32), Piece>) -> bool {
        let mut todo: Vec<(i32, i32)> =
            board.keys().take(1).cloned().collect();

        let mut seen = HashSet::new();
        while let Some(t) = todo.pop() {
            if seen.insert(t) {
                for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)].iter() {
                    let c = (t.0 + dx, t.1 + dy);
                    if board.contains_key(&c) {
                        todo.push(c);
                    }
                }
            }
        }

        seen.len() == board.len()
    }

    pub fn is_linear(board: &HashSet<(i32, i32)>) -> bool {
        let mut x_positions = HashSet::new();
        let mut y_positions = HashSet::new();
        for (x, y) in board.iter() {
            x_positions.insert(x);
            y_positions.insert(y);
        }
        return x_positions.len() == 1 || y_positions.len() == 1;
    }

    // Checks whether the given board is valid,
    // returning a vec of invalid piece locations
    pub fn invalid(board: &HashMap<(i32, i32), Piece>) -> HashSet<(i32, i32)> {
        // The empty board has no invalid pieces, by definition
        if board.is_empty() {
            return HashSet::new();
        }

        // If a board has disconnected components, then it's all invalid
        let todo = board.keys().cloned().collect();
        if !Self::connected(board) {
            return todo;
        }

        let mut checked_h = HashSet::new();
        let mut checked_v = HashSet::new();

        let mut out = HashSet::new();
        let explore = |f: &dyn Fn(i32) -> (i32, i32)| {
            let mut out = Vec::new();
            let mut run = |g: &dyn Fn(i32) -> i32| {
                for i in 0.. {
                    let c = f(g(i));
                    if let Some(piece) = board.get(&c) {
                        out.push((*piece, c));
                    } else {
                        break;
                    }
                }
            };
            run(&|i| i);
            run(&|i| (-i - 1));
            out
        };

        let check = |pieces: &[(Piece, (i32, i32))]| -> bool {
            let mut seen_colors = HashSet::new();
            let mut seen_shapes = HashSet::new();
            let mut seen_pieces = HashSet::new();
            for (piece, _pos) in pieces {
                // Detect duplicate pieces
                if !seen_pieces.insert(*piece) {
                    return false;
                }
                seen_colors.insert(piece.0);
                seen_shapes.insert(piece.1);
            }
            seen_colors.len() == 1 || seen_shapes.len() == 1
        };

        // Check that each row and column contains valid pieces
        for (x, y) in todo.into_iter() {
            if !checked_h.contains(&(x, y)) {
                let row = explore(&|i| (x + i, y));
                for (_, c) in row.iter() {
                    checked_h.insert(*c);
                }
                if !check(&row) {
                    for (_, c) in row.into_iter() {
                        out.insert(c);
                    }
                }
            }
            if !checked_v.contains(&(x, y)) {
                let col = explore(&|i| (x, y + i));
                for (_, c) in col.iter() {
                    checked_v.insert(*c);
                }
                if !check(&col) {
                    for (_, c) in col.into_iter() {
                        out.insert(c);
                    }
                }
            }
        }
        out
    }
}
