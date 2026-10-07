use yew::prelude::*;

use crate::api::images::invoke_preview;

/// Width and height of one card.
const CARD_PX: usize = 32;
/// How many pixels of each older card show below the card in front of it.
const STACK_STEP_PX: usize = 12;
/// How much darker each older card is than the one in front of it.
const DIM_PERCENT: usize = 12;

#[derive(Properties, PartialEq)]
pub struct AttachmentStackProps {
    pub paths: Vec<String>,
    pub on_remove: Callback<String>,
    /// A card was clicked: show this image large.
    pub on_view: Callback<String>,
}

/// The pasted attachments, piled like cards with the newest in front. Sits where the
/// expand icon is, next to the send button.
#[function_component(AttachmentStack)]
pub fn attachment_stack(props: &AttachmentStackProps) -> Html {
    let Some(newest) = props.paths.last() else {
        return html! {};
    };
    let count = props.paths.len();

    let remove_newest = {
        let path = newest.clone();
        let on_remove = props.on_remove.clone();
        Callback::from(move |_: MouseEvent| on_remove.emit(path.clone()))
    };

    html! {
        <div
            class="relative w-8 shrink-0"
            style={format!("height: {}px;", stack_height(count))}
        >
            <StackFrameMarks />

            { for props.paths.iter().enumerate().map(|(index, path)| {
                let depth = count - 1 - index;
                html! {
                    <StackCard
                        key={path.clone()}
                        path={path.clone()}
                        style={card_style(depth)}
                        on_view={props.on_view.clone()}
                    />
                }
            }) }

            <button
                onclick={remove_newest}
                title="Remove image"
                class="absolute right-1 top-1 z-30 flex h-4 w-4 items-center justify-center rounded-full bg-black/70 text-[12px] leading-none text-white hover:bg-black cursor-pointer"
            >
                {"×"}
            </button>
        </div>
    }
}

/// Total height of a stack of `count` cards.
fn stack_height(count: usize) -> usize {
    CARD_PX + count.saturating_sub(1) * STACK_STEP_PX
}

/// Where one card sits. `depth` is how many cards are in front of it: 0 is the front card.
fn card_style(depth: usize) -> String {
    let top = depth * STACK_STEP_PX;
    let layer = 10usize.saturating_sub(depth);
    let brightness = 100usize.saturating_sub(depth * DIM_PERCENT);
    format!("top: {top}px; z-index: {layer}; filter: brightness({brightness}%);")
}

/// Four corner brackets around the stack, a little outside it. They follow the stack's
/// size on their own, because they are placed against the corners of its box.
#[function_component(StackFrameMarks)]
fn stack_frame_marks() -> Html {
    html! {
        <>
            <div class="pointer-events-none absolute -left-1 -top-1 h-2 w-2 border-l-[1.8px] border-t-[1.8px] border-red-400"></div>
            <div class="pointer-events-none absolute -right-1 -top-1 h-2 w-2 border-r-[1.8px] border-t-[1.8px] border-red-400"></div>
            <div class="pointer-events-none absolute -bottom-1 -left-1 h-2 w-2 border-b-[1.8px] border-l-[1.8px] border-red-400"></div>
            <div class="pointer-events-none absolute -bottom-1 -right-1 h-2 w-2 border-b-[1.8px] border-r-[1.8px] border-red-400"></div>
        </>
    }
}

#[derive(Properties, PartialEq)]
struct StackCardProps {
    path: String,
    style: String,
    on_view: Callback<String>,
}

#[function_component(StackCard)]
fn stack_card(props: &StackCardProps) -> Html {
    let preview = use_state(|| None::<String>);

    {
        let preview = preview.clone();
        use_effect_with(props.path.clone(), move |path| {
            let path = path.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match invoke_preview(&path).await {
                    Ok(url) => preview.set(Some(url)),
                    Err(e) => log::warn!("could not preview {path}: {e}"),
                }
            });
            || ()
        });
    }

    let view = {
        let path = props.path.clone();
        let on_view = props.on_view.clone();
        Callback::from(move |_: MouseEvent| on_view.emit(path.clone()))
    };

    html! {
        <div
            onclick={view}
            title="Click to view"
            class="absolute left-0 h-8 w-8 cursor-pointer"
            style={props.style.clone()}
        >
            if let Some(url) = &*preview {
                <img src={url.clone()} class="h-full w-full rounded-lg border border-white/30 object-cover" />
            } else {
                <div class="h-full w-full rounded-lg border border-white/20 bg-neutral-800"></div>
            }
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_card_is_as_tall_as_a_card() {
        assert_eq!(stack_height(1), CARD_PX);
    }

    #[test]
    fn each_extra_card_adds_one_step() {
        assert_eq!(stack_height(3), CARD_PX + 2 * STACK_STEP_PX);
    }

    #[test]
    fn an_empty_stack_does_not_go_below_one_card() {
        assert_eq!(stack_height(0), CARD_PX);
    }

    #[test]
    fn front_card_is_at_the_top_and_not_dimmed() {
        let style = card_style(0);
        assert!(style.contains("top: 0px"));
        assert!(style.contains("z-index: 10"));
        assert!(style.contains("brightness(100%)"));
    }

    #[test]
    fn older_cards_sit_lower_behind_and_darker() {
        let style = card_style(2);
        assert!(style.contains("top: 24px"));
        assert!(style.contains("z-index: 8"));
        assert!(style.contains("brightness(76%)"));
    }
}
