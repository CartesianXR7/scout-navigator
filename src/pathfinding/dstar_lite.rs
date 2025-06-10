// src/pathfinding/dstar_lite.rs
// -----------------------------
//
// Full D*-Lite implementation on a 2D boolean grid.
// Constructor: `DStarLite::new(grid, start, goal)`.
// Adapted from Koenig & Likhachev's original 2002 paper.

use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;

use crate::pathfinding::pathfinder_trait::Pathfinder;

/// Shorthand for grid‐cell coordinates.
pub type Coord = (usize, usize);

#[derive(Clone, PartialEq, Eq)]
struct State {
    coord: Coord,
    k: (i64, i64), // Use i64 to avoid floating point comparison issues
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        let (k1a, k2a) = (self.k.0, self.k.1);
        let (k1b, k2b) = (other.k.0, other.k.1);
        if k1a < k1b {
            Ordering::Greater // invert because BinaryHeap is max‐heap
        } else if k1a > k1b {
            Ordering::Less
        } else if k2a < k2b {
            Ordering::Greater
        } else if k2a > k2b {
            Ordering::Less
        } else {
            other.coord.cmp(&self.coord)
        }
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &State) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct DStarLite {
    grid: Vec<Vec<bool>>,
    width: usize,
    height: usize,
    start: Coord,
    goal: Coord,

    g: HashMap<Coord, f64>,
    rhs: HashMap<Coord, f64>,
    km: f64,
    open_list: BinaryHeap<State>,
    neighbors_cache: HashMap<Coord, Vec<Coord>>,
    last_start: Coord,
}

impl DStarLite {
    const INF_COST: f64 = std::f64::INFINITY;

    /// Create a new D*-Lite on `grid`, with given `start` and `goal`.
    pub fn new(grid: Vec<Vec<bool>>, start: Coord, goal: Coord) -> Self {
        let width = grid.len();
        let height = if width > 0 { grid[0].len() } else { 0 };
        let mut g = HashMap::new();
        let mut rhs = HashMap::new();

        // Initialize all nodes' g and rhs to ∞
        for x in 0..width {
            for y in 0..height {
                let c = (x, y);
                rhs.insert(c, Self::INF_COST);
                g.insert(c, Self::INF_COST);
            }
        }
        // The goal's rhs = 0
        rhs.insert(goal, 0.0);

        let mut planner = DStarLite {
            grid,
            width,
            height,
            start,
            goal,
            g,
            rhs,
            km: 0.0,
            open_list: BinaryHeap::new(),
            neighbors_cache: HashMap::new(),
            last_start: start,
        };

        // Build neighbors cache
        planner.build_neighbors_cache();

        // Push the goal into open list with its key
        let goal_key = planner.calculate_key(goal);
        planner.open_list.push(State { coord: goal, k: goal_key });

        planner
    }

    /// Precompute all free‐cell neighbors for quick access
    fn build_neighbors_cache(&mut self) {
        for x in 0..self.width {
            for y in 0..self.height {
                let c = (x, y);
                if x < self.width && y < self.height && !self.grid[x][y] {
                    let mut nbrs = Vec::new();
                    // Up
                    if y > 0 && y.saturating_sub(1) < self.height && !self.grid[x][y - 1] {
                        nbrs.push((x, y - 1));
                    }
                    // Down
                    if y + 1 < self.height && !self.grid[x][y + 1] {
                        nbrs.push((x, y + 1));
                    }
                    // Left
                    if x > 0 && x.saturating_sub(1) < self.width && !self.grid[x - 1][y] {
                        nbrs.push((x - 1, y));
                    }
                    // Right
                    if x + 1 < self.width && !self.grid[x + 1][y] {
                        nbrs.push((x + 1, y));
                    }
                    self.neighbors_cache.insert(c, nbrs);
                }
            }
        }
    }

    /// Heuristic: Manhattan distance (converted to f64)
    fn heuristic(&self, a: Coord, b: Coord) -> f64 {
        let dx = (a.0 as i32 - b.0 as i32).abs() as f64;
        let dy = (a.1 as i32 - b.1 as i32).abs() as f64;
        dx + dy
    }

    /// Cost of moving from `u` to `v`: 1.0 if adjacent, ∞ otherwise
    fn cost(&self, u: Coord, v: Coord) -> f64 {
        if let Some(nbrs) = self.neighbors_cache.get(&u) {
            if nbrs.contains(&v) {
                1.0
            } else {
                Self::INF_COST
            }
        } else {
            Self::INF_COST
        }
    }

    /// Compute Key(u) = (min(g[u],rhs[u]) + h(u,s_start) + km, min(g[u],rhs[u]))
    fn calculate_key(&self, u: Coord) -> (i64, i64) {
        let g_u = *self.g.get(&u).unwrap_or(&Self::INF_COST);
        let rhs_u = *self.rhs.get(&u).unwrap_or(&Self::INF_COST);
        let h = self.heuristic(u, self.start);
        let key1 = ((g_u.min(rhs_u) + h + self.km) * 1000.0) as i64;
        let key2 = (g_u.min(rhs_u) * 1000.0) as i64;
        (key1, key2)
    }

    /// Compute rhs(u) = min_{s' ∈ neighbors(u)} [g(s') + cost(u,s')]
    fn compute_rhs(&self, u: Coord) -> f64 {
        if u == self.goal {
            return 0.0;
        }
        let mut min_rhs = Self::INF_COST;
        if let Some(nbrs) = self.neighbors_cache.get(&u) {
            for &s_prime in nbrs {
                let g_sp = *self.g.get(&s_prime).unwrap_or(&Self::INF_COST);
                let c = self.cost(u, s_prime);
                let tentative = g_sp + c;
                if tentative < min_rhs {
                    min_rhs = tentative;
                }
            }
        }
        min_rhs
    }

    /// Returns all predecessors of `u` (i.e. its neighbors)
    fn predecessors(&self, u: Coord) -> Vec<Coord> {
        self.neighbors_cache.get(&u).cloned().unwrap_or_default()
    }

    /// Update a single vertex `u` in the open list
    fn update_vertex(&mut self, u: Coord) {
        let rhs_u = *self.rhs.get(&u).unwrap_or(&Self::INF_COST);
        let g_u = *self.g.get(&u).unwrap_or(&Self::INF_COST);

        // We do "lazy" removal by simply re‐inserting with a new key if needed.
        if (rhs_u - g_u).abs() > f64::EPSILON {
            let k = self.calculate_key(u);
            self.open_list.push(State { coord: u, k });
        }
        // If rhs == g, then it is "consistent," and we do nothing.
    }

    /// Main D*-Lite loop: repeatedly pop from open_list until top key ≥ key(start)
    fn compute_shortest_path(&mut self) {
        loop {
            if let Some(top) = self.open_list.peek() {
                let k_old = top.k;
                let k_start = self.calculate_key(self.start);
                let rhs_start = *self.rhs.get(&self.start).unwrap_or(&Self::INF_COST);
                let g_start = *self.g.get(&self.start).unwrap_or(&Self::INF_COST);

                if k_old > k_start && (rhs_start - g_start).abs() <= f64::EPSILON {
                    break;
                }
            } else {
                break;
            }

            let state_u = self.open_list.pop().unwrap();
            let u = state_u.coord;
            let k_old = state_u.k;
            let k_new = self.calculate_key(u);
            let g_u = *self.g.get(&u).unwrap_or(&Self::INF_COST);
            let rhs_u = *self.rhs.get(&u).unwrap_or(&Self::INF_COST);

            if k_old < k_new {
                // Reinsert with up‐to‐date key
                self.open_list.push(State { coord: u, k: k_new });
            } else if g_u > rhs_u {
                // Overconsistent => set g[u] = rhs[u], update predecessors
                self.g.insert(u, rhs_u);
                let predecessors = self.predecessors(u);
                for p in predecessors {
                    let rhs_p = self.compute_rhs(p);
                    self.rhs.insert(p, rhs_p);
                    self.update_vertex(p);
                }
            } else {
                // Underconsistent => set g[u] = ∞, update u and predecessors
                self.g.insert(u, Self::INF_COST);
                let rhs_u2 = self.compute_rhs(u);
                self.rhs.insert(u, rhs_u2);
                self.update_vertex(u);
                let predecessors = self.predecessors(u);
                for p in predecessors {
                    let rhs_p = self.compute_rhs(p);
                    self.rhs.insert(p, rhs_p);
                    self.update_vertex(p);
                }
            }
        }
    }

    /// Reconstruct path from start to goal after compute_shortest_path has converged
    fn reconstruct_path(&mut self) -> Option<Vec<Coord>> {
        let mut path = Vec::new();
        let mut current = self.start;
        let rhs_start = *self.rhs.get(&current).unwrap_or(&Self::INF_COST);
        if rhs_start == Self::INF_COST {
            return None;
        }
        path.push(current);
        while current != self.goal {
            // Choose neighbor with min (g(neighbor)+cost(current,neighbor))
            let mut min_val = Self::INF_COST;
            let mut next_cell = None;
            if let Some(nbrs) = self.neighbors_cache.get(&current) {
                for &nbr in nbrs {
                    let g_n = *self.g.get(&nbr).unwrap_or(&Self::INF_COST);
                    let c = self.cost(current, nbr);
                    let val = g_n + c;
                    if val < min_val {
                        min_val = val;
                        next_cell = Some(nbr);
                    }
                }
            }
            if let Some(nx) = next_cell {
                current = nx;
                path.push(current);
            } else {
                return None;
            }
        }
        Some(path)
    }
}

impl Pathfinder for DStarLite {
    type Coord = Coord;

    fn compute_path(&mut self, start: Coord, goal: Coord) -> Option<Vec<Coord>> {
        if start != self.last_start {
            self.km += self.heuristic(self.last_start, start);
            self.last_start = start;
            // For each predecessor of old start
            let old_s = self.start;
            let preds = self.neighbors_cache.get(&old_s).cloned().unwrap_or_default();
            for nbr in preds {
                let rhs_n = self.compute_rhs(nbr);
                self.rhs.insert(nbr, rhs_n);
                self.update_vertex(nbr);
            }
        }

        self.start = start;
        self.goal = goal;
        self.rhs.insert(self.goal, 0.0);

        let goal_key = self.calculate_key(self.goal);
        self.open_list.push(State { coord: self.goal, k: goal_key });

        self.compute_shortest_path();
        self.reconstruct_path()
    }

    fn update_obstacle(&mut self, coord: Coord, is_blocked: bool) {
        let (x, y) = coord;
        if x < self.width && y < self.height {
            self.grid[x][y] = is_blocked;
        }
        let preds = self.predecessors(coord);
        for nbr in preds {
            let rhs_n = self.compute_rhs(nbr);
            self.rhs.insert(nbr, rhs_n);
            self.update_vertex(nbr);
        }
        let rhs_c = self.compute_rhs(coord);
        self.rhs.insert(coord, rhs_c);
        self.update_vertex(coord);
    }
}