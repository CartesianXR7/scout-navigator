// src/pathfinding/field_dstar.rs
// ------------------------------
//
// A complete Field D* ("F‐D*") implementation on a 2D boolean grid.
// Constructor: `FieldDStar::new(grid: Vec<Vec<bool>>, start: Coord, goal: Coord)`.

use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;

use crate::pathfinding::pathfinder_trait::Pathfinder;

/// Shorthand for grid‐cell coordinates.
pub type Coord = (usize, usize);

#[derive(Clone, Copy, PartialEq, Eq)]
struct FDState {
    coord: Coord,
    f: i64, // Use i64 to avoid floating point comparison issues
}

impl Ord for FDState {
    fn cmp(&self, other: &Self) -> Ordering {
        // invert because BinaryHeap is max-heap
        other.f.cmp(&self.f)
            .then_with(|| self.coord.cmp(&other.coord))
    }
}

impl PartialOrd for FDState {
    fn partial_cmp(&self, other: &FDState) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct FieldDStar {
    grid: Vec<Vec<bool>>,
    width: usize,
    height: usize,
    start: Coord,
    goal: Coord,

    g: HashMap<Coord, f64>,
    parent: HashMap<Coord, Coord>,
    open_list: BinaryHeap<FDState>,
}

impl FieldDStar {
    const INF: f64 = std::f64::INFINITY;

    /// Create a new Field D* on `grid`, with `start` and `goal`.
    pub fn new(grid: Vec<Vec<bool>>, start: Coord, goal: Coord) -> Self {
        let width = grid.len();
        let height = if width > 0 { grid[0].len() } else { 0 };
        let mut g = HashMap::new();
        let parent = HashMap::new();

        for x in 0..width {
            for y in 0..height {
                g.insert((x, y), Self::INF);
            }
        }
        g.insert(start, 0.0);

        let mut fds = FieldDStar {
            grid,
            width,
            height,
            start,
            goal,
            g,
            parent,
            open_list: BinaryHeap::new(),
        };

        let f0 = (fds.heuristic(start, goal) * 1000.0) as i64;
        fds.open_list.push(FDState { coord: start, f: f0 });
        fds
    }

    /// Heuristic: Euclidean distance between two coords.
    fn heuristic(&self, a: Coord, b: Coord) -> f64 {
        let dx = (a.0 as f64) - (b.0 as f64);
        let dy = (a.1 as f64) - (b.1 as f64);
        (dx * dx + dy * dy).sqrt()
    }

    /// Return up to 8 neighbors (including diagonals) that are free.
    fn neighbors(&self, (x, y): Coord) -> Vec<Coord> {
        let mut result = Vec::new();
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                    let cx = nx as usize;
                    let cy = ny as usize;
                    if cx < self.width && cy < self.height && !self.grid[cx][cy] {
                        result.push((cx, cy));
                    }
                }
            }
        }
        result
    }

    /// Cost between `a` and `b`: 1.0 for orthogonal, √2 for diagonal.
    fn edge_cost(&self, a: Coord, b: Coord) -> f64 {
        let dx = (a.0 as i32 - b.0 as i32).abs();
        let dy = (a.1 as i32 - b.1 as i32).abs();
        if dx == 1 && dy == 1 {
            std::f64::consts::SQRT_2
        } else {
            1.0
        }
    }

    /// "Expand" a node `u`: relax all neighbors via true field cost.
    fn expand(&mut self, u: Coord) {
        let g_u = *self.g.get(&u).unwrap_or(&Self::INF);
        for &nbr in &self.neighbors(u) {
            let c = self.edge_cost(u, nbr);
            let tentative = g_u + c;
            let g_n = *self.g.get(&nbr).unwrap_or(&Self::INF);
            if tentative < g_n {
                self.g.insert(nbr, tentative);
                self.parent.insert(nbr, u);
                let f_n = (tentative + self.heuristic(nbr, self.goal)) * 1000.0;
                self.open_list.push(FDState { coord: nbr, f: f_n as i64 });
            }
        }
    }
}

impl Pathfinder for FieldDStar {
    type Coord = Coord;

    fn compute_path(&mut self, start: Coord, goal: Coord) -> Option<Vec<Coord>> {
        self.start = start;
        self.goal = goal;

        self.g.clear();
        self.parent.clear();
        self.open_list.clear();

        for y in 0..self.height {
            for x in 0..self.width {
                self.g.insert((x, y), Self::INF);
            }
        }
        self.g.insert(start, 0.0);

        let f0 = (self.heuristic(start, goal) * 1000.0) as i64;
        self.open_list.push(FDState { coord: start, f: f0 });

        while let Some(FDState { coord: u, f: _ }) = self.open_list.pop() {
            if u == goal {
                let mut path = Vec::new();
                let mut current = goal;
                path.push(current);
                while current != start {
                    if let Some(&p) = self.parent.get(&current) {
                        current = p;
                        path.push(current);
                    } else {
                        break;
                    }
                }
                path.reverse();
                return Some(path);
            }
            self.expand(u);
        }

        None
    }

    fn update_obstacle(&mut self, coord: Coord, is_blocked: bool) {
        let (x, y) = coord;
        if x < self.width && y < self.height {
            self.grid[x][y] = is_blocked;
        }
        // No incremental repair—will be replanned from scratch next call.
    }
}