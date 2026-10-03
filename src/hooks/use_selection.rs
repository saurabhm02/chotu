use yew::prelude::*;

use crate::api::tauri::listen_to_text_event;

pub struct SelectionHandle {
    /// Text the user had selected in another app when they pressed Cmd+/.
    pub text: Option<String>,
    pub clear: Callback<()>,
}

/// Remembers the text the backend captured from the user's selection.
#[hook]
pub fn use_selection() -> SelectionHandle {
    let text = use_state(|| None::<String>);

    {
        let text = text.clone();
        use_effect_with((), move |_| {
            listen_to_text_event("selected-text", move |selected| text.set(Some(selected)));
            || ()
        });
    }

    let clear = {
        let text = text.clone();
        Callback::from(move |_: ()| text.set(None))
    };

    SelectionHandle {
        text: (*text).clone(),
        clear,
    }
}

