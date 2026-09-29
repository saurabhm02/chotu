use serde::Serialize;

#[derive(Serialize)]
pub struct AskBarInput {
    prompt: String,
}
