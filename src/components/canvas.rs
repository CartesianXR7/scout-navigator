// src/components/canvas.rs

use crate::pathfinding::Coord;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlBodyElement, HtmlCanvasElement, MouseEvent};
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub enum DragMode {
    None,
    PlacingObstacles,
    MovingStart,
    MovingGoal,
}

#[derive(Properties, PartialEq)]
pub struct CanvasProps {
    pub width: usize,
    pub height: usize,
    pub rover_state: crate::rover::RoverState,
    pub visual_start: Coord,       // Visual start marker position
    pub traveled_path: Vec<Coord>, // Turquoise path
    pub amber_dobs: Vec<Coord>,    // Amber DOBs for display
    pub on_mouse_down: Callback<Coord>,
    pub on_mouse_move: Callback<Coord>,
    pub on_mouse_up: Callback<()>,
    pub on_start_drag: Callback<Coord>,
    pub on_goal_drag: Callback<Coord>,
}

#[function_component(Canvas)]
pub fn canvas(props: &CanvasProps) -> Html {
    let canvas_ref = use_node_ref();
    let drag_mode = use_state(|| DragMode::None);
    let animation_frame = use_state(|| 0i32);

    // Dynamic cell size based on container - responsive to window resize
    let cell_size = use_state(|| 20.0f64);

    // Calculate cell size based on container - responsive to resize
    {
        let canvas_ref = canvas_ref.clone();
        let cell_size = cell_size.clone();
        let width = props.width;
        let height = props.height;

        use_effect_with((), move |_| {
            let update_size = {
                let canvas_ref = canvas_ref.clone();
                let cell_size = cell_size.clone();
                move || {
                    if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                        if let Some(parent) = canvas.parent_element() {
                            let parent_width = parent.client_width() as f64 - 40.0;
                            let parent_height = parent.client_height() as f64 - 40.0;

                            let cell_w = parent_width / width as f64;
                            let cell_h = parent_height / height as f64;
                            let new_cell_size = cell_w.min(cell_h).min(25.0).max(10.0);

                            cell_size.set(new_cell_size);
                        }
                    }
                }
            };

            // Initial size calculation
            update_size();

            // Add resize event listener
            let window = web_sys::window().unwrap();
            let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                update_size();
            }) as Box<dyn Fn()>);

            window
                .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())
                .unwrap();

            // Return cleanup function
            move || {
                if let Some(window) = web_sys::window() {
                    let _ = window.remove_event_listener_with_callback(
                        "resize",
                        closure.as_ref().unchecked_ref(),
                    );
                }
                drop(closure);
            }
        });
    }

    // Animation timer for pulsing effect
    {
        let animation_frame = animation_frame.clone();
        use_effect_with((), move |_| {
            let interval = gloo_timers::callback::Interval::new(50, move || {
                animation_frame.set((*animation_frame + 1) % 360);
            });

            move || drop(interval)
        });
    }

    // Main rendering effect - separate from animation
    {
        let canvas_ref = canvas_ref.clone();
        let rover_state = props.rover_state.clone();
        let visual_start = props.visual_start;
        let traveled_path = props.traveled_path.clone();
        let amber_dobs = props.amber_dobs.clone();
        let width = props.width;
        let height = props.height;
        let cell_size_val = *cell_size;
        let animation_frame = animation_frame.clone();

        use_effect_with(
            (
                rover_state.clone(),
                cell_size_val,
                traveled_path.clone(),
                amber_dobs.clone(),
            ),
            move |_| {
                let render = move || {
                    let frame = *animation_frame;

                    if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                        let cell_size = cell_size_val;

                        // Set canvas size
                        let w_px = (width as f64) * cell_size;
                        let h_px = (height as f64) * cell_size;
                        canvas.set_width(w_px as u32);
                        canvas.set_height(h_px as u32);

                        // Get 2D context
                        let context = canvas
                            .get_context("2d")
                            .unwrap()
                            .unwrap()
                            .dyn_into::<web_sys::CanvasRenderingContext2d>()
                            .unwrap();

                        // Clear background
                        let doc = window().unwrap().document().unwrap();
                        let body = doc.body().unwrap();
                        let body_element = body.dyn_into::<HtmlBodyElement>().unwrap();
                        let is_dark = body_element.class_list().contains("dark");

                        let bg_color = if is_dark { "#0a0a0a" } else { "#fafafa" };
                        context.set_fill_style_with_str(bg_color);
                        context.fill_rect(0.0, 0.0, w_px, h_px);

                        // Draw grid lines
                        let grid_color = if is_dark { "#1f1f1f" } else { "#e5e7eb" };
                        context.set_stroke_style_with_str(grid_color);
                        context.set_line_width(0.5);

                        for i in 0..=width {
                            let x = (i as f64) * cell_size + 0.5;
                            context.begin_path();
                            context.move_to(x, 0.0);
                            context.line_to(x, h_px);
                            context.stroke();
                        }

                        for j in 0..=height {
                            let y = (j as f64) * cell_size + 0.5;
                            context.begin_path();
                            context.move_to(0.0, y);
                            context.line_to(w_px, y);
                            context.stroke();
                        }

                        // LAYER 1: Draw original static obstacles (gray)
                        let obstacle_color = if is_dark { "#3f3f46" } else { "#52525b" };
                        context.set_fill_style_with_str(obstacle_color);
                        for &(ox, oy) in &rover_state.obstacles {
                            if ox < width && oy < height {
                                let x = (ox as f64) * cell_size;
                                let y = (oy as f64) * cell_size;
                                context.fill_rect(
                                    x + 1.0,
                                    y + 1.0,
                                    cell_size - 2.0,
                                    cell_size - 2.0,
                                );
                            }
                        }

                        // LAYER 2: Draw amber DOBs (yellow - undiscovered obstacles)
                        let amber_dob_color = if is_dark { "#d97706" } else { "#f59e0b" };
                        context.set_fill_style_with_str(amber_dob_color);
                        for &(ox, oy) in &amber_dobs {
                            if ox < width && oy < height {
                                let x = (ox as f64) * cell_size;
                                let y = (oy as f64) * cell_size;
                                context.fill_rect(
                                    x + 1.0,
                                    y + 1.0,
                                    cell_size - 2.0,
                                    cell_size - 2.0,
                                );
                            }
                        }

                        // LAYER 3: Draw converted obstacles (blue - discovered obstacles)
                        let converted_obstacle_color = if is_dark { "#2563eb" } else { "#3b82f6" };
                        context.set_fill_style_with_str(converted_obstacle_color);
                        for &(ox, oy) in &rover_state.converted_obstacles {
                            if ox < width && oy < height {
                                let x = (ox as f64) * cell_size;
                                let y = (oy as f64) * cell_size;
                                context.fill_rect(
                                    x + 1.0,
                                    y + 1.0,
                                    cell_size - 2.0,
                                    cell_size - 2.0,
                                );
                            }
                        }

                        // LAYER 4: Draw TURQUOISE traveled path (from visual start to current rover position)
                        if !traveled_path.is_empty() {
                            // Turquoise path line
                            context.set_stroke_style_with_str("#14b8a6");
                            context.set_line_width(3.0);
                            context.set_line_cap("round");
                            context.set_line_join("round");
                            context.begin_path();

                            for (i, &(x, y)) in traveled_path.iter().enumerate() {
                                let px = (x as f64) * cell_size + (cell_size / 2.0);
                                let py = (y as f64) * cell_size + (cell_size / 2.0);

                                if i == 0 {
                                    context.move_to(px, py);
                                } else {
                                    context.line_to(px, py);
                                }
                            }
                            context.stroke();

                            // Turquoise path dots
                            context.set_fill_style_with_str("#0d9488");
                            for &(x, y) in traveled_path.iter().skip(1) {
                                let px = (x as f64) * cell_size + (cell_size / 2.0);
                                let py = (y as f64) * cell_size + (cell_size / 2.0);

                                context.begin_path();
                                context
                                    .arc(px, py, 3.0, 0.0, std::f64::consts::PI * 2.0)
                                    .unwrap();
                                context.fill();
                            }
                        }

                        // LAYER 5: Draw PURPLE future path (from current rover position to goal)
                        if !rover_state.path.is_empty() && rover_state.path.len() > 1 {
                            context.set_stroke_style_with_str("#a855f7");
                            context.set_line_width(3.0);
                            context.set_line_cap("round");
                            context.set_line_join("round");
                            context.begin_path();

                            let start_idx = 0;
                            for (i, &(x, y)) in rover_state.path[start_idx..].iter().enumerate() {
                                let px = (x as f64) * cell_size + (cell_size / 2.0);
                                let py = (y as f64) * cell_size + (cell_size / 2.0);

                                if i == 0 {
                                    context.move_to(px, py);
                                } else {
                                    context.line_to(px, py);
                                }
                            }
                            context.stroke();

                            // Purple path dots
                            context.set_fill_style_with_str("#9333ea");
                            if rover_state.path.len() > 2 {
                                for &(x, y) in
                                    rover_state.path[1..rover_state.path.len() - 1].iter()
                                {
                                    let px = (x as f64) * cell_size + (cell_size / 2.0);
                                    let py = (y as f64) * cell_size + (cell_size / 2.0);

                                    context.begin_path();
                                    context
                                        .arc(px, py, 3.0, 0.0, std::f64::consts::PI * 2.0)
                                        .unwrap();
                                    context.fill();
                                }
                            }
                        }

                        // LAYER 6: Draw visual start position (green) - stays in original position
                        let (start_x, start_y) = visual_start;
                        if start_x < width && start_y < height {
                            let x = (start_x as f64) * cell_size;
                            let y = (start_y as f64) * cell_size;

                            context.set_fill_style_with_str("#16a34a");
                            context.fill_rect(x + 2.0, y + 2.0, cell_size - 4.0, cell_size - 4.0);

                            // Text
                            context.set_fill_style_with_str("#FFFFFF");
                            context.set_font("bold 11px -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif");
                            context.set_text_align("center");
                            context
                                .fill_text("S", x + cell_size / 2.0, y + cell_size / 2.0 + 4.0)
                                .unwrap();
                        }

                        // LAYER 7: Draw goal position (red)
                        let (goal_x, goal_y) = rover_state.goal;
                        if goal_x < width && goal_y < height {
                            let x = (goal_x as f64) * cell_size;
                            let y = (goal_y as f64) * cell_size;

                            context.set_fill_style_with_str("#dc2626");
                            context.fill_rect(x + 2.0, y + 2.0, cell_size - 4.0, cell_size - 4.0);

                            // Text
                            context.set_fill_style_with_str("#FFFFFF");
                            context.set_font("bold 11px -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif");
                            context.set_text_align("center");
                            context
                                .fill_text("G", x + cell_size / 2.0, y + cell_size / 2.0 + 4.0)
                                .unwrap();
                        }

                        // LAYER 8: Draw rover with circular orange detection range
                        let (rx, ry) = rover_state.pos;
                        if rx < width && ry < height {
                            let cx = (rx as f64) * cell_size + (cell_size / 2.0);
                            let cy = (ry as f64) * cell_size + (cell_size / 2.0);

                            // Pulsing circular detection range in orange (2 cells radius)
                            let time = (frame as f64) * 0.02;
                            let pulse = (time.sin() * 0.3 + 0.7).max(0.1);

                            // Draw circular detection range
                            context.save();

                            // Set shadow for glow effect
                            context.set_shadow_color("rgba(251, 146, 60, 0.5)");
                            context.set_shadow_blur(15.0);

                            // Main detection circle (2 cells radius)
                            context.set_stroke_style_with_str(&format!(
                                "rgba(251, 146, 60, {})",
                                pulse * 0.8
                            ));
                            context.set_line_width(3.0);
                            context.begin_path();
                            context
                                .arc(cx, cy, 2.0 * cell_size, 0.0, std::f64::consts::PI * 2.0)
                                .unwrap();
                            context.stroke();

                            // Inner circle for visual effect
                            context.set_stroke_style_with_str(&format!(
                                "rgba(251, 146, 60, {})",
                                pulse * 0.5
                            ));
                            context.set_line_width(2.0);
                            context.begin_path();
                            context
                                .arc(cx, cy, 1.5 * cell_size, 0.0, std::f64::consts::PI * 2.0)
                                .unwrap();
                            context.stroke();

                            context.restore();

                            // Rover body (khaki brown)
                            context.set_fill_style_with_str("#8b7355");
                            context.begin_path();
                            context
                                .arc(cx, cy, 10.0, 0.0, std::f64::consts::PI * 2.0)
                                .unwrap();
                            context.fill();

                            // Inner circle
                            context.set_fill_style_with_str("#a0926b");
                            context.begin_path();
                            context
                                .arc(cx, cy, 6.0, 0.0, std::f64::consts::PI * 2.0)
                                .unwrap();
                            context.fill();

                            // Inner highlight
                            context.set_fill_style_with_str("rgba(255, 255, 255, 0.4)");
                            context.begin_path();
                            context
                                .arc(cx - 2.0, cy - 2.0, 2.0, 0.0, std::f64::consts::PI * 2.0)
                                .unwrap();
                            context.fill();
                        }
                    }
                };

                // Initial render
                render();

                // Set up animation loop
                let render_loop = gloo_timers::callback::Interval::new(50, move || {
                    render();
                });

                move || drop(render_loop)
            },
        );
    }

    // Mouse event handlers (unchanged)
    let width = props.width;
    let height = props.height;
    let cell_size_val = *cell_size;
    let rover_state = props.rover_state.clone();
    let visual_start = props.visual_start;

    let onmousedown = {
        let canvas_ref = canvas_ref.clone();
        let drag_mode = drag_mode.clone();
        let on_mouse_down = props.on_mouse_down.clone();
        let on_start_drag = props.on_start_drag.clone();
        let on_goal_drag = props.on_goal_drag.clone();
        let goal_pos = rover_state.goal;

        Callback::from(move |e: MouseEvent| {
            e.prevent_default();

            if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                let rect = canvas.get_bounding_client_rect();
                let x = e.client_x() as f64 - rect.left();
                let y = e.client_y() as f64 - rect.top();

                let cell_x = (x / cell_size_val).floor() as usize;
                let cell_y = (y / cell_size_val).floor() as usize;

                if cell_x < width && cell_y < height {
                    if (cell_x, cell_y) == visual_start {
                        drag_mode.set(DragMode::MovingStart);
                        on_start_drag.emit((cell_x, cell_y));
                    } else if (cell_x, cell_y) == goal_pos {
                        drag_mode.set(DragMode::MovingGoal);
                        on_goal_drag.emit((cell_x, cell_y));
                    } else {
                        drag_mode.set(DragMode::PlacingObstacles);
                        on_mouse_down.emit((cell_x, cell_y));
                    }
                }
            }
        })
    };

    let onmousemove = {
        let canvas_ref = canvas_ref.clone();
        let drag_mode = drag_mode.clone();
        let on_mouse_move = props.on_mouse_move.clone();
        let on_start_drag = props.on_start_drag.clone();
        let on_goal_drag = props.on_goal_drag.clone();

        Callback::from(move |e: MouseEvent| {
            if *drag_mode == DragMode::None {
                return;
            }

            if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                let rect = canvas.get_bounding_client_rect();
                let x = e.client_x() as f64 - rect.left();
                let y = e.client_y() as f64 - rect.top();

                let cell_x = (x / cell_size_val).floor() as usize;
                let cell_y = (y / cell_size_val).floor() as usize;

                if cell_x < width && cell_y < height {
                    match *drag_mode {
                        DragMode::PlacingObstacles => on_mouse_move.emit((cell_x, cell_y)),
                        DragMode::MovingStart => on_start_drag.emit((cell_x, cell_y)),
                        DragMode::MovingGoal => on_goal_drag.emit((cell_x, cell_y)),
                        DragMode::None => {}
                    }
                }
            }
        })
    };

    let onmouseup = {
        let drag_mode = drag_mode.clone();
        let on_mouse_up = props.on_mouse_up.clone();
        Callback::from(move |_: MouseEvent| {
            drag_mode.set(DragMode::None);
            on_mouse_up.emit(());
        })
    };

    let onmouseleave = {
        let drag_mode = drag_mode.clone();
        let on_mouse_up = props.on_mouse_up.clone();
        Callback::from(move |_: MouseEvent| {
            drag_mode.set(DragMode::None);
            on_mouse_up.emit(());
        })
    };

    html! {
        <canvas
            ref={canvas_ref}
            onmousedown={onmousedown}
            onmousemove={onmousemove}
            onmouseup={onmouseup}
            onmouseleave={onmouseleave}
            style="display: block; border-radius: 12px; box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06); cursor: crosshair;"
        />
    }
}
