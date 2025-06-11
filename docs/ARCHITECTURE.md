```markdown
# Scout — Detailed Architecture

## Layer Overview (visual)

```text
┌─────────────────┐
│ Layer 1 · DOB   │  Dynamic Obstacle Map (amber → blue)
└────┬────────────┘
     ↓
┌─────────────────┐
│ Layer 2 · UI    │  Grid visualisation / user input
└────┬────────────┘
     ↓
┌─────────────────┐
│ Layer 3 · Rover │  Rover state + path planner
└────┬────────────┘
     ↓
┌─────────────────┐
│ Layer 4 · FOM   │  Fixed Obstacle Map (authoritative)
└─────────────────┘


| Layer                              | Purpose                           | Consumes                                                                       | Produces                                             |
| ---------------------------------- | --------------------------------- | ------------------------------------------------------------------------------ | ---------------------------------------------------- |
| **1 · Dynamic Obstacle Map (DOB)** | Manages user-placed obstacles…    | • Mouse clicks (UI)<br>• Rover coordinates (L-3)<br>• Initial SOB layout (L-4) | • New SOB coords → L-4<br>• Amber/blue updates → L-2 |
| **2 · Grid UI / Visualisation**    | Single source of truth…           | Inputs from L-1, L-3, L-4                                                      | Draw commands → browser                              |
| **3 · Rover & Path**               | Keeps rover pose, travelled path… | • Program state<br>• Start/goal nodes + FOM (L-4)<br>• Algorithm impl          | • Pose + paths → L-2<br>• Status → backend           |
| **4 · Fixed Obstacle Map (FOM)**   | Authoritative obstacle list…      | • Initial grid<br>• Converted DOBs (L-1)<br>• Rover requests                   | • SOB list → L-3<br>• Original SOB list → L-2, L-1   |


Dynamic Obstacle Block (DOB) - Represents the cells on the map placed by the user mid-journey that the rover is "unaware" of (amber color) which convert to Static Obstacle Blocks (blue color) if the Rover node comes within a 2-cell proximity on its journey. 

Static Obstacle Block (SOB) - Represents the cells on the map considered obstacles the Rover node must pathfind around and cannot pass through on its journey from the Starting node to the Destination / Goal node.

NOTE - DOBs, once converted via continuous proximity detection, are added to the Fixed Obstacle Map (FOM) of which in essence is a list of known obstacles. The Rover node references the FOM on every step it takes and, employing a user-chosen pathfinding algorithm, re-calculates an ideal path to the goal. This approach allows the Rover node to adhere to a memory-less strategy where particular map elements are segmented away from its path re-calculation loop.


Path Re-Calculation Loop (Memoryless):
1. the Rover node checks if its current coordinates are equal to the coordinates of the Destination / Goal node (boolean TRUE or FALSE)
2. the Rover node calls the grid map / static object map from LAYER 4 (FOM) to retrieve a list of known obstacle coordinates
3. the Rover node charts a new path to the goal from its current position
4. The Rover node uses this new path to take the FIRST step on NEW planned / charted path
5. the rover STOPS moving 
→ IF step 1 in the loop is TRUE then STOP LOOP because we have reached out Destination / Goal node successfully
→ IF step 1 in the loop is FALSE then move to step 2 in the loop
→ Each step CANNOT execute until the step before it has completed.

Directory Overview:
ScoutNav/
├── src/
│   ├── bin/serve.rs            # dev HTTP server (localhost:8000)
│   ├── components/             # Yew UI widgets
│   │   ├── canvas.rs           # WebGL/2-D drawing surface
│   │   ├── controls.rs         # play/pause/algorithm selectors
│   │   ├── help_bubble.rs      # inline docs
│   │   └── main_app.rs         # root <App/>
│   ├── pathfinding/
│   │   ├── astar.rs
│   │   ├── dstar_lite.rs
│   │   ├── field_dstar.rs
│   │   └── pathfinder_trait.rs # common interface
│   ├── rover.rs                # agent FSM: move → scan → update map
│   └── lib.rs                  # wasm-bindgen glue
├── index.html / styles.css     # SPA shell + theming
├── Cargo.toml / Cargo.lock
├── package.json / package-lock.json
├── docs/                       # (demo.gif, architecture.svg etc.)
└── .github/workflows/ci.yml    # GitHub Actions