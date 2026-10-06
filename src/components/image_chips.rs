use yew::prelude::*;

use crate::api::images::invoke_preview;
use crate::hooks::use_window::refit_window;

#[derive(Properties, PartialEq)]
pub struct ImageChipsProps {
    pub paths: Vec<String>,
    pub on_remove: Callback<String>,
}

#[function_component(ImageChips)]
pub fn image_chips(props: &ImageChipsProps) -> Html {
    if props.paths.is_empty() {
        return html! {};
    }

    html! {
        <div class="flex flex-wrap gap-2 px-3 py-3">
            {
                for props.paths.iter().map(|path| html!{
                    <ImageChip key={path.clone()} path={path.clone()} on_remove={props.on_remove.clone()} />
                })
            }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct ImageChipProps {
    pub path: String,
    pub on_remove: Callback<String>,
}

#[function_component(ImageChip)]
pub fn image_chip(props: &ImageChipProps) -> Html {
    let preview = use_state(|| None::<String>);

    {
        let preview = preview.clone();
        use_effect_with(props.path.clone(), move |path| {
            let path = path.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match invoke_preview(&path).await {
                    Ok(url) => {
                        preview.set(Some(url));
                        refit_window();
                    }
                    Err(e) => log::warn!("could not preview {path}: {e}"),
                }
            });
            || ()
        });
    }


    let remove = {
        let path = props.path.clone();
        let on_remove = props.on_remove.clone();
        Callback::from(move |_: MouseEvent| on_remove.emit(path.clone()))
    };

    html! {
        <div class = "relative h-14 w-14 shrink-0">
            if let Some(url) = &*preview {
                <img src={url.clone()} class="h-full w-full rounded-lg border border-white/15 object-cover" />
            } else{
                <div class="h-full w-full rounded-lg border border-white/10 bg-white/[0.04]"></div>
            }

            <button
                onclick={remove}
                title="Remove image"
                class="absolute -right-1.5 -top-1.5 flex h-4 w-4 items-center justify-center rounded-full border border-white/30 bg-black text-[11px] leading-none text-white cursor-pointer"
            >
                {"×"}
            </button>
        </div>
    }
}
