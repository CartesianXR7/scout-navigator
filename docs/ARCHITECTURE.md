```markdown
# Scout — Detailed Architecture

## Layer Overview (visual)

```text
┌───────────────┐
│ Layer 1 · DOB │  Dynamic Obstacle Map (amber → blue)
└────┬──────────┘
     ↓
┌───────────────┐
│ Layer 2 · UI  │  Grid visualisation / user input
└────┬──────────┘
     ↓
┌───────────────┐
│ Layer 3 · Rover│  Rover state + path planner
└────┬──────────┘
     ↓
┌───────────────┐
│ Layer 4 · SOM │  Static Obstacle Map (authoritative)
└───────────────┘


| Layer                              | Purpose                           | Consumes                                                                       | Produces                                             |
| ---------------------------------- | --------------------------------- | ------------------------------------------------------------------------------ | ---------------------------------------------------- |
| **1 · Dynamic Obstacle Map (DOB)** | Manages user-placed obstacles…    | • Mouse clicks (UI)<br>• Rover coordinates (L-3)<br>• Initial SOB layout (L-4) | • New SOB coords → L-4<br>• Amber/blue updates → L-2 |
| **2 · Grid UI / Visualisation**    | Single source of truth…           | Inputs from L-1, L-3, L-4                                                      | Draw commands → browser                              |
| **3 · Rover & Path**               | Keeps rover pose, travelled path… | • Program state<br>• Start/goal nodes + SOM (L-4)<br>• Algorithm impl          | • Pose + paths → L-2<br>• Status → backend           |
| **4 · Static Obstacle Map (SOM)**  | Authoritative obstacle list…      | • Initial grid<br>• Converted DOBs (L-1)<br>• Rover requests                   | • SOB list → L-3<br>• Original SOB list → L-2, L-1   |



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

