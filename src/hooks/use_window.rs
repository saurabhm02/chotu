use std::rc::Rc;

use wasm_bindgen::JsCast;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

use crate::api::tauri::{listen_to_event, listen_to_text_event};
use crate::api::window::{resize_window, start_window_drag};

const MINI_SIZE: f64 = 52.0;
const EXPANDED_WIDTH: f64 = 550.0;
const MIN_HEIGHT: f64 = 36.0;
const MAX_HEIGHT: f64 = 700.0;
/// While an answer streams in, resize the window at most this often (milliseconds).
const RESIZE_EVERY_MS: f64 = 100.0;
/// Clicking any of these should never start a window drag.
const INTERACTIVE_TAGS: [&str; 6] = ["textarea", "input", "button", "svg", "path", "a"];

#[derive(Clone, PartialEq, Default)]
struct MiniState(bool);

enum MiniAction {
    Toggle,
    Set(bool),
}

impl Reducible for MiniState {
    type Action = MiniAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            MiniAction::Toggle => Rc::new(MiniState(!self.0)),
            MiniAction::Set(value) => Rc::new(MiniState(value)),
        }
    }
}

pub struct WindowHandle {
    pub is_mini: bool,
    pub on_expand: Callback<MouseEvent>,
    /// Starts a window drag when the press lands on empty background, not on
    /// an interactive element like the text box or a button.
    pub on_mousedown: Callback<MouseEvent>,
}

/// Mini/expanded window state, the global shortcut toggle, and OS window sizing.
/// `content` only signals when to re-measure.
#[hook]
pub fn use_window<D: PartialEq + 'static>(input_ref: NodeRef, content: D) -> WindowHandle {
    let is_mini = use_reducer(MiniState::default);

    use_global_toggle_listener(&is_mini);
    use_native_window_resize(&is_mini, &input_ref, content);

    let on_expand = {
        let is_mini = is_mini.clone();
        Callback::from(move |_: MouseEvent| is_mini.dispatch(MiniAction::Set(false)))
    };

    let on_mousedown = Callback::from(start_drag_unless_interactive);

    WindowHandle {
        is_mini: is_mini.0,
        on_expand,
        on_mousedown,
    }
}

/// Subscribes once to the backend's global shortcut event (e.g. Cmd+/), which
/// can flip mini mode even while the app isn't focused.
#[hook]
fn use_global_toggle_listener(is_mini: &UseReducerHandle<MiniState>) {
    let is_mini = is_mini.clone();
    use_effect_with((), move |_| {
        let is_mini_for_toggle = is_mini.clone();
        listen_to_event("toggle-floating-mode", move || {
            is_mini_for_toggle.dispatch(MiniAction::Toggle);
        });
        // Selected text arrived: always open (never collapse) so it can be shown.
        listen_to_text_event("selected-text", move |_| {
            is_mini.dispatch(MiniAction::Set(false));
        });
        || ()
    });
}

/// Resizes the real OS window whenever mini mode or the content changes:
/// a tiny square in mini mode, otherwise just tall enough for the content.
#[hook]
fn use_native_window_resize<D: PartialEq + 'static>(
    is_mini: &UseReducerHandle<MiniState>,
    input_ref: &NodeRef,
    content: D,
) {
    let want_mini = is_mini.0;
    let input_ref = input_ref.clone();
    // When the window was last resized (milliseconds).
    let last_run = use_mut_ref(|| 0.0_f64);

    use_effect_with((content, want_mini), move |_| {
        // Resize now if it has been a while, otherwise wait out the rest of the 100 ms.
        let since_last = js_sys::Date::now() - *last_run.borrow();
        let wait = (RESIZE_EVERY_MS - since_last).clamp(0.0, RESIZE_EVERY_MS);

        let run = move || {
            *last_run.borrow_mut() = js_sys::Date::now();
            wasm_bindgen_futures::spawn_local(apply_window_size(want_mini, input_ref));
        };
        let pending = gloo_timers::callback::Timeout::new(wait as u32, run);

        // New content arrived before the timer fired: dropping the timer cancels it.
        move || drop(pending)
    });
}

/// Re-fit the window to its content. For changes the hook can't see,
/// like opening the sources list inside an answer.
pub fn refit_window() {
    gloo_timers::callback::Timeout::new(30, || {
        wasm_bindgen_futures::spawn_local(apply_window_size(false, NodeRef::default()));
    })
    .forget();
}

async fn apply_window_size(want_mini: bool, input_ref: NodeRef) {
    if want_mini {
        let _ = resize_window(MINI_SIZE, MINI_SIZE).await;
        return;
    }

    let height = measure_content_height().clamp(MIN_HEIGHT, MAX_HEIGHT);
    if let Err(e) = resize_window(EXPANDED_WIDTH, height).await {
        log::error!("resize failed: {:?}", e);
    }
    if let Some(input) = input_ref.cast::<HtmlTextAreaElement>() {
        let _ = input.focus();
    }
}

/// Height the expanded UI needs. Measured at the target width: when expanding,
/// the window is still 52px wide, which would wrap the content and over-report.
fn measure_content_height() -> f64 {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return MIN_HEIGHT;
    };

    let Some(el) = doc.get_element_by_id("app-container") else {
        return doc
            .body()
            .map(|body| body.scroll_height() as f64)
            .unwrap_or(MIN_HEIGHT);
    };

    let style = el.unchecked_ref::<web_sys::HtmlElement>().style();
    // The container is capped to the window (so the input never leaves the screen).
    // Lift the cap while measuring, so we learn the height the content really wants.
    let _ = style.set_property("width", &format!("{EXPANDED_WIDTH}px"));
    let _ = style.set_property("max-height", "none");
    let height = el.scroll_height() as f64;
    let _ = style.remove_property("width");
    let _ = style.remove_property("max-height");
    height
}

fn start_drag_unless_interactive(event: MouseEvent) {
    if event.button() != 0 {
        return; // only the left mouse button starts a drag
    }
    let Some(target) = event.target_dyn_into::<web_sys::HtmlElement>() else {
        return;
    };
    let tag = target.tag_name().to_lowercase();
    if !INTERACTIVE_TAGS.contains(&tag.as_str()) {
        start_window_drag();
    }
}
