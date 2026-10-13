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
        let mut request = self
            .client
            .post(&self.url)
            .bearer_auth(&self.key)
            .json(body);
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
    use std::sync::{Arc, Mutex};
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
            decider_at(url, &["one"])
                .ask_yes_or_no("text", QUESTION)
                .await,
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

    struct Visit {
        status: u16,
        body: Option<String>,
        delay: Duration,
    }

    fn answer_after(body: String, delay_ms: u64) -> Visit {
        Visit {
            status: 200,
            body: Some(body),
            delay: Duration::from_millis(delay_ms),
        }
    }

    async fn read_request_body(socket: &mut tokio::net::TcpStream) -> Value {
        let mut seen = Vec::new();
        let mut chunk = [0u8; 4096];
        loop {
            let count = socket.read(&mut chunk).await.unwrap();
            if count == 0 {
                return Value::Null;
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
                    return serde_json::from_str(&text[head_end + 4..]).unwrap_or(Value::Null);
                }
            }
        }
    }

    async fn recording_server(visits: Vec<Visit>) -> (String, Arc<Mutex<Vec<Value>>>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/decisions", listener.local_addr().unwrap());
        let received = Arc::new(Mutex::new(Vec::new()));
        let record = received.clone();
        tokio::spawn(async move {
            for visit in visits {
                let (mut socket, _) = listener.accept().await.unwrap();
                let body = read_request_body(&mut socket).await;
                record.lock().unwrap().push(body);
                tokio::time::sleep(visit.delay).await;
                match visit.body {
                    Some(body) => {
                        let answer = format!(
                            "HTTP/1.1 {} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                            visit.status,
                            body.len()
                        );
                        let _ = socket.write_all(answer.as_bytes()).await;
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
        (url, received)
    }

    fn kind_question() -> DecisionQuestion {
        DecisionQuestion::choice(
            "kind",
            "What kind of content is this?",
            &[
                ("error", "an error message"),
                ("code", "source code"),
                ("other", "anything else"),
            ],
        )
    }

    fn quality_question() -> DecisionQuestion {
        DecisionQuestion::score(
            "quality",
            "How clear is this?",
            &["very unclear", "unclear", "clear", "very clear"],
        )
    }

    fn file_question() -> DecisionQuestion {
        DecisionQuestion::yes_no("has_file", "Does the text name a file?")
    }

    fn three_questions() -> Vec<DecisionQuestion> {
        vec![kind_question(), quality_question(), file_question()]
    }

    fn real_shaped_reply() -> Value {
        json!({
            "model": "inception/mercury-decide-20260930",
            "answers": {
                "kind": {
                    "type": "choice",
                    "choice": "error",
                    "probabilities": { "error": 0.9993, "code": 0.0002, "other": 0.0005 },
                    "confidence": 0.999
                },
                "quality": {
                    "type": "score",
                    "score": 1.99,
                    "legend": { "0": "very unclear", "1": "unclear", "2": "clear", "3": "very clear" },
                    "probabilities": { "0": 0.009, "1": 0.069, "2": 0.843, "3": 0.078 },
                    "confidence": 0.83
                },
                "has_file": { "type": "noul", "noul": 0.996 }
            },
            "usage": { "input_tokens": 101, "output_tokens": 3, "cost": 0 }
        })
    }

    #[test]
    fn the_questions_body_has_every_type_in_the_api_shape() {
        let body = build_questions_body("some/model", "the text", &three_questions());
        assert_eq!(body["model"], "some/model");
        assert_eq!(body["state"], "the text");

        let kind = &body["questions"]["kind"];
        assert_eq!(kind["type"], "choice");
        assert_eq!(kind["instructions"], "What kind of content is this?");
        assert_eq!(kind["criteria"]["error"], "an error message");
        assert_eq!(kind["criteria"]["other"], "anything else");

        let quality = &body["questions"]["quality"];
        assert_eq!(quality["type"], "score");
        assert_eq!(quality["criteria"][0], "very unclear");
        assert_eq!(quality["criteria"][3], "very clear");

        let has_file = &body["questions"]["has_file"];
        assert_eq!(has_file["type"], "noul");
        assert!(has_file.get("criteria").is_none());
    }

    #[test]
    fn a_real_shaped_reply_is_read_for_every_type() {
        let answers = parse_answers(&real_shaped_reply(), &three_questions());
        assert_eq!(answers.len(), 3);

        let kind = answers.choice("kind").unwrap();
        assert_eq!(kind.chosen, "error");
        assert_eq!(kind.probabilities.len(), 3);
        assert!((kind.probability_of("code").unwrap() - 0.0002).abs() < 1e-6);
        assert!(kind.confidence > 0.99);

        let quality = answers.score("quality").unwrap();
        assert_eq!(quality.level, 2);
        assert_eq!(quality.label, "clear");
        assert!((quality.value - 1.99).abs() < 1e-6);
        assert_eq!(quality.probabilities.len(), 4);

        assert_eq!(answers.yes_no("has_file").unwrap().verdict, Verdict::Yes);
    }

    #[test]
    fn an_answer_that_does_not_fit_the_question_is_left_out() {
        let reply = json!({ "answers": {
            "kind": { "type": "choice", "choice": "banana", "probabilities": {}, "confidence": 1.0 },
            "quality": { "type": "score", "score": 7.0, "probabilities": {}, "confidence": 1.0 },
            "has_file": { "type": "noul", "noul": 1.5 }
        }});
        let answers = parse_answers(&reply, &three_questions());
        assert!(answers.is_empty());
    }

    #[test]
    fn a_missing_chance_counts_as_zero_and_a_score_is_rounded_to_a_level() {
        let reply = json!({ "answers": {
            "kind": { "type": "choice", "choice": "code" },
            "quality": { "type": "score", "score": 0.4 }
        }});
        let answers = parse_answers(&reply, &[kind_question(), quality_question()]);

        let kind = answers.choice("kind").unwrap();
        assert_eq!(kind.probability_of("code"), Some(0.0));
        assert_eq!(kind.confidence, 0.0);

        let quality = answers.score("quality").unwrap();
        assert_eq!(quality.level, 0);
        assert_eq!(quality.label, "very unclear");
    }

    #[tokio::test]
    async fn several_questions_are_answered_in_one_call() {
        let (url, received) =
            recording_server(vec![answer_after(real_shaped_reply().to_string(), 0)]).await;
        let answers = decider_at(url, &["inception/mercury-decide:free"])
            .ask_questions("the text", &three_questions(), Duration::from_secs(2))
            .await
            .unwrap();

        assert_eq!(answers.len(), 3);
        let requests = received.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0]["questions"].as_object().unwrap().len(), 3);
    }

    #[tokio::test]
    async fn a_yes_no_only_model_is_skipped_for_choice_and_score_questions() {
        let (url, received) =
            recording_server(vec![answer_after(real_shaped_reply().to_string(), 0)]).await;
        let decider = decider_at(
            url,
            &["respan/span-01-lite:free", "inception/mercury-decide:free"],
        );
        let answers = decider
            .ask_questions("the text", &three_questions(), Duration::from_secs(2))
            .await;

        assert!(answers.is_some());
        let requests = received.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0]["model"], "inception/mercury-decide:free");
    }

    #[tokio::test]
    async fn yes_no_questions_still_try_the_yes_no_model_first() {
        let reply = json!({ "answers": { "has_file": { "type": "noul", "noul": 0.1 } } });
        let (url, received) = recording_server(vec![answer_after(reply.to_string(), 0)]).await;
        let decider = decider_at(
            url,
            &["respan/span-01-lite:free", "inception/mercury-decide:free"],
        );
        let answers = decider
            .ask_questions("the text", &[file_question()], Duration::from_secs(2))
            .await
            .unwrap();

        assert_eq!(answers.yes_no("has_file").unwrap().verdict, Verdict::No);
        assert_eq!(
            received.lock().unwrap()[0]["model"],
            "respan/span-01-lite:free"
        );
    }

    #[tokio::test]
    async fn a_model_that_misses_a_question_is_followed_by_the_next_one() {
        let partial = json!({ "answers": { "has_file": { "type": "noul", "noul": 0.9 } } });
        let (url, received) = recording_server(vec![
            answer_after(partial.to_string(), 0),
            answer_after(real_shaped_reply().to_string(), 0),
        ])
        .await;
        let answers = decider_at(url, &["first", "second"])
            .ask_questions("the text", &three_questions(), Duration::from_secs(2))
            .await
            .unwrap();

        assert_eq!(answers.len(), 3);
        assert_eq!(received.lock().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn when_no_model_answers_everything_the_fullest_reply_is_kept() {
        let one = json!({ "answers": { "has_file": { "type": "noul", "noul": 0.9 } } });
        let two = json!({ "answers": {
            "has_file": { "type": "noul", "noul": 0.9 },
            "kind": { "type": "choice", "choice": "code", "probabilities": {}, "confidence": 0.5 }
        }});
        let (url, _) = recording_server(vec![
            answer_after(one.to_string(), 0),
            answer_after(two.to_string(), 0),
        ])
        .await;
        let answers = decider_at(url, &["first", "second"])
            .ask_questions("the text", &three_questions(), Duration::from_secs(2))
            .await
            .unwrap();

        assert_eq!(answers.len(), 2);
        assert!(answers.choice("kind").is_some());
    }

    #[tokio::test]
    async fn the_call_gets_its_own_time_limit() {
        let reply = real_shaped_reply().to_string();

        let (slow_url, _) = recording_server(vec![answer_after(reply, 600)]).await;
        let patient = decider_at(slow_url, &["one"])
            .ask_questions("text", &three_questions(), Duration::from_secs(3))
            .await;
        assert!(
            patient.is_some(),
            "a longer limit for the call should be used"
        );

        let (silent_url, _) = recording_server(vec![Visit {
            status: 200,
            body: None,
            delay: Duration::ZERO,
        }])
        .await;
        let started = Instant::now();
        let impatient = decider_at(silent_url, &["one"])
            .ask_questions("text", &three_questions(), Duration::from_millis(150))
            .await;
        assert!(impatient.is_none());
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    #[tokio::test]
    async fn no_questions_or_repeated_ids_ask_nothing() {
        let (url, received) = recording_server(vec![]).await;
        let decider = decider_at(url, &["one"]);
        let timeout = Duration::from_secs(1);

        assert_eq!(decider.ask_questions("text", &[], timeout).await, None);
        let repeated = [file_question(), file_question()];
        assert_eq!(
            decider.ask_questions("text", &repeated, timeout).await,
            None
        );
        assert!(received.lock().unwrap().is_empty());
    }

    #[tokio::test]
    #[ignore]
    async fn several_real_questions_in_one_call() {
        dotenvy::dotenv().ok();
        let decider = DecisionClient::shared();
        let text = "TypeError: cannot read properties of undefined (reading 'map') at App.js:42";
        let answers = decider
            .ask_questions(text, &three_questions(), Duration::from_secs(10))
            .await
            .expect("no decision model answered");

        println!("{answers:#?}");
        assert_eq!(answers.choice("kind").unwrap().chosen, "error");
        assert!(answers.score("quality").is_some());
        assert_eq!(answers.yes_no("has_file").unwrap().verdict, Verdict::Yes);
    }
}
