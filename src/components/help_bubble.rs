// src/components/help_bubble.rs

use wasm_bindgen::JsCast;
use web_sys::MouseEvent;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct HelpBubbleProps {
    pub on_close: Callback<()>,
}

#[function_component(HelpBubble)]
pub fn help_bubble(props: &HelpBubbleProps) -> Html {
    let on_close = props.on_close.clone();
    let position = use_state(|| (80.0, 20.0)); // x, y position
    let is_dragging = use_state(|| false);
    let drag_start = use_state(|| (0.0, 0.0));
    let is_expanded = use_state(|| false);

    let onmousedown = {
        let is_dragging = is_dragging.clone();
        let drag_start = drag_start.clone();
        let position = position.clone();

        Callback::from(move |e: MouseEvent| {
            // Only drag from header
            if let Some(target) = e.target() {
                if let Ok(element) = target.dyn_into::<web_sys::HtmlElement>() {
                    if element.class_list().contains("help-header") {
                        e.prevent_default();
                        is_dragging.set(true);
                        drag_start.set((
                            e.client_x() as f64 - position.0,
                            e.client_y() as f64 - position.1,
                        ));
                    }
                }
            }
        })
    };

    let onmousemove = {
        let is_dragging = is_dragging.clone();
        let drag_start = drag_start.clone();
        let position = position.clone();

        Callback::from(move |e: MouseEvent| {
            if *is_dragging {
                e.prevent_default();
                let new_x = e.client_x() as f64 - drag_start.0;
                let new_y = e.client_y() as f64 - drag_start.1;
                position.set((new_x, new_y));
            }
        })
    };

    let onmouseup = {
        let is_dragging = is_dragging.clone();

        Callback::from(move |_: MouseEvent| {
            is_dragging.set(false);
        })
    };

    let onmouseleave = {
        let is_dragging = is_dragging.clone();

        Callback::from(move |_: MouseEvent| {
            is_dragging.set(false);
        })
    };

    let toggle_expand = {
        let is_expanded = is_expanded.clone();
        Callback::from(move |_| {
            is_expanded.set(!*is_expanded);
        })
    };

    html! {
        <div
            class={format!("help-bubble {}", if *is_expanded { "expanded" } else { "" })}
            style={format!("right: {}px; top: {}px;", position.0, position.1)}
            onmousedown={onmousedown}
            onmousemove={onmousemove}
            onmouseup={onmouseup}
            onmouseleave={onmouseleave}
        >
            <div class="help-header">
                <span class="help-title">{ "💡 Quick Guide" }</span>
                <div class="help-controls">
                    <button
                        class="help-expand-btn"
                        onclick={toggle_expand}
                        aria-label="Toggle expand"
                    >
                        { if *is_expanded { "−" } else { "+" } }
                    </button>
                    <button
                        class="help-close-btn"
                        onclick={Callback::from(move |_| on_close.emit(()))}
                        aria-label="Close help"
                    >
                        { "×" }
                    </button>
                </div>
            </div>

            {if *is_expanded {
                html! {
                    <div class="help-content">
                        <div class="help-section">
                            <strong>{ "Controls:" }</strong>
                            <ul>
                                <li>{ "🖱️ Click & drag to place obstacles" }</li>
                                <li>{ "🎯 Drag S/G to move start/goal" }</li>
                                <li>{ "🚀 Find Path → Start Journey" }</li>
                            </ul>
                        </div>
                        <div class="help-section">
                            <strong>{ "Memoryless Rover:" }</strong>
                            <ul>
                                <li>{ "🤖 Recalculates path EVERY cell" }</li>
                                <li>{ "🟡 Yellow obstacles detect rover" }</li>
                                <li>{ "🟠 Detection range = 2 cells" }</li>
                                <li>{ "⚡ Auto-converts when detected" }</li>
                                <li>{ "🔄 Never pauses - continuous motion" }</li>
                                <li>{ "⚠️ Shows alert if trapped" }</li>
                            </ul>
                        </div>
                        <div class="help-section">
                            <strong>{ "Path Colors:" }</strong>
                            <ul>
                                <li>{ "🟦 Turquoise = traveled path" }</li>
                                <li>{ "🟪 Purple = future path" }</li>
                            </ul>
                        </div>
                    </div>
                }
            } else {
                html! {}
            }}
        </div>
    }
}
