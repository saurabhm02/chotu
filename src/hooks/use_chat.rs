use std::rc::Rc;

use yew::prelude::*;

use crate::api::dispatcher::run_cmd;
use crate::models::{detect_cmd, ChatTurn};
use crate::utils::time::current_time_str;

/// The full conversation so far, one `ChatTurn` per question asked.
#[derive(Clone, PartialEq, Default)]
struct History(Vec<ChatTurn>);

enum HistoryAction {
    /// Show a new, still-empty turn right away, while we wait for the AI.
    StartTurn(ChatTurn),
    /// A new piece of streamed text just arrived — tack it onto the last turn.
    AppendToLastTurn(String),
    /// The AI is done — replace the last turn's answer with the final version.
    FinishLastTurn {
        response: String,
        sources: Vec<(String, String)>,
    },
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
            HistoryAction::FinishLastTurn { response, sources } => {
                if let Some(turn) = turns.last_mut() {
                    turn.response = response;
                    turn.sources = sources;
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
}

/// Chat history plus everything needed to send a message and watch it stream in.
#[hook]
pub fn use_chat() -> ChatHandle {
    let history = use_reducer(History::default);
    let is_loading = use_state(|| false);

    let send = {
        let history = history.clone();
        let is_loading = is_loading.clone();

        Callback::from(move |(raw_text, quote): (String, Option<String>)| {
            let prompt = raw_text.trim().to_string();
            if prompt.is_empty() {
                return;
            }

            let history = history.clone();
            let is_loading = is_loading.clone();
            wasm_bindgen_futures::spawn_local(send_and_stream_reply(
                history, is_loading, prompt, quote,
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
    prompt: String,
    quote: Option<String>,
) {
    is_loading.set(true);

    // `query` is what's left after removing a leading "/command".
    let (command, query) = detect_cmd(&prompt);

    history.dispatch(HistoryAction::StartTurn(ChatTurn {
        prompt: prompt.clone(), // shown exactly as typed, e.g. "/web rust news"
        quote: quote.clone(),
        response: String::new(),
        sources: vec![],
        timestamp: current_time_str(),
    }));

    let history_for_chunks = history.clone();
    let on_chunk = move |chunk: String| {
        history_for_chunks.dispatch(HistoryAction::AppendToLastTurn(chunk));
    };

    let (response, sources) = run_cmd(command, query, quote, on_chunk).await;

    history.dispatch(HistoryAction::FinishLastTurn { response, sources });
    is_loading.set(false);
}
