// src/pathfinding/astar.rs

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

use crate::pathfinding::pathfinder_trait::Pathfinder;

pub type Coord = (usize, usize);

#[derive(Clone, Copy, Eq, PartialEq)]
struct Node {
    coord: Coord,
    f_score: usize,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .f_score
            .cmp(&self.f_score)
            .then_with(|| self.coord.cmp(&other.coord))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Node) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct AStar {
    grid: Vec<Vec<bool>>,
    width: usize,
    height: usize,
}

impl AStar {
    pub fn new(grid: Vec<Vec<bool>>, _start: Coord, _goal: Coord) -> Self {
        let width = grid.len();
        let height = if width > 0 { grid[0].len() } else { 0 };
        AStar {
            grid,
            width,
            height,
        }
    }

    fn heuristic(&self, a: Coord, b: Coord) -> usize {
        let dx = if a.0 > b.0 { a.0 - b.0 } else { b.0 - a.0 };
        let dy = if a.1 > b.1 { a.1 - b.1 } else { b.1 - a.1 };
        dx + dy
    }

    fn neighbors(&self, (x, y): Coord) -> Vec<Coord> {
        let mut result = Vec::with_capacity(4);

        // Up
        if y > 0 && x < self.width && (y - 1) < self.height && !self.grid[x][y - 1] {
            result.push((x, y - 1));
        }
        // Down
        if y + 1 < self.height && x < self.width && !self.grid[x][y + 1] {
            result.push((x, y + 1));
        }
        // Left
        if x > 0 && y < self.height && !self.grid[x - 1][y] {
            result.push((x - 1, y));
        }
        // Right
        if x + 1 < self.width && y < self.height && !self.grid[x + 1][y] {
            result.push((x + 1, y));
        }

        result
    }
}

impl Pathfinder for AStar {
    type Coord = Coord;

    fn compute_path(&mut self, start: Coord, goal: Coord) -> Option<Vec<Coord>> {
        let mut open_set = BinaryHeap::new();
        let mut closed_set = HashSet::new();
        let mut came_from: HashMap<Coord, Coord> = HashMap::new();
        let mut g_score: HashMap<Coord, usize> = HashMap::new();
        let mut f_score: HashMap<Coord, usize> = HashMap::new();

        g_score.insert(start, 0);
        f_score.insert(start, self.heuristic(start, goal));
        open_set.push(Node {
            coord: start,
            f_score: f_score[&start],
        });

        while let Some(current_node) = open_set.pop() {
            let current = current_node.coord;

            if current == goal {
                // Reconstruct path
                let mut path = Vec::new();
                let mut cur = goal;
                path.push(cur);

                while let Some(&prev) = came_from.get(&cur) {
                    cur = prev;
                    path.push(cur);
                }

                path.reverse();
                return Some(path);
            }

            closed_set.insert(current);

            for neighbor in self.neighbors(current) {
                if closed_set.contains(&neighbor) {
                    continue;
                }

                let tentative_g = g_score.get(&current).unwrap_or(&usize::MAX) + 1;
                let neighbor_g = *g_score.get(&neighbor).unwrap_or(&usize::MAX);

                if tentative_g < neighbor_g {
                    came_from.insert(neighbor, current);
                    g_score.insert(neighbor, tentative_g);
                    let f = tentative_g + self.heuristic(neighbor, goal);
                    f_score.insert(neighbor, f);

                    open_set.push(Node {
                        coord: neighbor,
                        f_score: f,
                    });
                }
            }
        }

        None
    }

    fn update_obstacle(&mut self, coord: Coord, is_blocked: bool) {
        let (x, y) = coord;
        if x < self.width && y < self.height {
            self.grid[x][y] = is_blocked;
        }
    }
}
