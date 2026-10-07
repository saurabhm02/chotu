use yew::prelude::*;

use crate::api::images::invoke_view;
use crate::hooks::use_window::refit_window;

#[derive(Properties, PartialEq)]
pub struct ImageViewerProps {
    pub path: String,
    pub on_close: Callback<()>,
}

/// One image shown large, above the input. A click on the image or on `×` closes it.
#[function_component(ImageViewer)]
pub fn image_viewer(props: &ImageViewerProps) -> Html {
    let source = use_state(|| None::<String>);

    {
        let source = source.clone();
        use_effect_with(props.path.clone(), move |path| {
            let path = path.clone();
            source.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                match invoke_view(&path).await {
                    Ok(url) => {
                        source.set(Some(url));
                        refit_window();
                    }
                    Err(e) => log::warn!("could not open {path}: {e}"),
                }
            });
            || ()
        });
    }

    let close = {
        let on_close = props.on_close.clone();
        Callback::from(move |_: MouseEvent| on_close.emit(()))
    };

    html! {
        <div class="relative w-full px-3 pt-3">
            <div class="relative flex min-h-24 items-center justify-center rounded-lg border border-white/10 bg-white/[0.03] p-2">
                if let Some(url) = &*source {
                    <img
                        src={url.clone()}
                        onclick={close.clone()}
                        title="Click to close"
                        class="max-h-[460px] max-w-full cursor-zoom-out rounded object-contain"
                    />
                } else {
                    <span class="text-[12px] text-neutral-500">{"Loading..."}</span>
                }

                <button
                    onclick={close}
                    title="Close"
                    class="absolute right-2 top-2 flex h-5 w-5 items-center justify-center rounded-full bg-black/70 text-[14px] leading-none text-white hover:bg-black cursor-pointer"
                >
                    {"×"}
                </button>
            </div>
        </div>
    }
}
