use std::cell::RefCell;
use std::rc::Rc;

use yew::prelude::*;

use crate::api::dispatcher::run_cmd;
use crate::api::history::{invoke_chat_add_message, invoke_chat_create};
use crate::models::chat::NewMessage;
use crate::models::stream::{Phase, StreamEvent};
use crate::models::{detect_cmd, ChatTurn, Commands};
use crate::utils::memory::recent_messages;
use crate::utils::time::current_time_str;

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
    /// `/new`: forget the chat on screen and start empty.
    Clear,
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
}

/// Chat history plus everything needed to send a message and watch it stream in.
#[hook]
pub fn use_chat() -> ChatHandle {
    let history = use_reducer(History::default);
    let is_loading = use_state(|| false);
    let chat_id = use_mut_ref(|| None::<i64>);

    let send = {
        let history = history.clone();
        let is_loading = is_loading.clone();
        let chat_id = chat_id.clone();

        Callback::from(move |(raw_text, quote): (String, Option<String>)| {
            let prompt = raw_text.trim().to_string();
            if prompt.is_empty() {
                return;
            }

            if detect_cmd(&prompt).0 == Some(Commands::New) {
                if !*is_loading {
                    history.dispatch(HistoryAction::Clear);
                    *chat_id.borrow_mut() = None;
                }
                return;
            }

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

    ChatHandle {
        history: history.0.clone(),
        is_loading: *is_loading,
        send,
        regenerate,
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

    // What the AI should remember: the last messages of this chat, taken before the
    // new turn is added.
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
        timestamp: current_time_str(),
    }));

    // Save the question right away, so it is kept even if the app closes mid-answer.
    let saved_chat = save_question(&chat_id, &prompt, &quote).await;

    // The words arrive on a callback, so the model's name is kept here for the save below.
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

    // Save the answer too (errors as well, marked as errors).
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
    }

    history.dispatch(HistoryAction::FinishLastTurn {
        response: result.text,
        sources: result.sources,
        elapsed_ms,
        is_error: result.is_error,
    });
    is_loading.set(false);
}

/// Saves the user's question. The chat is created first if this is its first message.
/// Returns the chat id, or `None` when saving does not work (the chat still continues).
async fn save_question(
    chat_id: &Rc<RefCell<Option<i64>>>,
    prompt: &str,
    quote: &Option<String>,
) -> Option<i64> {
    let existing = *chat_id.borrow();
    let id = match existing {
        Some(id) => id,
        None => match invoke_chat_create(prompt).await {
            Ok(id) => {
                *chat_id.borrow_mut() = Some(id);
                id
            }
            Err(e) => {
                log::warn!("could not start a saved chat: {e}");
                return None;
            }
        },
    };

    save_message(NewMessage {
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
    Some(id)
}

/// A failed save is only logged: the chat on screen keeps working.
async fn save_message(message: NewMessage) {
    if let Err(e) = invoke_chat_add_message(&message).await {
        log::warn!("could not save a message: {e}");
    }
}
