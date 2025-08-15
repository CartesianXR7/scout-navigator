# scout-navigator
*A live-rerouting path-finding visualiser in Rust + WASM*

[![CI](https://github.com/CartesianXR7/ScoutNav/actions/workflows/ci.yml/badge.svg)](https://github.com/CartesianXR7/ScoutNav/actions/workflows/ci.yml)
[![License: GPL-3.0-or-later](https://img.shields.io/badge/License-GPLv3%2B-blue.svg)](LICENSE) 

> **Scout** shows how **A\***, **D\*-Lite**, and **Field-D\*** continually (re)compute optimal routes while new obstacles appear mid-journey.

---

## Features

| Area | Highlights |
|------|------------|
| Algorithms | A\*, D\*-Lite (default), Field-D\* |
| Dynamic obstacles | User-placed **DOBs** autoconvert to static blocks when the rover is ≤ 2 cells away (Chebyshev) |
| UI | Canvas grid with pan/zoom, dark mode, FPS limiter |
| Build | Pure Rust → `wasm-bindgen` → tiny JS wrapper |
| Dev server | `src/bin/serve.rs` (≈ 80 LOC) – no Node required |

---

## Quick start

    # once per machine
    rustup target add wasm32-unknown-unknown
    cargo install wasm-pack

    # development (watch mode)
    wasm-pack build --target web --out-dir pkg --dev --watch &
    cargo run --bin serve        # → http://localhost:8000

### Production build

    wasm-pack build --target web --out-dir pkg --release

---

## High-level architecture
```bash
Grid Map 4-Layer Architecture
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
```

*(A deep dive lives in [docs/ARCHITECTURE.md](https://github.com/CartesianXR7/scout-navigator/blob/main/docs/ARCHITECTURE.md))*

---

## Directory layout

    src/bin/serve.rs       — mini HTTP server
    src/components/        — Yew UI widgets
    src/pathfinding/       — A*, D*-Lite, Field-D* + trait
    src/rover.rs           — rover state machine
    index.html, styles.css — SPA shell
    Cargo.toml / Cargo.lock
    docs/                  — deep docs (architecture, gifs…)

---

## Tests & CI

* Run `cargo test` locally.  
* GitHub Actions: check → test → `wasm-pack build`.

---

## Contributing

1. `git switch -c feat/my-change`  
2. `cargo fmt && cargo clippy -- -D warnings`  
3. Conventional-commit message (e.g. `feat: add theta*`)  
4. Open a PR – details in **CONTRIBUTING.md**.

---

## License

GPL-3.0-or-later – see **[LICENSE](https://github.com/CartesianXR7/scout-navigator/blob/main/LICENSE)**.

