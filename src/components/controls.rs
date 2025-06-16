// src/components/controls.rs

use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ControlsProps {
    pub on_compute: Callback<()>,
    pub on_start_journey: Callback<()>,
    pub on_pause: Callback<()>,
    pub on_reset: Callback<()>,
    pub on_restart: Callback<()>,
    pub on_algo_change: Callback<String>,
    pub on_speed_change: Callback<u32>,
    pub current_algorithm: String,
    pub current_speed: u32,
    pub is_computing: bool,
    pub is_animating: bool,
    pub path_computed: bool,
    pub on_toggle_panel: Callback<()>,
    pub is_panel_minimized: bool,
}

#[function_component(Controls)]
pub fn controls(props: &ControlsProps) -> Html {
    let on_compute = props.on_compute.clone();
    let on_start_journey = props.on_start_journey.clone();
    let on_pause = props.on_pause.clone();
    let on_reset = props.on_reset.clone();
    let on_restart = props.on_restart.clone();
    let on_algo_change = props.on_algo_change.clone();
    let on_speed_change = props.on_speed_change.clone();
    let on_toggle_panel = props.on_toggle_panel.clone();
    let current_speed = props.current_speed;
    let current_algorithm = props.current_algorithm.clone();
    let is_computing = props.is_computing;
    let is_animating = props.is_animating;
    let path_computed = props.path_computed;
    let is_panel_minimized = props.is_panel_minimized;

    // Handler for algorithm dropdown
    let on_change_algo = Callback::from(move |e: Event| {
        let select = e
            .target()
            .unwrap()
            .dyn_into::<HtmlSelectElement>()
            .expect("should be a select element");
        let alg_str = select.value();
        on_algo_change.emit(alg_str);
    });

    // Handler for speed slider
    let on_change_speed = Callback::from(move |e: InputEvent| {
        if let Some(target) = e.target() {
            if let Ok(input) = target.dyn_into::<HtmlInputElement>() {
                if let Ok(val) = input.value().parse::<u32>() {
                    on_speed_change.emit(val);
                }
            }
        }
    });

    // Button states
    let find_path_text = if is_computing {
        "Computing..."
    } else {
        "Find Path"
    };

    let start_journey_text = if is_animating {
        "Traveling..."
    } else {
        "Start Journey"
    };

    let find_path_disabled = is_computing || is_animating;
    let start_journey_disabled = !path_computed || is_computing || is_animating;
    let pause_button_disabled = !is_animating;

    html! {
        <div class={format!("controls-panel {}", if is_panel_minimized { "minimized" } else { "" })}>
            <div class="panel-header">
                <h3>{ "Scout Pathfinder" }</h3>
                <div class="header-controls">
                    <button
                        class="toggle-btn"
                        onclick={Callback::from(move |_| on_toggle_panel.emit(()))}
                    >
                        { if is_panel_minimized { "▼" } else { "▲" } }
                    </button>
                </div>
            </div>

            {if !is_panel_minimized {
                html! {
                    <>
                        <div class="controls-section">
                            <div class="button-grid">
                                <button
                                    class={format!("btn btn-primary {}", if find_path_disabled { "disabled" } else { "" })}
                                    onclick={if find_path_disabled { Callback::noop() } else { Callback::from(move |_| on_compute.emit(())) }}
                                    disabled={find_path_disabled}
                                >
                                    <span class="btn-icon">{ "🔍" }</span>
                                    { find_path_text }
                                </button>

                                <button
                                    class={format!("btn btn-success {}", if start_journey_disabled { "disabled" } else { "" })}
                                    onclick={if start_journey_disabled { Callback::noop() } else { Callback::from(move |_| on_start_journey.emit(())) }}
                                    disabled={start_journey_disabled}
                                >
                                    <span class="btn-icon">{ "🚀" }</span>
                                    { start_journey_text }
                                </button>

                                <button
                                    class={format!("btn btn-secondary {}", if pause_button_disabled { "disabled" } else { "" })}
                                    onclick={if pause_button_disabled { Callback::noop() } else { Callback::from(move |_| on_pause.emit(())) }}
                                    disabled={pause_button_disabled}
                                >
                                    <span class="btn-icon">{ "⏸️" }</span>
                                    { "Pause" }
                                </button>

                                <button
                                    class="btn btn-warning"
                                    onclick={Callback::from(move |_| on_restart.emit(()))}
                                    disabled={is_animating}
                                >
                                    <span class="btn-icon">{ "🔄" }</span>
                                    { "Restart" }
                                </button>

                                <button
                                    class="btn btn-danger"
                                    onclick={Callback::from(move |_| on_reset.emit(()))}
                                    disabled={is_animating}
                                >
                                    <span class="btn-icon">{ "🔧" }</span>
                                    { "Reset" }
                                </button>
                            </div>
                        </div>

                        <div class="controls-section">
                            <div class="select-wrapper">
                                <label class="control-label">{ "Algorithm" }</label>
                                <select
                                    class="select-input"
                                    onchange={on_change_algo}
                                    disabled={is_computing || is_animating}
                                    value={current_algorithm.clone()}
                                >
                                    <option value="D*-Lite" selected={current_algorithm == "D*-Lite"}>{ "D*-Lite (Dynamic)" }</option>
                                    <option value="A*" selected={current_algorithm == "A*"}>{ "A* (Classic)" }</option>
                                    <option value="Field D*" selected={current_algorithm == "Field D*"}>{ "Field D* (Smooth)" }</option>
                                </select>
                            </div>

                            <div class="slider-wrapper">
                                <label class="control-label">
                                    { "Speed" }
                                    <span class="speed-value">{ current_speed }</span>
                                </label>
                                <input
                                    type="range"
                                    class="range-input"
                                    min="1"
                                    max="10"
                                    value={current_speed.to_string()}
                                    oninput={on_change_speed}
                                    disabled={is_computing}
                                />
                                <div class="speed-markers">
                                    <span>{ "Slow" }</span>
                                    <span>{ "Fast" }</span>
                                </div>
                            </div>
                        </div>

                        <div class="controls-section">
                            <div class="legend">
                                <div class="legend-title">{ "Map Legend" }</div>
                                <div class="legend-grid">
                                    <div class="legend-item">
                                        <div class="legend-color start"></div>
                                        <span>{ "Start" }</span>
                                    </div>
                                    <div class="legend-item">
                                        <div class="legend-color goal"></div>
                                        <span>{ "Goal" }</span>
                                    </div>
                                    <div class="legend-item">
                                        <div class="legend-color obstacle"></div>
                                        <span>{ "Obstacle" }</span>
                                    </div>
                                    <div class="legend-item">
                                        <div class="legend-color dynamic-obstacle"></div>
                                        <span>{ "Undiscovered" }</span>
                                    </div>
                                    <div class="legend-item">
                                        <div class="legend-color converted"></div>
                                        <span>{ "Discovered Obs." }</span>
                                    </div>
                                    <div class="legend-item">
                                        <div class="legend-color path"></div>
                                        <span>{ "Path" }</span>
                                    </div>
                                    <div class="legend-item">
                                        <div class="legend-color rover"></div>
                                        <span>{ "Rover" }</span>
                                    </div>
                                    <div class="legend-item">
                                        <div class="legend-color detection"></div>
                                        <span>{ "Detection" }</span>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </>
                }
            } else {
                html! {}
            }}
        </div>
    }
}
