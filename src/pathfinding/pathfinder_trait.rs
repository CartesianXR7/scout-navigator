// src/pathfinding/pathfinder_trait.rs
// ------------------------------------
//
// Defines the common interface for A*, D*-Lite and Field D*.

use std::hash::Hash;

/// Every pathfinder works on discrete grid coordinates `(usize, usize)`.
/// This trait requires `Coord: Copy + Eq + Hash`.
///
pub trait Pathfinder {
    /// The grid‐cell coordinate type. In our case, `(usize, usize)`.
    type Coord: Copy + Eq + Hash;

    /// Compute a path from `start` to `goal`, returning `Some(vec_of_coords)` if a path exists,
    /// or `None` if no path can be found.
    fn compute_path(&mut self, start: Self::Coord, goal: Self::Coord) -> Option<Vec<Self::Coord>>;

    /// Inform the algorithm that `coord` is now (un)blocked.
    /// `is_blocked = true` means “place an obstacle at `coord`,” 
    /// `is_blocked = false` means “remove obstacle at `coord`.”
    fn update_obstacle(&mut self, coord: Self::Coord, is_blocked: bool);
}
