// src/rover.rs 

use std::collections::HashSet;
use crate::pathfinding::{AStar, DStarLite, FieldDStar, Pathfinder, Coord};

#[derive(Clone, PartialEq)]
pub struct RoverState {
    pub pos: Coord,
    pub goal: Coord,
    pub path: Vec<Coord>,
    pub obstacles: HashSet<Coord>,  // Original fixed obstacles (gray)
    pub dynamic_obstacles: Vec<Coord>,  // User-added dynamic obstacles (yellow)
    pub converted_obstacles: HashSet<Coord>, // Converted obstacles (blue)
    pub algorithm: String,
    pub speed: u32,
    pub width: usize,
    pub height: usize,
}

pub struct Rover {
    pub state: RoverState,
    pathfinder: Box<dyn Pathfinder<Coord = Coord>>,
    pub width: usize,
    pub height: usize,
}

impl Rover {
    pub fn new(width: usize, height: usize) -> Self {
        let start = (5, 5);
        let goal = (width - 5, height - 5);
        let rover_state = RoverState {
            pos: start,
            goal,
            path: Vec::new(),
            obstacles: HashSet::new(),
            dynamic_obstacles: Vec::new(),
            converted_obstacles: HashSet::new(),
            algorithm: "D*-Lite".into(),
            speed: 5,
            width,
            height,
        };

        // Start with empty grid
        let grid = vec![vec![false; height]; width];
        let pf: Box<dyn Pathfinder<Coord = Coord>> =
            Box::new(DStarLite::new(grid, start, goal));

        Rover {
            state: rover_state,
            pathfinder: pf,
            width,
            height,
        }
    }

    pub fn clone(&self) -> Self {
        let grid = self.build_grid();
        let pf: Box<dyn Pathfinder<Coord = Coord>> = match self.state.algorithm.as_str() {
            "A*" => Box::new(AStar::new(grid, self.state.pos, self.state.goal)),
            "D*-Lite" => Box::new(DStarLite::new(grid, self.state.pos, self.state.goal)),
            "Field D*" => Box::new(FieldDStar::new(grid, self.state.pos, self.state.goal)),
            _ => Box::new(DStarLite::new(grid, self.state.pos, self.state.goal)),
        };
        
        Rover {
            state: self.state.clone(),
            pathfinder: pf,
            width: self.width,
            height: self.height,
        }
    }

    pub fn clone_state(&self) -> RoverState {
        self.state.clone()
    }

    pub fn set_algorithm(&mut self, algo: &str) {
        let grid = self.build_grid();
        self.state.algorithm = algo.to_string();
        
        // Rebuild pathfinder with current grid state
        self.pathfinder = match algo {
            "A*" => Box::new(AStar::new(grid, self.state.pos, self.state.goal)),
            "D*-Lite" => Box::new(DStarLite::new(grid, self.state.pos, self.state.goal)),
            "Field D*" => Box::new(FieldDStar::new(grid, self.state.pos, self.state.goal)),
            _ => Box::new(DStarLite::new(grid, self.state.pos, self.state.goal)),
        };
    }

    pub fn compute_path_now(&mut self) -> Vec<Coord> {
        // Rebuild pathfinder with current obstacles
        let grid = self.build_grid();
        self.pathfinder = match self.state.algorithm.as_str() {
            "A*" => Box::new(AStar::new(grid, self.state.pos, self.state.goal)),
            "D*-Lite" => Box::new(DStarLite::new(grid, self.state.pos, self.state.goal)),
            "Field D*" => Box::new(FieldDStar::new(grid, self.state.pos, self.state.goal)),
            _ => Box::new(DStarLite::new(grid, self.state.pos, self.state.goal)),
        };

        let path = self
            .pathfinder
            .compute_path(self.state.pos, self.state.goal)
            .unwrap_or_default();
        
        self.state.path = path.clone();
        path
    }

    pub fn build_grid(&self) -> Vec<Vec<bool>> {
        let mut grid = vec![vec![false; self.height]; self.width];
        
        // Mark original static obstacles
        for &(ox, oy) in &self.state.obstacles {
            if ox < self.width && oy < self.height {
                grid[ox][oy] = true;
            }
        }
        
        // Mark converted obstacles 
        for &(ox, oy) in &self.state.converted_obstacles {
            if ox < self.width && oy < self.height {
                grid[ox][oy] = true;
            }
        }
        
        // NOTE: dynamic_obstacles are NOT included in pathfinding grid
        // They are only visual until converted
        
        grid
    }

    pub fn set_obstacles(&mut self, obstacles: Vec<Coord>) {
        self.state.obstacles = obstacles.into_iter().collect();
    }

    pub fn set_position(&mut self, new_pos: Coord) {
        self.state.pos = new_pos;
    }

    pub fn set_goal(&mut self, new_goal: Coord) {
        self.state.goal = new_goal;
    }

    pub fn set_speed(&mut self, s: u32) {
        self.state.speed = s;
    }
}

impl RoverState {
    pub fn grid_width(&self) -> usize {
        self.width
    }
    pub fn grid_height(&self) -> usize {
        self.height
    }
}