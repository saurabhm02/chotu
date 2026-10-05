use std::cell::RefCell;
use std::rc::Rc;

use yew::prelude::*;

use crate::api::dispatcher::run_cmd;
use crate::api::history::{
    invoke_chat_add_message, invoke_chat_create, invoke_chat_generate_title, invoke_chat_messages,
    invoke_chat_rename, invoke_store_attachments,
};
use crate::models::chat::NewMessage;
use crate::models::stream::{Phase, StreamEvent};
use crate::models::{detect_cmd, ChatTurn, Commands};
use crate::utils::memory::recent_messages;
use crate::utils::restore::turns_from_messages;
use crate::utils::time::{clock_from_ms, current_time_str};

/// The full conversation so far, one `ChatTurn` per question asked.
#[derive(Clone, PartialEq, Default)]
struct History(Vec<ChatTurn>);

enum HistoryAction {
    /// Show a new, still-empty turn right away, while we wait for the AI.
    StartTurn(ChatTurn),
    /// A new piece of streamed text just arrived — tack it onto the last turn.
    AppendToLastTurn(String),
    /// The backend moved on (searching, reading, thinking).
    SetStatus(Phase),
    /// The pages we are about to read (title, url).
    SetSources(Vec<(String, String)>),
    /// The model that is answering.
    SetModel(String),
    /// The AI is done — replace the last turn's answer with the final version.
    FinishLastTurn {
        response: String,
        sources: Vec<(String, String)>,
        elapsed_ms: u64,
        is_error: bool,
    },
    /// `/new`: empties the screen.
    Clear,
    SetAttachments(Vec<String>),
    /// Shows a saved chat in place of the current one.
    Replace(Vec<ChatTurn>),
}

impl Reducible for History {
    type Action = HistoryAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut turns = self.0.clone();

        match action {
            HistoryAction::StartTurn(turn) => turns.push(turn),
            HistoryAction::AppendToLastTurn(text) => {
                if let Some(turn) = turns.last_mut() {
                    turn.response.push_str(&text);
                }
            }
            HistoryAction::SetStatus(phase) => {
                if let Some(turn) = turns.last_mut() {
                    turn.status = Some(phase);
                }
            }
            HistoryAction::SetSources(sources) => {
                if let Some(turn) = turns.last_mut() {
                    turn.sources = sources;
                }
            }
            HistoryAction::SetModel(model) => {
                if let Some(turn) = turns.last_mut() {
                    turn.model = model;
                }
            }
            HistoryAction::FinishLastTurn {
                response,
                sources,
                elapsed_ms,
                is_error,
            } => {
                if let Some(turn) = turns.last_mut() {
                    turn.response = response;
                    turn.sources = sources;
                    turn.elapsed_ms = Some(elapsed_ms);
                    turn.is_error = is_error;
                }
            }
            HistoryAction::Clear => turns.clear(),
            HistoryAction::Replace(new_turns) => turns = new_turns,
            HistoryAction::SetAttachments(paths) => {
                if let Some(turn) = turns.last_mut() {
                    turn.attachments = paths;
                }
            }
        }

        Rc::new(History(turns))
    }
}

pub struct ChatHandle {
    pub history: Vec<ChatTurn>,
    pub is_loading: bool,
    /// Send (what the user typed, text they had highlighted in another app).
    /// Streams the reply into `history`.
    pub send: Callback<(String, Option<String>)>,
    /// Re-send the prompt already sitting at this position in `history`.
    pub regenerate: Callback<usize>,
    /// `Some(filter)` while the history list is open.
    pub history_panel: Option<String>,
    pub close_panel: Callback<()>,
    /// Shows a saved chat; new messages continue it.
    pub open_chat: Callback<i64>,
    /// Empties the screen if the deleted chat is the one shown.
    pub forget_chat: Callback<i64>,
}

/// Chat history plus everything needed to send a message and watch it stream in.
#[hook]
pub fn use_chat() -> ChatHandle {
    let history = use_reducer(History::default);
    let is_loading = use_state(|| false);
    let chat_id = use_mut_ref(|| None::<i64>);
    // `Some(filter)` while the history list is open.
    let panel = use_state(|| None::<String>);

    let send = {
        let history = history.clone();
        let is_loading = is_loading.clone();
        let chat_id = chat_id.clone();
        let panel = panel.clone();

        Callback::from(move |(raw_text, quote): (String, Option<String>)| {
            let prompt = raw_text.trim().to_string();
            if prompt.is_empty() {
                return;
            }

            let (command, rest) = detect_cmd(&prompt);
            match command {
                // Ignored while an answer streams, so its tail cannot land in the new chat.
                Some(Commands::New) => {
                    if !*is_loading {
                        history.dispatch(HistoryAction::Clear);
                        *chat_id.borrow_mut() = None;
                        panel.set(None);
                    }
                    return;
                }
                Some(Commands::History) => {
                    panel.set(Some(rest));
                    return;
                }
                Some(Commands::Rename) => {
                    let current = *chat_id.borrow();
                    let panel = panel.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        if let (Some(id), false) = (current, rest.trim().is_empty()) {
                            if let Err(e) = invoke_chat_rename(id, &rest).await {
                                log::warn!("could not rename the chat: {e}");
                            }
                        }
                        panel.set(Some(String::new()));
                    });
                    return;
                }
                _ => {}
            }

            panel.set(None);

            let history = history.clone();
            let is_loading = is_loading.clone();
            let chat_id = chat_id.clone();
            wasm_bindgen_futures::spawn_local(send_and_stream_reply(
                history, is_loading, chat_id, prompt, quote,
            ));
        })
    };

    let regenerate = {
        let send = send.clone();
        let history = history.clone();
        Callback::from(move |turn_index: usize| {
            if let Some(turn) = history.0.get(turn_index) {
                send.emit((turn.prompt.clone(), turn.quote.clone()));
            }
        })
    };

    let close_panel = {
        let panel = panel.clone();
        Callback::from(move |_: ()| panel.set(None))
    };

    let open_chat = {
        let history = history.clone();
        let is_loading = is_loading.clone();
        let chat_id = chat_id.clone();
        let panel = panel.clone();
        Callback::from(move |id: i64| {
            if *is_loading {
                return;
            }
            let history = history.clone();
            let chat_id = chat_id.clone();
            let panel = panel.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match invoke_chat_messages(id).await {
                    Ok(messages) => {
                        let turns = turns_from_messages(&messages, clock_from_ms);
                        history.dispatch(HistoryAction::Replace(turns));
                        *chat_id.borrow_mut() = Some(id);
                        panel.set(None);
                    }
                    Err(e) => log::warn!("could not open the chat: {e}"),
                }
            });
        })
    };

    let forget_chat = {
        let history = history.clone();
        let is_loading = is_loading.clone();
        let chat_id = chat_id.clone();
        Callback::from(move |id: i64| {
            if !*is_loading && *chat_id.borrow() == Some(id) {
                history.dispatch(HistoryAction::Clear);
                *chat_id.borrow_mut() = None;
            }
        })
    };

    ChatHandle {
        history: history.0.clone(),
        is_loading: *is_loading,
        send,
        regenerate,
        history_panel: (*panel).clone(),
        close_panel,
        open_chat,
        forget_chat,
    }
}

/// Runs one full request: show a blank turn immediately, stream the answer in
/// chunk by chunk, then lock in the final text once the backend finishes.
async fn send_and_stream_reply(
    history: UseReducerHandle<History>,
    is_loading: UseStateHandle<bool>,
    chat_id: Rc<RefCell<Option<i64>>>,
    prompt: String,
    quote: Option<String>,
) {
    is_loading.set(true);
    let started = js_sys::Date::now();

    // `query` is what's left after removing a leading "/command".
    let (command, query) = detect_cmd(&prompt);

    let memory = recent_messages(&history.0);

    history.dispatch(HistoryAction::StartTurn(ChatTurn {
        prompt: prompt.clone(), // shown exactly as typed, e.g. "/web rust news"
        quote: quote.clone(),
        response: String::new(),
        sources: vec![],
        status: None,
        model: String::new(),
        elapsed_ms: None,
        is_error: false,
        attachments: vec![],
        timestamp: current_time_str(),
    }));

    // Saved before the answer so the question survives a crash mid-answer.
    let saved = save_question(&chat_id, &prompt, &quote).await;
    let saved_chat = saved.as_ref().map(|s| s.chat_id);
    let question_id = saved.as_ref().and_then(|s| s.message_id);
    let chat_is_new = saved.as_ref().map(|s| s.chat_is_new).unwrap_or(false);

    // The event callback learns the model name first; the save after the answer needs it.
    let model_name = Rc::new(RefCell::new(String::new()));

    let history_for_events = history.clone();
    let model_for_events = model_name.clone();
    let on_event = move |event: StreamEvent| {
        let action = match event {
            StreamEvent::Token(text) => HistoryAction::AppendToLastTurn(text),
            StreamEvent::Status(phase) => HistoryAction::SetStatus(phase),
            StreamEvent::Sources(list) => {
                HistoryAction::SetSources(list.into_iter().map(|s| (s.title, s.url)).collect())
            }
            StreamEvent::Model(name) => {
                *model_for_events.borrow_mut() = name.clone();
                HistoryAction::SetModel(name)
            }
        };
        history_for_events.dispatch(action);
    };

    let result = run_cmd(command, query, quote, memory, on_event).await;

    let elapsed_ms = (js_sys::Date::now() - started) as u64;

    if let (Some(message_id), false) = (question_id, result.screenshots.is_empty()) {
        match invoke_store_attachments(message_id, &result.screenshots).await {
            Ok(paths) => history.dispatch(HistoryAction::SetAttachments(paths)),
            Err(e) => log::warn!("could not store the screenshot: {e}"),
        }
    }

    if let Some(chat_id) = saved_chat {
        let model = model_name.borrow().clone();
        save_message(NewMessage {
            chat_id,
            role: "assistant".to_string(),
            content: result.text.clone(),
            quote: None,
            sources: result.sources.clone(),
            model: (!model.is_empty()).then_some(model),
            is_error: result.is_error,
            elapsed_ms: Some(elapsed_ms),
        })
        .await;

        // One title request per chat, after its first good answer. Nobody waits for it.
        if chat_is_new && !result.is_error {
            let first_message = prompt.clone();
            let answer = result.text.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match invoke_chat_generate_title(chat_id, &first_message, &answer).await {
                    Ok(title) => log::info!("chat {chat_id} titled {title:?}"),
                    Err(e) => log::info!("chat {chat_id} keeps its temporary title: {e}"),
                }
            });
        }
    }

    history.dispatch(HistoryAction::FinishLastTurn {
        response: result.text,
        sources: result.sources,
        elapsed_ms,
        is_error: result.is_error,
    });
    is_loading.set(false);
}

struct SavedQuestion {
    chat_id: i64,
    /// `None` when saving the message itself failed.
    message_id: Option<i64>,
    chat_is_new: bool,
}

/// Saves the question, creating the chat first if needed. `None` means the chat could not
/// be saved; the conversation on screen carries on regardless.
async fn save_question(
    chat_id: &Rc<RefCell<Option<i64>>>,
    prompt: &str,
    quote: &Option<String>,
) -> Option<SavedQuestion> {
    let existing = *chat_id.borrow();
    let (id, chat_is_new) = match existing {
        Some(id) => (id, false),
        None => match invoke_chat_create(prompt).await {
            Ok(id) => {
                *chat_id.borrow_mut() = Some(id);
                (id, true)
            }
            Err(e) => {
                log::warn!("could not start a saved chat: {e}");
                return None;
            }
        },
    };

    let message_id = save_message(NewMessage {
        chat_id: id,
        role: "user".to_string(),
        content: prompt.to_string(),
        quote: quote.clone(),
        sources: vec![],
        model: None,
        is_error: false,
        elapsed_ms: None,
    })
    .await;
    Some(SavedQuestion {
        chat_id: id,
        message_id,
        chat_is_new,
    })
}

/// Saves a message. A failure is only logged.
async fn save_message(message: NewMessage) -> Option<i64> {
    match invoke_chat_add_message(&message).await {
        Ok(id) => Some(id),
        Err(e) => {
            log::warn!("could not save a message: {e}");
            None
        }
    }
}
