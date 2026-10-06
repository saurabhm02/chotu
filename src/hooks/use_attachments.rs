use std::rc::Rc;
use wasm_bindgen::JsCast;
use yew::prelude::*;

use crate::api::images::invoke_paste;

pub const MAX_ATTACHMENTS: usize = 4;

#[derive(Clone, PartialEq, Default)]
struct PreviewAttachments(Vec<String>);

enum PreviewAttachmentAction {
    Add(String),
    Remove(String),
}

impl Reducible for PreviewAttachments {
    type Action = PreviewAttachmentAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut paths = self.0.clone();

        match action {
            PreviewAttachmentAction::Add(path) => {
                if paths.len() < MAX_ATTACHMENTS && !paths.contains(&path) {
                    paths.push(path);
                }
            }
            PreviewAttachmentAction::Remove(path) => paths.retain(|existing| existing != &path),
        }
        Rc::new(PreviewAttachments(paths))
    }
}

pub struct PreviewAttachmentsHandle {
    pub paths: Vec<String>,
    pub on_paste: Callback<Event>,
    pub remove: Callback<String>,
}

#[hook]
pub fn use_preview_attachments() -> PreviewAttachmentsHandle {
    let attachments = use_reducer(PreviewAttachments::default);

    let on_paste = {
        let attachments = attachments.clone();
        Callback::from(move |event: Event| {
            if !clipboard_has_image(&event) {
                return;
            }

            // Stop the browser pasting the image bytes itself, Rust reads the clipboard.
            event.prevent_default();

            if attachments.0.len() >= MAX_ATTACHMENTS {
                return;
            }

            let attachments = attachments.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match invoke_paste().await {
                    Ok(Some(path)) => attachments.dispatch(PreviewAttachmentAction::Add(path)),
                    Ok(None) => {}
                    Err(e) => log::warn!("could not paste the attachment: {e}"),
                }
            });
        })
    };

    let remove = {
        let attachments = attachments.clone();
        Callback::from(move |path: String| {
            attachments.dispatch(PreviewAttachmentAction::Remove(path))
        })
    };

    PreviewAttachmentsHandle {
        paths: attachments.0.clone(),
        on_paste,
        remove,
    }
}

/// Whether the paste event carries at least one image file.
fn clipboard_has_image(event: &Event) -> bool {
    let Some(event) = event.dyn_ref::<web_sys::ClipboardEvent>() else {
        return false;
    };
    let Some(files) = event.clipboard_data().and_then(|data| data.files()) else {
        return false;
    };
    (0..files.length())
        .filter_map(|index| files.get(index))
        .any(|file| file.type_().starts_with("image/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply(state: PreviewAttachments, action: PreviewAttachmentAction) -> PreviewAttachments {
        (*Rc::new(state).reduce(action)).clone()
    }

    fn with_paths(paths: &[&str]) -> PreviewAttachments {
        PreviewAttachments(paths.iter().map(|path| path.to_string()).collect())
    }

    #[test]
    fn add_appends_a_path() {
        let state = apply(
            with_paths(&["a.png"]),
            PreviewAttachmentAction::Add("b.png".into()),
        );
        assert_eq!(state.0, ["a.png", "b.png"]);
    }

    #[test]
    fn add_ignores_a_path_that_is_already_there() {
        let state = apply(
            with_paths(&["a.png"]),
            PreviewAttachmentAction::Add("a.png".into()),
        );
        assert_eq!(state.0, ["a.png"]);
    }

    #[test]
    fn add_stops_at_the_limit() {
        let full = with_paths(&["1", "2", "3", "4"]);
        let state = apply(full, PreviewAttachmentAction::Add("5".into()));
        assert_eq!(state.0.len(), MAX_ATTACHMENTS);
        assert!(!state.0.contains(&"5".to_string()));
    }

    #[test]
    fn remove_drops_only_that_path() {
        let state = apply(
            with_paths(&["a", "b", "c"]),
            PreviewAttachmentAction::Remove("b".into()),
        );
        assert_eq!(state.0, ["a", "c"]);
    }

    #[test]
    fn remove_of_an_unknown_path_changes_nothing() {
        let state = apply(
            with_paths(&["a"]),
            PreviewAttachmentAction::Remove("zzz".into()),
        );
        assert_eq!(state.0, ["a"]);
    }
}
