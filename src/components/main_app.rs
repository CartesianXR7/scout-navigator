// src/components/main_app.rs - COMPLETE FULL CODE with PROPER LAYER OWNERSHIP

use yew::prelude::*;
use web_sys::window;
use std::collections::HashSet;

use crate::rover::Rover;
use crate::pathfinding::Coord;
use crate::components::controls::Controls;
use crate::components::canvas::Canvas;
use crate::components::help_bubble::HelpBubble;

#[derive(Clone, PartialEq)]
struct JourneyStats {
    start_time: Option<f64>,
    end_time: Option<f64>,
    total_distance: f64,
    reroute_count: u32,
    nodes_visited: u32,
    obstacles_detected: u32,
    path_efficiency: f64,
}

// LAYER 4: SOM (Static Obstacle Map) - OWNS: Blocked coordinates for pathfinding ONLY
#[derive(Clone, PartialEq)]
struct SomLayer {
    original_static_obstacles: HashSet<Coord>,    // User-placed static obstacles
    converted_dob_obstacles: HashSet<Coord>,      // Converted DOB coordinates - PATHFINDING ONLY
}

impl SomLayer {
    fn new() -> Self {
        Self {
            original_static_obstacles: HashSet::new(),
            converted_dob_obstacles: HashSet::new(),
        }
    }
    
    // ONLY PURPOSE: Provide complete blocked coordinates to Rover Layer for pathfinding
    fn get_complete_obstacle_map(&self) -> Vec<Coord> {
        self.original_static_obstacles.union(&self.converted_dob_obstacles).cloned().collect()
    }
    
    // Receive converted DOB coordinate from Layer 1 - COORDINATE ONLY, no display data
    fn add_converted_dob(&mut self, coord: Coord) {
        self.converted_dob_obstacles.insert(coord);
        web_sys::console::log_1(&format!("🗺️ SOM Layer 4: Added blocked coordinate {:?} for pathfinding", coord).into());
    }
    
    // Set initial static obstacles (called on "Find Path")
    fn set_initial_obstacles(&mut self, obstacles: HashSet<Coord>) {
        self.original_static_obstacles = obstacles;
        web_sys::console::log_1(&format!("🗺️ SOM Layer 4: Set {} initial obstacles", self.original_static_obstacles.len()).into());
    }
    
    // Check if cell is occupied (for DOB placement validation)
    fn is_cell_occupied(&self, coord: Coord) -> bool {
        self.original_static_obstacles.contains(&coord) || self.converted_dob_obstacles.contains(&coord)
    }
    
    // Clear converted coordinates (for restart)
    fn clear_converted_dob_obstacles(&mut self) {
        self.converted_dob_obstacles.clear();
    }
}

// LAYER 3: ROVER LAYER - OWNS: All path data (traveled and planned)
#[derive(Clone, PartialEq)]
struct RoverLayer {
    current_position: Coord,
    goal_position: Coord,
    start_position: Coord,
    traveled_path: Vec<Coord>,      // OWNED: Historical path data
    planned_path: Vec<Coord>,       // OWNED: Current planned path data
    algorithm: String,
    is_journey_active: bool,
}

impl RoverLayer {
    fn new(start: Coord, goal: Coord) -> Self {
        Self {
            current_position: start,
            goal_position: goal,
            start_position: start,
            traveled_path: vec![start],
            planned_path: Vec::new(),
            algorithm: "A*".to_string(),
            is_journey_active: false,
        }
    }
    
    // ONLY PURPOSE: Compute NEW planned path using blocked coordinates from SOM Layer
    fn compute_path_from_som(&mut self, obstacle_map: Vec<Coord>) -> bool {
        web_sys::console::log_1(&format!("🤖 Rover Layer 3: Computing COMPLETELY NEW planned path from {:?} to {:?} using {}", 
            self.current_position, self.goal_position, self.algorithm).into());
        web_sys::console::log_1(&format!("🗺️ Using {} SOM obstacles (NO amber DOBs included)", obstacle_map.len()).into());
        web_sys::console::log_1(&format!("📍 Traveled path UNCHANGED: {} steps | Planning NEW path", self.traveled_path.len()).into());
        
        // CRITICAL: Completely clear PLANNED PATH ONLY (traveled path stays intact)
        self.planned_path.clear();
        self.planned_path.shrink_to_fit();
        web_sys::console::log_1(&"🔥 CLEARED planned path (traveled path untouched)".into());
        
        // Minimal goal protection - only if directly blocked by SOM obstacles
        if obstacle_map.contains(&self.goal_position) {
            web_sys::console::log_1(&"❌ Goal is directly blocked by SOM obstacle".into());
            return false;
        }
        
        // First try simple direct path if no SOM obstacles in the way
        if obstacle_map.is_empty() {
            let simple_path = Self::create_simple_direct_path(self.current_position, self.goal_position);
            if !simple_path.is_empty() {
                // COMPLETE REPLACEMENT of PLANNED PATH ONLY
                self.planned_path = simple_path;
                web_sys::console::log_1(&format!("✅ NEW planned path (simple direct) - {} steps | Traveled: {} unchanged", 
                    self.planned_path.len(), self.traveled_path.len()).into());
                return true;
            }
        }
        
        // Create pathfinding rover with SOM obstacle coordinates ONLY
        let mut rover = Rover::new(50, 30);
        rover.set_position(self.current_position);
        rover.set_goal(self.goal_position);
        rover.set_obstacles(obstacle_map.clone());
        rover.set_algorithm(&self.algorithm);
        
        let new_path = rover.compute_path_now();
        
        if new_path.is_empty() {
            // Fallback to simple pathfinding
            let fallback_path = Self::create_greedy_path(self.current_position, self.goal_position, &obstacle_map);
            if !fallback_path.is_empty() {
                // COMPLETE REPLACEMENT of PLANNED PATH ONLY
                self.planned_path = fallback_path;
                web_sys::console::log_1(&format!("✅ NEW planned path (fallback) - {} steps | Traveled: {} unchanged", 
                    self.planned_path.len(), self.traveled_path.len()).into());
                return true;
            }
            
            web_sys::console::log_1(&"❌ Rover Layer 3: All pathfinding methods failed".into());
            return false;
        }
        
        // Minimal validation: just check path starts correctly
        if !new_path.is_empty() && new_path[0] != self.current_position {
            web_sys::console::log_1(&format!("❌ Rover Layer 3: Path validation failed - starts at {:?}, expected {:?}", 
                new_path[0], self.current_position).into());
            return false;
        }
        
        // COMPLETE REPLACEMENT of PLANNED PATH ONLY - traveled path untouched
        self.planned_path = new_path;
        web_sys::console::log_1(&format!("✅ NEW planned path COMPLETE - {} steps: {:?} -> {:?} | Traveled: {} unchanged", 
            self.planned_path.len(), 
            self.planned_path.first().unwrap_or(&(0,0)), 
            self.planned_path.last().unwrap_or(&(0,0)),
            self.traveled_path.len()
        ).into());
        true
    }
    
    // Simple direct path for when there are no obstacles
    fn create_simple_direct_path(start: Coord, goal: Coord) -> Vec<Coord> {
        let mut path = vec![start];
        let mut current = start;
        
        while current != goal {
            let (cx, cy) = current;
            let (gx, gy) = goal;
            
            // Move towards goal
            let next_x = if cx < gx { cx + 1 } else if cx > gx { cx - 1 } else { cx };
            let next_y = if cy < gy { cy + 1 } else if cy > gy { cy - 1 } else { cy };
            
            current = (next_x, next_y);
            path.push(current);
            
            // Safety check to prevent infinite loops
            if path.len() > 1000 {
                break;
            }
        }
        
        path
    }
    
    // Greedy pathfinding that avoids obstacles
    fn create_greedy_path(start: Coord, goal: Coord, obstacles: &[Coord]) -> Vec<Coord> {
        use std::collections::HashSet;
        
        let obstacle_set: HashSet<Coord> = obstacles.iter().cloned().collect();
        let mut path = vec![start];
        let mut current = start;
        
        for _ in 0..1000 { // Limit iterations
            if current == goal {
                break;
            }
            
            let (cx, cy) = current;
            let (gx, gy) = goal;
            
            // Try moving towards goal
            let mut best_next = current;
            let mut best_distance = f64::INFINITY;
            
            // Check all 4 directions
            for (dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
                let next_x = cx as i32 + dx;
                let next_y = cy as i32 + dy;
                
                if next_x >= 0 && next_x < 50 && next_y >= 0 && next_y < 30 {
                    let next_coord = (next_x as usize, next_y as usize);
                    if !obstacle_set.contains(&next_coord) {
                        let distance = ((next_x as f64 - gx as f64).powi(2) + (next_y as f64 - gy as f64).powi(2)).sqrt();
                        if distance < best_distance {
                            best_distance = distance;
                            best_next = next_coord;
                        }
                    }
                }
            }
            
            if best_next == current {
                // Stuck, try to find any free adjacent cell
                for (dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
                    let next_x = cx as i32 + dx;
                    let next_y = cy as i32 + dy;
                    
                    if next_x >= 0 && next_x < 50 && next_y >= 0 && next_y < 30 {
                        let next_coord = (next_x as usize, next_y as usize);
                        if !obstacle_set.contains(&next_coord) {
                            best_next = next_coord;
                            break;
                        }
                    }
                }
            }
            
            if best_next == current {
                break; // No free adjacent cells
            }
            
            current = best_next;
            path.push(current);
        }
        
        if current == goal {
            path
        } else {
            Vec::new() // Failed to reach goal
        }
    }
    
    // ONLY PURPOSE: Update OWNED position and path data with CLEAR separation
    fn execute_movement_step(&mut self) -> bool {
        if self.planned_path.len() < 2 {
            web_sys::console::log_1(&format!("❌ Cannot move - planned path too short: {}", self.planned_path.len()).into());
            return false;
        }
        
        // CRITICAL VALIDATION: Ensure we're following the correct path
        let current_step = self.planned_path[0];
        if current_step != self.current_position {
            web_sys::console::log_1(&format!("⚠️ PATH DESYNC: Expected current position {:?}, but planned path starts at {:?}", 
                self.current_position, current_step).into());
            // Fix the desync by updating the path to start from current position
            if self.planned_path.len() > 1 {
                self.planned_path[0] = self.current_position;
                web_sys::console::log_1(&"🔧 Fixed path desync".into());
            } else {
                web_sys::console::log_1(&"❌ Cannot fix path desync - path too short".into());
                return false;
            }
        }
        
        // Take the next step in the PLANNED path
        let next_position = self.planned_path[1];
        
        // Additional validation: ensure next step is adjacent
        let dx = (self.current_position.0 as i32 - next_position.0 as i32).abs();
        let dy = (self.current_position.1 as i32 - next_position.1 as i32).abs();
        if dx > 1 || dy > 1 {
            web_sys::console::log_1(&format!("❌ INVALID STEP: From {:?} to {:?} - not adjacent (dx={}, dy={})", 
                self.current_position, next_position, dx, dy).into());
            return false;
        }
        
        // Update rover position
        let old_position = self.current_position;
        self.current_position = next_position;
        
        // CRITICAL SEPARATION:
        // 1. Add new position to TRAVELED PATH (historical, immutable, only grows)
        self.traveled_path.push(next_position);
        web_sys::console::log_1(&format!("📍 TRAVELED PATH: Added {:?} (total traveled: {})", 
            next_position, self.traveled_path.len()).into());
        
        // 2. Remove completed step from PLANNED PATH (current plan, shrinks as we move)
        self.planned_path.remove(0);
        web_sys::console::log_1(&format!("🗺️ PLANNED PATH: Removed completed step (remaining planned: {})", 
            self.planned_path.len()).into());
        
        web_sys::console::log_1(&format!("✅ MOVED: {:?} -> {:?} | Traveled: {} | Planned: {}", 
            old_position, self.current_position, self.traveled_path.len(), self.planned_path.len()).into());
        
        // Debug: Log next few steps if available
        if self.planned_path.len() >= 2 {
            web_sys::console::log_1(&format!("🗺️ Next planned steps: {:?} -> {:?}", 
                self.planned_path[0], self.planned_path[1]).into());
        } else if self.planned_path.len() == 1 {
            web_sys::console::log_1(&format!("🏁 Final step in planned path: {:?}", self.planned_path[0]).into());
        }
        
        true
    }
    
    fn has_reached_goal(&self) -> bool {
        self.current_position == self.goal_position
    }
    
    fn set_algorithm(&mut self, algo: &str) {
        self.algorithm = algo.to_string();
        self.planned_path.clear(); // Clear OWNED path data
    }
    
    fn set_goal(&mut self, new_goal: Coord) {
        self.goal_position = new_goal;
        self.planned_path.clear(); // Clear OWNED path data
    }
    
    fn reset_to_start(&mut self, start: Coord) {
        self.start_position = start;
        self.current_position = start;
        self.traveled_path = vec![start]; // Reset OWNED path data
        self.planned_path.clear(); // Clear OWNED path data
        self.is_journey_active = false;
    }
}

// LAYER 1: DOB LAYER - OWNS: All DOB states and display classifications
#[derive(Clone, PartialEq)]
struct DobLayer {
    amber_dobs: Vec<Coord>,                    // OWNED: Active dynamic obstacles (yellow display)
    blue_converted_dobs: HashSet<Coord>,       // OWNED: Converted obstacles (blue display)
}

impl DobLayer {
    fn new() -> Self {
        Self {
            amber_dobs: Vec::new(),
            blue_converted_dobs: HashSet::new(),
        }
    }
    
    // ONLY PURPOSE: Check proximity and update OWNED DOB classifications
    fn check_proximity_and_convert(&mut self, rover_position: Coord) -> Vec<Coord> {
        let mut converted_coords = Vec::new();
        let mut remaining_amber = Vec::new();
        
        for &dob_coord in &self.amber_dobs {
            let dx = (rover_position.0 as i32 - dob_coord.0 as i32).abs();
            let dy = (rover_position.1 as i32 - dob_coord.1 as i32).abs();
            let distance = dx.max(dy); // Chebyshev distance
            
            if distance <= 2 {
                // Convert from amber to blue - UPDATE OWNED DATA
                self.blue_converted_dobs.insert(dob_coord);
                converted_coords.push(dob_coord);
                web_sys::console::log_1(&format!("🟡→🔵 DOB Layer 1: Converted DOB {:?}", dob_coord).into());
            } else {
                remaining_amber.push(dob_coord);
            }
        }
        
        // Update OWNED amber list
        self.amber_dobs = remaining_amber;
        converted_coords
    }
    
    // ONLY PURPOSE: Manage OWNED DOB data
    fn toggle_dob(&mut self, coord: Coord, som_layer: &SomLayer) -> bool {
        // Check if cell is occupied by static obstacles
        if som_layer.is_cell_occupied(coord) {
            return false; // Cannot place DOB on occupied cell
        }
        
        if let Some(pos) = self.amber_dobs.iter().position(|&c| c == coord) {
            self.amber_dobs.remove(pos);
            web_sys::console::log_1(&format!("🟡 DOB Layer 1: Removed amber DOB {:?}", coord).into());
        } else {
            self.amber_dobs.push(coord);
            web_sys::console::log_1(&format!("🟡 DOB Layer 1: Added amber DOB {:?}", coord).into());
        }
        true
    }
    
    // Add DOB during journey 
    fn add_dob(&mut self, coord: Coord, som_layer: &SomLayer) -> bool {
        if som_layer.is_cell_occupied(coord) || self.amber_dobs.contains(&coord) {
            return false;
        }
        self.amber_dobs.push(coord);
        true
    }
    
    // ONLY PURPOSE: Provide OWNED DOB display data to UI Layer
    fn get_amber_dobs_for_display(&self) -> Vec<Coord> {
        self.amber_dobs.clone()
    }
    
    fn get_blue_dobs_for_display(&self) -> HashSet<Coord> {
        self.blue_converted_dobs.clone()
    }
    
    fn clear_all(&mut self) {
        self.amber_dobs.clear();
        self.blue_converted_dobs.clear();
    }
}

// PROPER 4-Layer execution with CLEAR data ownership and NO conflicts
fn execute_one_cycle(
    som_layer: &UseStateHandle<SomLayer>,
    rover_layer: &UseStateHandle<RoverLayer>,
    dob_layer: &UseStateHandle<DobLayer>,
    journey_stats: &UseStateHandle<JourneyStats>,
    trapped_alert: &UseStateHandle<bool>,
    is_animating: &UseStateHandle<bool>,
) {
    // Clone the actual values from UseStateHandle
    let mut current_rover: RoverLayer = (**rover_layer).clone();
    
    // STEP 1: Check if current coordinates == goal coordinates (TRUE/FALSE)
    web_sys::console::log_1(&format!("🎯 STEP 1: Checking if {:?} == {:?}", 
        current_rover.current_position, current_rover.goal_position).into());
    
    if current_rover.current_position == current_rover.goal_position {
        web_sys::console::log_1(&"🎯 STEP 1: TRUE - Goal reached! STOPPING LOOP".into());
        
        let mut stats: JourneyStats = (**journey_stats).clone();
        stats.end_time = Some(js_sys::Date::now());
        journey_stats.set(stats);
        is_animating.set(false);
        return; // STOP LOOP
    }
    
    web_sys::console::log_1(&"🎯 STEP 1: FALSE - Continue to step 2".into());
    
    // STEP 2: DOB Layer checks proximity and updates OWNED data
    web_sys::console::log_1(&"🟡 STEP 2: DOB Layer checking proximity and converting".into());
    
    let mut current_dob: DobLayer = (**dob_layer).clone();
    let mut current_som: SomLayer = (**som_layer).clone();
    
    // DOB Layer updates its OWNED data and returns newly converted coordinates
    let newly_converted_coords = current_dob.check_proximity_and_convert(current_rover.current_position);
    
    // CRITICAL: If obstacles were detected, STOP movement and recompute path
    let obstacles_detected = !newly_converted_coords.is_empty();
    
    if obstacles_detected {
        web_sys::console::log_1(&format!("🚨 OBSTACLES DETECTED: {} DOBs converted - STOPPING MOVEMENT TO RECOMPUTE", newly_converted_coords.len()).into());
        
        // STEP 3: Send newly converted coordinates to SOM Layer
        for &coord in &newly_converted_coords {
            current_som.add_converted_dob(coord);
        }
        
        // STEP 4: Get updated obstacle map from SOM Layer
        let obstacle_map = current_som.get_complete_obstacle_map();
        web_sys::console::log_1(&format!("🗺️ STEP 4: Retrieved {} total blocked coordinates from SOM", obstacle_map.len()).into());
        
        // STEP 5: FORCE complete path recomputation - NO movement this cycle
        web_sys::console::log_1(&format!("🧠 STEP 5: FORCED PATH RECOMPUTATION from {:?} to {:?}", 
            current_rover.current_position, current_rover.goal_position).into());
        
        let path_computed = current_rover.compute_path_from_som(obstacle_map);
        
        if !path_computed || current_rover.planned_path.len() < 2 {
            web_sys::console::log_1(&"❌ STEP 5 FAILED: No valid path - rover trapped".into());
            trapped_alert.set(true);
            is_animating.set(false);
            return; // STOP LOOP
        }
        
        web_sys::console::log_1(&format!("✅ STEP 5 SUCCESS: NEW path computed - {} steps, next: {:?}", 
            current_rover.planned_path.len(), current_rover.planned_path.get(1).unwrap_or(&(0,0))).into());
        
        // NO MOVEMENT THIS CYCLE - just update layers with new path
        dob_layer.set(current_dob);
        som_layer.set(current_som);
        rover_layer.set(current_rover.clone());
        
        // Update stats for obstacle detection
        let mut stats: JourneyStats = (**journey_stats).clone();
        stats.obstacles_detected += newly_converted_coords.len() as u32;
        stats.reroute_count += 1;
        journey_stats.set(stats);
        
        web_sys::console::log_1(&format!("🛑 CYCLE COMPLETE: Path recomputed for rover at {:?}, NO movement this cycle", current_rover.current_position).into());
        return;
    }
    
    // STEP 6: NO obstacles detected - proceed with normal movement
    web_sys::console::log_1(&"🚶 STEP 6: No obstacles detected - proceeding with movement".into());
    
    // Validate we have a valid path for movement
    if current_rover.planned_path.len() < 2 {
        web_sys::console::log_1(&format!("❌ STEP 6 ABORT: Path too short for movement: {}", current_rover.planned_path.len()).into());
        trapped_alert.set(true);
        is_animating.set(false);
        return;
    }
    
    // Execute movement step
    let next_step = current_rover.planned_path[1];
    web_sys::console::log_1(&format!("🚶 STEP 6: Taking step to {:?}", next_step).into());
    
    let old_position = current_rover.current_position;
    let movement_success = current_rover.execute_movement_step();
    
    if !movement_success || current_rover.current_position == old_position {
        web_sys::console::log_1(&"❌ STEP 6 FAILED: Movement unsuccessful".into());
        trapped_alert.set(true);
        is_animating.set(false);
        return; // STOP LOOP
    }
    
    web_sys::console::log_1(&format!("🚶 STEP 6 COMPLETE: Moved to {:?}, remaining path: {}", 
        current_rover.current_position, current_rover.planned_path.len()).into());
    
    // STEP 7: Update layers with movement data
    web_sys::console::log_1(&"⏹️ STEP 7: Updating layers with movement data".into());
    
    rover_layer.set(current_rover.clone());
    
    // STEP 8: Update stats for movement
    let mut stats: JourneyStats = (**journey_stats).clone();
    stats.nodes_visited += 1;
    stats.total_distance += 1.0;
    journey_stats.set(stats);
    
    web_sys::console::log_1(&format!("✅ STEP 7 COMPLETE: Movement cycle complete - rover at {:?}", current_rover.current_position).into());
}

#[function_component(MainApp)]
pub fn main_app() -> Html {
    web_sys::console::log_1(&"🏗️ Scout Pathfinder: COMPLETE with PROPER LAYER DATA OWNERSHIP".into());
    
    let grid_width = 50usize;
    let grid_height = 30usize;
    
    // === THE 4 LAYERS with CLEAR ownership ===
    let som_layer = use_state(|| SomLayer::new());           // OWNS: Blocked coordinates
    let rover_layer = use_state(|| RoverLayer::new((5, 5), (45, 25))); // OWNS: Path data
    let dob_layer = use_state(|| DobLayer::new());           // OWNS: DOB states and display
    // UI Layer (Layer 2) OWNS: Nothing - only displays data from other layers
    
    // UI Control State
    let is_computing = use_state(|| false);
    let is_animating = use_state(|| false);
    let path_computed = use_state(|| false);
    let is_panel_minimized = use_state(|| false);
    let show_help = use_state(|| true);
    let is_dark = use_state(|| false);
    let trapped_alert = use_state(|| false);
    let current_speed = use_state(|| 5u32);
    
    let visual_start = use_state(|| (5, 5)); // Visual marker stays put
    
    let journey_stats = use_state(|| JourneyStats {
        start_time: None,
        end_time: None,
        total_distance: 0.0,
        reroute_count: 0,
        nodes_visited: 0,
        obstacles_detected: 0,
        path_efficiency: 100.0,
    });

    // === PURE SYNCHRONOUS JOURNEY EXECUTION ===
    // When rover position changes AND animation is active, execute one cycle
    {
        let som_layer = som_layer.clone();
        let rover_layer = rover_layer.clone();
        let dob_layer = dob_layer.clone();
        let journey_stats = journey_stats.clone();
        let trapped_alert = trapped_alert.clone();
        let is_animating = is_animating.clone();
        let current_speed = current_speed.clone();
        
        use_effect_with(
            ((*rover_layer).current_position, *is_animating, *current_speed, (*dob_layer).amber_dobs.len()),
            move |(rover_position, is_active, speed, dob_count)| {
                if !*is_active {
                    return;
                }
                
                // CRITICAL SAFETY CHECKS to prevent infinite execution
                let current_stats = (*journey_stats).clone();
                if current_stats.nodes_visited > 1000 {
                    web_sys::console::log_1(&"🛑 Safety stop - too many steps".into());
                    is_animating.set(false);
                    return;
                }
                
                // Check if rover has a valid path before proceeding
                let current_rover_state = (*rover_layer).clone();
                if current_rover_state.current_position == current_rover_state.goal_position {
                    web_sys::console::log_1(&"🎯 Goal reached - stopping animation".into());
                    is_animating.set(false);
                    return;
                }
                
                // Safety check: don't execute if no path and no obstacles to convert
                if current_rover_state.planned_path.len() < 2 && *dob_count == 0 {
                    web_sys::console::log_1(&"🛑 No valid path and no obstacles to process - stopping".into());
                    trapped_alert.set(true);
                    is_animating.set(false);
                    return;
                }
                
                web_sys::console::log_1(&format!("🔄 CYCLE TRIGGER: Rover at {:?}, speed {}, DOBs: {}, path_len: {}", 
                    rover_position, speed, dob_count, current_rover_state.planned_path.len()).into());
                
                // Immediately log current DOB state to debug
                let debug_dob = (*dob_layer).clone();
                web_sys::console::log_1(&format!("🟡 PRE-CYCLE DOB CHECK: {} amber DOBs: {:?}", 
                    debug_dob.amber_dobs.len(), debug_dob.amber_dobs).into());
                
                // Calculate delay based on speed (1-10 scale)
                // Speed 1 = 1000ms, Speed 5 = 500ms, Speed 10 = 100ms
                let delay_ms = 1100 - (*speed as u32 * 100);
                
                let timeout = gloo_timers::callback::Timeout::new(delay_ms, move || {
                    // Execute one complete synchronous cycle
                    execute_one_cycle(
                        &som_layer,
                        &rover_layer,
                        &dob_layer,
                        &journey_stats,
                        &trapped_alert,
                        &is_animating,
                    );
                });
                timeout.forget();
            }
        );
    }

    // Mouse interaction state
    let is_dragging = use_state(|| false);
    let drag_mode = use_state(|| false);
    let last_drag_cell = use_state(|| None::<Coord>);

    // Dark mode effect
    {
        let is_dark = is_dark.clone();
        use_effect_with(
            *is_dark,
            move |is_dark| {
                if let Some(window) = window() {
                    if let Some(document) = window.document() {
                        if let Some(body) = document.body() {
                            if *is_dark {
                                body.class_list().add_1("dark").unwrap();
                            } else {
                                body.class_list().remove_1("dark").unwrap();
                            }
                        }
                    }
                }
            }
        );
    }

    // === LAYER INTERACTION CALLBACKS ===

    let on_compute = {
        let som_layer = som_layer.clone();
        let rover_layer = rover_layer.clone();
        let is_computing = is_computing.clone();
        let path_computed = path_computed.clone();

        Callback::from(move |_| {
            web_sys::console::log_1(&"🧠 COMPUTE PATH: Creating initial planned path".into());
            is_computing.set(true);

            let current_som = (*som_layer).clone();
            let mut current_rover = (*rover_layer).clone();
            
            // Clear planned path only
            current_rover.planned_path.clear();
            current_rover.planned_path.shrink_to_fit();
            web_sys::console::log_1(&format!("🧹 Cleared planned path | Traveled path: {} steps", 
                current_rover.traveled_path.len()).into());
            
            // SOM Layer provides blocked coordinates to Rover Layer
            let obstacle_map = current_som.get_complete_obstacle_map();
            web_sys::console::log_1(&format!("🗺️ Using {} obstacles from SOM for pathfinding", obstacle_map.len()).into());
            
            let path_found = current_rover.compute_path_from_som(obstacle_map);
            
            if path_found {
                web_sys::console::log_1(&format!("✅ Path computation SUCCESS: {} planned steps | {} traveled steps", 
                    current_rover.planned_path.len(), current_rover.traveled_path.len()).into());
            } else {
                web_sys::console::log_1(&"❌ Path computation FAILED".into());
            }
            
            rover_layer.set(current_rover);
            is_computing.set(false);
            path_computed.set(path_found);
        })
    };

    let on_start_journey = {
        let is_animating = is_animating.clone();
        let trapped_alert = trapped_alert.clone();
        let journey_stats = journey_stats.clone();
        let visual_start = visual_start.clone();
        let rover_layer = rover_layer.clone();

        Callback::from(move |_| {
            web_sys::console::log_1(&"🚀 START JOURNEY CLICKED!".into());

            let mut current_rover = (*rover_layer).clone();
            
            if current_rover.planned_path.is_empty() {
                web_sys::console::log_1(&"❌ Cannot start - no planned path computed".into());
                return;
            }
            
            // CRITICAL: Ensure planned path starts from current position
            if current_rover.planned_path[0] != current_rover.current_position {
                web_sys::console::log_1(&format!("🔧 Fixing planned path start: {:?} -> {:?}", 
                    current_rover.planned_path[0], current_rover.current_position).into());
                current_rover.planned_path[0] = current_rover.current_position;
            }
            
            web_sys::console::log_1(&format!("🚀 Starting journey | Traveled: {} steps | Planned: {} steps: {:?} -> {:?}", 
                current_rover.traveled_path.len(),
                current_rover.planned_path.len(),
                current_rover.planned_path.first().unwrap_or(&(0,0)),
                current_rover.planned_path.last().unwrap_or(&(0,0))
            ).into());
            
            // Initialize journey state
            trapped_alert.set(false);
            visual_start.set(current_rover.start_position);
            
            journey_stats.set(JourneyStats {
                start_time: Some(js_sys::Date::now()),
                end_time: None,
                total_distance: 0.0,
                reroute_count: 0,
                nodes_visited: 1,
                obstacles_detected: 0,
                path_efficiency: 100.0,
            });

            // Update rover state
            rover_layer.set(current_rover);

            web_sys::console::log_1(&"🚀 Journey initialized - starting movement execution".into());
            
            // START EXECUTION by setting animation flag
            is_animating.set(true);
        })
    };

    let on_pause = {
        let is_animating = is_animating.clone();
        Callback::from(move |_| {
            web_sys::console::log_1(&"⏸️ EMERGENCY STOP: Journey paused by user".into());
            is_animating.set(false);
        })
    };

    let on_algo_change = {
        let rover_layer = rover_layer.clone();
        let path_computed = path_computed.clone();

        Callback::from(move |alg_str: String| {
            web_sys::console::log_1(&format!("🔄 ALGORITHM CHANGE: Switching to {}", alg_str).into());
            
            let mut current_rover = (*rover_layer).clone();
            current_rover.set_algorithm(&alg_str);
            rover_layer.set(current_rover);
            path_computed.set(false);
            
            web_sys::console::log_1(&format!("✅ Algorithm changed to: {}", alg_str).into());
        })
    };

    let on_speed_change = {
        let current_speed = current_speed.clone();
        Callback::from(move |new_speed: u32| {
            current_speed.set(new_speed);
        })
    };

    let on_mouse_down = {
        let som_layer = som_layer.clone();
        let rover_layer = rover_layer.clone();
        let dob_layer = dob_layer.clone();
        let is_dragging = is_dragging.clone();
        let drag_mode = drag_mode.clone();
        let last_drag_cell = last_drag_cell.clone();
        let is_animating = is_animating.clone();
        let visual_start = visual_start.clone();
        let path_computed = path_computed.clone();

        Callback::from(move |coord: Coord| {
            web_sys::console::log_1(&format!("🖱️ MOUSE DOWN at {:?} - Animation: {}", coord, *is_animating).into());
            
            let current_rover = (*rover_layer).clone();
            let current_som = (*som_layer).clone();
            let current_dob = (*dob_layer).clone();
            
            // Don't place obstacles on key positions
            if coord == *visual_start || coord == current_rover.goal_position || coord == current_rover.current_position {
                web_sys::console::log_1(&format!("❌ Cannot place at {:?} - protected position", coord).into());
                return;
            }

            is_dragging.set(true);
            last_drag_cell.set(Some(coord));
            
            if *is_animating {
                // DURING JOURNEY: DOB Layer manages OWNED DOB data
                web_sys::console::log_1(&format!("🟡 JOURNEY MODE: DOB operation at {:?}", coord).into());
                web_sys::console::log_1(&format!("🟡 Current DOB state: {} amber DOBs", current_dob.amber_dobs.len()).into());
                
                let mut updated_dob = current_dob.clone();
                
                // Check if cell is already occupied by static obstacles
                if current_som.is_cell_occupied(coord) {
                    web_sys::console::log_1(&format!("❌ Cannot place DOB at {:?} - cell occupied by static obstacle", coord).into());
                    return;
                }
                
                // Check if DOB already exists at this location
                let already_has_dob = updated_dob.amber_dobs.contains(&coord);
                web_sys::console::log_1(&format!("🟡 DOB exists at {:?}: {}", coord, already_has_dob).into());
                
                if already_has_dob {
                    // Remove existing DOB
                    updated_dob.amber_dobs.retain(|&c| c != coord);
                    web_sys::console::log_1(&format!("🟡 REMOVED amber DOB at {:?} - total: {}", coord, updated_dob.amber_dobs.len()).into());
                    drag_mode.set(false); // Removing mode
                } else {
                    // Add new DOB
                    updated_dob.amber_dobs.push(coord);
                    web_sys::console::log_1(&format!("🟡 ADDED amber DOB at {:?} - total: {}", coord, updated_dob.amber_dobs.len()).into());
                    drag_mode.set(true); // Adding mode
                }
                
                web_sys::console::log_1(&format!("🟡 Setting DOB layer with {} amber DOBs: {:?}", 
                    updated_dob.amber_dobs.len(), updated_dob.amber_dobs).into());
                dob_layer.set(updated_dob);
            } else {
                // BEFORE JOURNEY: SOM Layer manages static obstacles
                web_sys::console::log_1(&format!("🔵 SETUP MODE: Adding static obstacle at {:?}", coord).into());
                let mut updated_som = current_som;
                let has_static = updated_som.original_static_obstacles.contains(&coord);
                drag_mode.set(!has_static);
                
                if has_static {
                    updated_som.original_static_obstacles.remove(&coord);
                } else {
                    updated_som.original_static_obstacles.insert(coord);
                }
                
                som_layer.set(updated_som);
                path_computed.set(false);
            }
        })
    };

    let on_mouse_move = {
        let som_layer = som_layer.clone();
        let rover_layer = rover_layer.clone();
        let dob_layer = dob_layer.clone();
        let is_dragging = is_dragging.clone();
        let drag_mode = drag_mode.clone();
        let last_drag_cell = last_drag_cell.clone();
        let is_animating = is_animating.clone();
        let visual_start = visual_start.clone();
        let path_computed = path_computed.clone();

        Callback::from(move |coord: Coord| {
            if !*is_dragging || Some(coord) == *last_drag_cell {
                return;
            }

            let current_rover = (*rover_layer).clone();
            let current_som = (*som_layer).clone();
            let current_dob = (*dob_layer).clone();
            
            if coord == *visual_start || coord == current_rover.goal_position || coord == current_rover.current_position {
                return;
            }

            last_drag_cell.set(Some(coord));

            if *is_animating {
                // DURING JOURNEY: DOB Layer drag operations
                web_sys::console::log_1(&format!("🟡 MOUSE DRAG: DOB operation at {:?} (mode: {})", coord, if *drag_mode { "ADD" } else { "REMOVE" }).into());
                
                let mut updated_dob = current_dob;
                
                // Check if cell is already occupied by static obstacles
                if current_som.is_cell_occupied(coord) {
                    return;
                }
                
                let has_amber = updated_dob.amber_dobs.contains(&coord);
                
                if *drag_mode && !has_amber {
                    // Adding DOBs during drag
                    updated_dob.amber_dobs.push(coord);
                    web_sys::console::log_1(&format!("🟡 Dragged amber DOB added at {:?} - total: {}", coord, updated_dob.amber_dobs.len()).into());
                } else if !*drag_mode && has_amber {
                    // Removing DOBs during drag
                    updated_dob.amber_dobs.retain(|&c| c != coord);
                    web_sys::console::log_1(&format!("🟡 Dragged amber DOB removed at {:?} - total: {}", coord, updated_dob.amber_dobs.len()).into());
                }
                
                dob_layer.set(updated_dob);
            } else {
                // BEFORE JOURNEY: SOM Layer static obstacle drag operations
                let mut updated_som = current_som;
                let has_static = updated_som.original_static_obstacles.contains(&coord);
                
                if *drag_mode && !has_static {
                    updated_som.original_static_obstacles.insert(coord);
                } else if !*drag_mode && has_static {
                    updated_som.original_static_obstacles.remove(&coord);
                }
                
                som_layer.set(updated_som);
                path_computed.set(false);
            }
        })
    };

    let on_mouse_up = {
        let is_dragging = is_dragging.clone();
        let last_drag_cell = last_drag_cell.clone();
        Callback::from(move |_| {
            is_dragging.set(false);
            last_drag_cell.set(None);
        })
    };

    let on_start_drag = {
        let rover_layer = rover_layer.clone();
        let path_computed = path_computed.clone();
        let visual_start = visual_start.clone();

        Callback::from(move |new_pos: Coord| {
            let mut updated_rover = (*rover_layer).clone();
            updated_rover.reset_to_start(new_pos);
            rover_layer.set(updated_rover);
            path_computed.set(false);
            visual_start.set(new_pos);
        })
    };

    let on_goal_drag = {
        let rover_layer = rover_layer.clone();
        let path_computed = path_computed.clone();

        Callback::from(move |new_goal: Coord| {
            let mut updated_rover = (*rover_layer).clone();
            updated_rover.set_goal(new_goal);
            rover_layer.set(updated_rover);
            path_computed.set(false);
        })
    };

    let on_reset = {
        let som_layer = som_layer.clone();
        let rover_layer = rover_layer.clone();
        let dob_layer = dob_layer.clone();
        let path_computed = path_computed.clone();
        let is_animating = is_animating.clone();
        let show_help = show_help.clone();
        let journey_stats = journey_stats.clone();
        let visual_start = visual_start.clone();
        let trapped_alert = trapped_alert.clone();

        Callback::from(move |_| {
            web_sys::console::log_1(&"🔄 RESET: All layers cleared".into());
            
            is_animating.set(false);
            path_computed.set(false);
            show_help.set(true);
            trapped_alert.set(false);
            visual_start.set((5, 5));
            
            som_layer.set(SomLayer::new());
            rover_layer.set(RoverLayer::new((5, 5), (45, 25)));
            dob_layer.set(DobLayer::new());
            
            journey_stats.set(JourneyStats {
                start_time: None,
                end_time: None,
                total_distance: 0.0,
                reroute_count: 0,
                nodes_visited: 0,
                obstacles_detected: 0,
                path_efficiency: 100.0,
            });
        })
    };

    let on_restart = {
        let rover_layer = rover_layer.clone();
        let dob_layer = dob_layer.clone();
        let som_layer = som_layer.clone();
        let path_computed = path_computed.clone();
        let is_animating = is_animating.clone();
        let visual_start = visual_start.clone();
        let journey_stats = journey_stats.clone();
        let trapped_alert = trapped_alert.clone();

        Callback::from(move |_| {
            is_animating.set(false);
            trapped_alert.set(false);
            path_computed.set(false);
            
            let start_pos = *visual_start;
            
            // Each layer resets its OWNED data
            let mut updated_rover = (*rover_layer).clone();
            updated_rover.reset_to_start(start_pos);
            rover_layer.set(updated_rover);
            
            dob_layer.set(DobLayer::new());
            
            let mut updated_som = (*som_layer).clone();
            updated_som.clear_converted_dob_obstacles();
            som_layer.set(updated_som);
            
            journey_stats.set(JourneyStats {
                start_time: None,
                end_time: None,
                total_distance: 0.0,
                reroute_count: 0,
                nodes_visited: 0,
                obstacles_detected: 0,
                path_efficiency: 100.0,
            });
        })
    };

    let on_toggle_panel = {
        let is_panel_minimized = is_panel_minimized.clone();
        Callback::from(move |_| {
            is_panel_minimized.set(!*is_panel_minimized);
        })
    };

    let on_toggle_dark = {
        let is_dark = is_dark.clone();
        Callback::from(move |_| {
            is_dark.set(!*is_dark);
        })
    };

    let on_close_help = {
        let show_help = show_help.clone();
        Callback::from(move |_| {
            show_help.set(false);
        })
    };

    // === LAYER 2: UI VISUALIZATION LAYER - ONLY DISPLAYS DATA FROM OWNING LAYERS ===
    let current_som = (*som_layer).clone();
    let current_rover = (*rover_layer).clone();
    let current_dob = (*dob_layer).clone();
    
    // CRITICAL: DOB Layer separation - rover system NEVER sees amber DOBs
    // Only converted DOBs (blue) are passed via SOM layer for pathfinding
    let display_rover_state = crate::rover::RoverState {
        pos: current_rover.current_position,
        goal: current_rover.goal_position,
        path: current_rover.planned_path.clone(),                    // FROM Rover Layer (OWNER)
        obstacles: current_som.original_static_obstacles.clone(),    // FROM SOM Layer (OWNER)
        dynamic_obstacles: Vec::new(),                               // NEVER pass amber DOBs to rover system!
        converted_obstacles: current_dob.get_blue_dobs_for_display(), // FROM DOB Layer (OWNER) - only converted
        algorithm: current_rover.algorithm.clone(),
        speed: *current_speed,
        width: 50,
        height: 30,
    };
    
    let visual_start_pos = *visual_start;
    let stats = (*journey_stats).clone();

    html! {
        <>
            <div class="app-container">
                <div class={format!("main-content {}", if *is_panel_minimized { "panel-minimized" } else { "" })}>
                    <Controls
                        on_compute={on_compute}
                        on_start_journey={on_start_journey}
                        on_pause={on_pause}
                        on_reset={on_reset}
                        on_restart={on_restart}
                        on_algo_change={Callback::noop()} 
                        on_speed_change={on_speed_change}
                        on_toggle_panel={on_toggle_panel}
                        current_algorithm={"D*-Lite".to_string()} // FORCED: Only show D*-Lite
                        current_speed={*current_speed}
                        is_computing={*is_computing}
                        is_animating={*is_animating}
                        path_computed={*path_computed}
                        is_panel_minimized={*is_panel_minimized}
                    />
                    <div class="canvas-container">
                        <Canvas
                            width={grid_width}
                            height={grid_height}
                            rover_state={display_rover_state}
                            visual_start={visual_start_pos}
                            traveled_path={current_rover.traveled_path.clone()}  // FROM Rover Layer (OWNER)
                            amber_dobs={current_dob.get_amber_dobs_for_display()} 
                            on_mouse_down={on_mouse_down}
                            on_mouse_move={on_mouse_move}
                            on_mouse_up={on_mouse_up}
                            on_start_drag={on_start_drag}
                            on_goal_drag={on_goal_drag}
                        />
                    </div>
                    {if *show_help {
                        html! {
                            <HelpBubble on_close={on_close_help} />
                        }
                    } else {
                        html! {}
                    }}
                </div>
                
                {if *trapped_alert {
                    html! {
                        <div class="trapped-alert">
                            <span class="alert-icon">{ "⚠️" }</span>
                            <span class="alert-text">{ "Rover is blocked! Direct path to goal cannot be achieved." }</span>
                            <button 
                                class="alert-close"
                                onclick={Callback::from(move |_| trapped_alert.set(false))}
                            >
                                { "×" }
                            </button>
                        </div>
                    }
                } else {
                    html! {}
                }}
                
                <div class="stats-bar">
                    <div class="stats-content">
                        {
                            if let (Some(start), Some(end)) = (stats.start_time, stats.end_time) {
                                let duration = (end - start) / 1000.0;
                                let avg_speed = if duration > 0.0 { stats.nodes_visited as f64 / duration } else { 0.0 };
                                html! {
                                    <div class="stats-complete">
                                        <span class="stat-item">{ "🎉 Complete!" }</span>
                                        <span class="stat-item">{ format!("⏱️ {:.1}s", duration) }</span>
                                        <span class="stat-item">{ format!("📏 {:.1} cells", stats.total_distance) }</span>
                                        <span class="stat-item">{ format!("🔄 {} reroutes", stats.reroute_count) }</span>
                                        <span class="stat-item">{ format!("🚧 {} obstacles detected", stats.obstacles_detected) }</span>
                                        <span class="stat-item">{ format!("📍 {} nodes", stats.nodes_visited) }</span>
                                        <span class="stat-item">{ format!("⚡ {:.1} n/s", avg_speed) }</span>
                                        <span class="stat-item">{ format!("📊 {:.0}% efficiency", stats.path_efficiency) }</span>
                                    </div>
                                }
                            } else if stats.start_time.is_some() && *is_animating {
                                let elapsed = (js_sys::Date::now() - stats.start_time.unwrap()) / 1000.0;
                                html! {
                                    <div class="stats-traveling">
                                        <span class="stat-item">{ "🚀 Traveling" }</span>
                                        <span class="stat-item">{ format!("⏱️ {:.1}s", elapsed) }</span>
                                        <span class="stat-item">{ format!("📏 {:.1} cells", stats.total_distance) }</span>
                                        <span class="stat-item">{ format!("🔄 {} reroutes", stats.reroute_count) }</span>
                                        <span class="stat-item">{ format!("🚧 {} detected", stats.obstacles_detected) }</span>
                                        <span class="stat-item">{ format!("📍 {} nodes", stats.nodes_visited) }</span>
                                    </div>
                                }
                            } else {
                                html! {
                                    <div class="stats-idle">
                                        <span class="stat-item">{ "🎯 Ready!" }</span>
                                        <span class="stat-item">{ format!("🧭 {}", current_rover.algorithm) }</span>
                                        <span class="stat-item">{ format!("🏃 Speed: {}", *current_speed) }</span>
                                        <span class="stat-item">{ "👆 Click 'Find Path' → 'Start Journey'" }</span>
                                    </div>
                                }
                            }
                        }
                    </div>
                    <button 
                        class="dark-mode-toggle-footer"
                        onclick={on_toggle_dark}
                        aria_label="Toggle dark mode"
                    >
                        { if *is_dark { "☀️" } else { "🌙" } }
                    </button>
                </div>
            </div>
        </>
    }
}