// src/pathfinding/mod.rs

pub mod astar;
pub mod dstar_lite;
pub mod field_dstar;
pub mod pathfinder_trait;

// Re-export the types so others can write, e.g. `use crate::pathfinding::AStar;`
pub use astar::AStar;
pub use dstar_lite::DStarLite;
pub use field_dstar::FieldDStar;
pub use pathfinder_trait::Pathfinder;

// A common Coord alias (each algorithm uses `(usize, usize)` for grid coords)
pub type Coord = (usize, usize);
