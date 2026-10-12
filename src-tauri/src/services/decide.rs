use serde_json::{json, Value};
use std::collections::HashSet;
use std::env;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use crate::config::{
    DECISION_API_URL, DECISION_TIMEOUT_S, DEFAULT_DECISION_MODELS, NOUL_ONLY_DECISION_MODELS,
};
use crate::models::decision::{
    Answer, Answers, ChoiceAnswer, Decision, DecisionQuestion, QuestionType, ScoreAnswer,
};

use super::failover;

pub const QUESTION_ID: &str = "answer";

pub struct DecisionClient {
    client: reqwest::Client,
    url: String,
    key: String,
    models: Vec<String>,
}

static SHARED: OnceLock<DecisionClient> = OnceLock::new();

pub fn build_req_body(model: &str, text: &str, question: &str) -> Value {
    let mut que = serde_json::Map::new();
    que.insert(
        QUESTION_ID.to_string(),
        json!({ "type": "noul", "instructions": question }),
    );
    json!({ "model": model, "state": text, "questions": que })
}

pub fn parse_probability_from_response(reply: &Value) -> Option<f32> {
    let chance = reply["answers"][QUESTION_ID]["noul"].as_f64()? as f32;
    (0.0..=1.0).contains(&chance).then_some(chance)
}

impl DecisionClient {
    pub fn new(url: String, key: String, models: Vec<String>, wait: Duration) -> Self {
        let client = reqwest::Client::builder()
            .timeout(wait)
            .build()
            .expect("could not create the HTTP client");
        Self {
            client,
            url,
            key,
            models,
        }
    }

    pub fn shared() -> &'static DecisionClient {
        SHARED.get_or_init(DecisionClient::from_env)
    }

    fn from_env() -> Self {
        let key = env::var("API_KEY").unwrap_or_default();
        let models = failover::model_list(
            env::var("AI_DECISION_MODEL").ok().as_deref(),
            &DEFAULT_DECISION_MODELS,
        );
        let wait = Duration::from_secs(DECISION_TIMEOUT_S);
        log::info!("DecisionClient created (models: {models:?})");
        Self::new(DECISION_API_URL.to_string(), key, models, wait)
    }

    pub async fn ask_yes_or_no(&self, text: &str, question: &str) -> Option<Decision> {
        for model in &self.models {
            if let Some(probability_of_yes) = self.ask_model(model, text, question).await {
                return Some(Decision::from_probability(probability_of_yes));
            }
        }
        None
    }


    /// Sends `body` to one model. `timeout` replaces the client's usual limit for this call.
    /// Any problem (offline, slow, an error status, an unreadable reply) gives `None`.
    async fn post_to_model(
        &self,
        model: &str,
        body: &Value,
        timeout: Option<Duration>,
    ) -> Option<Value> {
        let mut request = self.client.post(&self.url).bearer_auth(&self.key).json(body);
        if let Some(timeout) = timeout {
            request = request.timeout(timeout);
        }

        let res = request
            .send()
            .await
            .inspect_err(|e| log::warn!("decision model {model} did not answer: {e}"))
            .ok()?;

        let status = res.status();
        if !status.is_success() {
            log::warn!("decision model {model} answered with {status}");
            return None;
        }

        res.json()
            .await
            .inspect_err(|e| log::warn!("decision model {model} sent a reply we cannot read: {e}"))
            .ok()
    }

    /// Asks several questions about `text` in one call, so it counts once against the free
    /// limit. The first model that answers every question wins; otherwise the fullest reply is
    /// returned. `None` means nothing usable came back, or the questions were empty or shared an id.
    pub async fn ask_questions(
        &self,
        text: &str,
        questions: &[DecisionQuestion],
        timeout: Duration,
    ) -> Option<Answers> {
        let unique_ids: HashSet<&str> = questions.iter().map(|q| q.id.as_str()).collect();
        if questions.is_empty() || unique_ids.len() != questions.len() {
            log::warn!("decision questions are empty or share an id, nothing was asked");
            return None;
        }

        let needs_full_model = questions.iter().any(DecisionQuestion::needs_full_model);
        let mut best: Option<Answers> = None;
        for model in &self.models {
            if needs_full_model && NOUL_ONLY_DECISION_MODELS.contains(&model.as_str()) {
                continue;
            }

            let started = Instant::now();
            let body = build_questions_body(model, text, questions);
            let Some(reply) = self.post_to_model(model, &body, Some(timeout)).await else {
                continue;
            };

            let answers = parse_answers(&reply, questions);
            log::info!(
                "decision model {model}: {} of {} answers in {} ms",
                answers.len(),
                questions.len(),
                started.elapsed().as_millis()
            );
            if answers.len() == questions.len() {
                return Some(answers);
            }
            if answers.len() > best.as_ref().map_or(0, Answers::len) {
                best = Some(answers);
            }
        }
        best
    }

    pub async fn ask_model(&self, model: &str, text: &str, question: &str) -> Option<f32> {
        let started = Instant::now();

        let reply = self
            .post_to_model(model, &build_req_body(model, text, question), None)
            .await?;

        let chance_of_yes = parse_probability_from_response(&reply);

        match chance_of_yes {
            Some(chance) => log::info!(
                "decision model {model}: {chance:.2} in {} ms",
                started.elapsed().as_millis()
            ),
            None => log::warn!("decision model {model} sent a reply with no usable answer"),
        }
        chance_of_yes
    }
}

fn build_questions_body(model: &str, text: &str, questions: &[DecisionQuestion]) -> Value {
    let mut body_questions = serde_json::Map::new();
    for question in questions {
        let body = match &question.question_type {
            QuestionType::YesNo => json!({
                "type": "noul",
                "instructions": question.instruction,
            }),
            QuestionType::Choice { options } => {
                let criteria: serde_json::Map<String, Value> = options
                    .iter()
                    .map(|option| (option.id.clone(), json!(option.description)))
                    .collect();
                json!({
                    "type": "choice",
                    "instructions": question.instruction,
                    "criteria": criteria,
                })
            }
            QuestionType::Score { levels } => json!({
                "type": "score",
                "instructions": question.instruction,
                "criteria": levels,
            }),
        };
        body_questions.insert(question.id.clone(), body);
    }
    json!({ "model": model, "state": text, "questions": body_questions })
}

fn unit_interval(value: &Value) -> Option<f32> {
    let number = value.as_f64()? as f32;
    (0.0..=1.0).contains(&number).then_some(number)
}

fn parse_answer(answer: &Value, question_type: &QuestionType) -> Option<Answer> {
    let confidence = unit_interval(&answer["confidence"]).unwrap_or(0.0);
    match question_type {
        QuestionType::YesNo => {
            unit_interval(&answer["noul"]).map(|p| Answer::YesNo(Decision::from_probability(p)))
        }
        QuestionType::Choice { options } => {
            let chosen = answer["choice"].as_str()?;
            if !options.iter().any(|option| option.id == chosen) {
                return None;
            }
            let probabilities = options
                .iter()
                .map(|option| {
                    let probability = unit_interval(&answer["probabilities"][option.id.as_str()]);
                    (option.id.clone(), probability.unwrap_or(0.0))
                })
                .collect();
            Some(Answer::Choice(ChoiceAnswer {
                chosen: chosen.to_string(),
                probabilities,
                confidence,
            }))
        }
        QuestionType::Score { levels } => {
            let value = answer["score"].as_f64()? as f32;
            let highest = levels.len().checked_sub(1)?;
            if !(0.0..=highest as f32).contains(&value) {
                return None;
            }
            let level = (value.round() as usize).min(highest);
            let probabilities = (0..levels.len())
                .map(|index| {
                    unit_interval(&answer["probabilities"][index.to_string().as_str()])
                        .unwrap_or(0.0)
                })
                .collect();
            Some(Answer::Score(ScoreAnswer {
                value,
                level,
                label: levels[level].clone(),
                probabilities,
                confidence,
            }))
        }
    }
}

fn parse_answers(reply: &Value, questions: &[DecisionQuestion]) -> Answers {
    let mut answers = Answers::default();
    for question in questions {
        let answer = &reply["answers"][question.id.as_str()];
        if let Some(parsed) = parse_answer(answer, &question.question_type) {
            answers.insert(&question.id, parsed);
        }
    }
    answers
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::decision::Verdict;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    const QUESTION: &str = "Does the text contain a password?";

    fn reply_with_chance(chance: f64) -> String {
        json!({ "answers": { QUESTION_ID: { "type": "noul", "noul": chance } } }).to_string()
    }

    async fn read_request(socket: &mut tokio::net::TcpStream) {
        let mut seen = Vec::new();
        let mut chunk = [0u8; 4096];
        loop {
            let count = socket.read(&mut chunk).await.unwrap();
            if count == 0 {
                return;
            }
            seen.extend_from_slice(&chunk[..count]);
            let text = String::from_utf8_lossy(&seen).to_string();
            if let Some(head_end) = text.find("\r\n\r\n") {
                let announced = text[..head_end]
                    .lines()
                    .find_map(|line| {
                        line.to_lowercase()
                            .strip_prefix("content-length:")
                            .map(str::to_string)
                    })
                    .and_then(|value| value.trim().parse::<usize>().ok())
                    .unwrap_or(0);
                if seen.len() >= head_end + 4 + announced {
                    return;
                }
            }
        }
    }

    async fn fake_server(replies: Vec<(u16, Option<String>)>) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/decisions", listener.local_addr().unwrap());
        tokio::spawn(async move {
            for (status, body) in replies {
                let (mut socket, _) = listener.accept().await.unwrap();
                read_request(&mut socket).await;
                match body {
                    Some(body) => {
                        let answer = format!(
                            "HTTP/1.1 {status} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                            body.len()
                        );
                        socket.write_all(answer.as_bytes()).await.unwrap();
                    }
                    None => {
                        tokio::spawn(async move {
                            let _silent = socket;
                            tokio::time::sleep(Duration::from_secs(5)).await;
                        });
                    }
                }
            }
        });
        url
    }

    fn decider_at(url: String, models: &[&str]) -> DecisionClient {
        let models = models.iter().map(|m| m.to_string()).collect();
        DecisionClient::new(
            url,
            "test-key".to_string(),
            models,
            Duration::from_millis(300),
        )
    }

    #[test]
    fn the_message_has_the_model_the_text_and_the_question() {
        let body = build_req_body("some/model", "my password is x", QUESTION);
        assert_eq!(body["model"], "some/model");
        assert_eq!(body["state"], "my password is x");
        assert_eq!(body["questions"][QUESTION_ID]["type"], "noul");
        assert_eq!(body["questions"][QUESTION_ID]["instructions"], QUESTION);
    }

    #[test]
    fn a_good_reply_gives_the_chance() {
        let reply: Value = serde_json::from_str(&reply_with_chance(0.97)).unwrap();
        let chance = parse_probability_from_response(&reply).unwrap();
        assert!((chance - 0.97).abs() < 1e-6);
    }

    #[test]
    fn odd_replies_give_nothing() {
        let odd = [
            json!({}),
            json!({ "answers": {} }),
            json!({ "answers": { QUESTION_ID: { "type": "noul" } } }),
            json!({ "answers": { QUESTION_ID: { "noul": "high" } } }),
            json!({ "answers": { QUESTION_ID: { "noul": 1.5 } } }),
            json!({ "answers": { QUESTION_ID: { "noul": -0.2 } } }),
            json!("just text"),
        ];
        for reply in odd {
            assert_eq!(parse_probability_from_response(&reply), None, "{reply}");
        }
    }

    #[tokio::test]
    async fn a_working_model_gives_a_decision() {
        let url = fake_server(vec![(200, Some(reply_with_chance(0.99)))]).await;
        let decision = decider_at(url, &["one"])
            .ask_yes_or_no("text", QUESTION)
            .await
            .unwrap();
        assert_eq!(decision.verdict, Verdict::Yes);
    }

    #[tokio::test]
    async fn the_next_model_is_tried_when_the_first_fails() {
        let url = fake_server(vec![
            (500, Some("oops".into())),
            (200, Some(reply_with_chance(0.01))),
        ])
        .await;
        let decision = decider_at(url, &["broken", "working"])
            .ask_yes_or_no("text", QUESTION)
            .await
            .unwrap();
        assert_eq!(decision.verdict, Verdict::No);
    }

    #[tokio::test]
    async fn a_model_that_stays_silent_is_given_up_on() {
        let url = fake_server(vec![(200, None), (200, Some(reply_with_chance(0.5)))]).await;
        let started = Instant::now();
        let decision = decider_at(url, &["silent", "working"])
            .ask_yes_or_no("text", QUESTION)
            .await
            .unwrap();
        assert_eq!(decision.verdict, Verdict::Unsure);
        assert!(started.elapsed() < Duration::from_secs(3));
    }

    #[tokio::test]
    async fn a_reply_we_cannot_use_counts_as_no_answer() {
        let url = fake_server(vec![(200, Some("not json".into()))]).await;
        assert_eq!(
            decider_at(url, &["one"]).ask_yes_or_no("text", QUESTION).await,
            None
        );
    }

    #[tokio::test]
    async fn nothing_listening_gives_no_answer_and_no_crash() {
        let decider = decider_at("http://127.0.0.1:1/decisions".to_string(), &["one", "two"]);
        assert_eq!(decider.ask_yes_or_no("text", QUESTION).await, None);
    }

    #[tokio::test]
    async fn no_models_gives_no_answer() {
        let decider = decider_at("http://127.0.0.1:1/decisions".to_string(), &[]);
        assert_eq!(decider.ask_yes_or_no("text", QUESTION).await, None);
    }

    #[tokio::test]
    #[ignore]
    async fn the_real_models_answer() {
        dotenvy::dotenv().ok();
        let decider = DecisionClient::shared();
        let secret = decider
            .ask_yes_or_no("my password is hunter2", QUESTION)
            .await
            .expect("no decision model answered");
        let plain = decider
            .ask_yes_or_no("Let's meet at 5pm tomorrow near the cafe", QUESTION)
            .await
            .expect("no decision model answered");
        println!("secret: {secret:?}\nplain: {plain:?}");
        assert_eq!(secret.verdict, Verdict::Yes);
        assert_eq!(plain.verdict, Verdict::No);
    }
}
