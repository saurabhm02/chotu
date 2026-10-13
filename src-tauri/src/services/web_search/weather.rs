use serde_json::Value;

use super::get_backup;
use crate::config::{WEATHER_DAYS, WEATHER_PLACE_TRIES};
use crate::models::web::SerpItem;

const WEATHER_WORDS: [&str; 9] = [
    "weather",
    "forecast",
    "temperature",
    "rain",
    "raining",
    "humidity",
    "snow",
    "sunny",
    "windy",
];

/// Words that are in a weather question but are not part of the place name.
const FILLER_WORDS: [&str; 41] = [
    "what",
    "whats",
    "what's",
    "is",
    "are",
    "the",
    "in",
    "at",
    "for",
    "of",
    "like",
    "going",
    "to",
    "be",
    "will",
    "it",
    "its",
    "it's",
    "how",
    "today",
    "tomorrow",
    "tonight",
    "now",
    "morning",
    "evening",
    "afternoon",
    "this",
    "week",
    "weekend",
    "outside",
    "hot",
    "cold",
    "there",
    "right",
    "current",
    "currently",
    "please",
    "tell",
    "me",
    "about",
    "a",
];

struct Place {
    label: String,
    latitude: f64,
    longitude: f64,
}

fn split_into_words(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split_whitespace()
        .map(|word| {
            word.trim_matches(|c: char| !c.is_alphanumeric())
                .to_string()
        })
        .filter(|word| !word.is_empty())
        .collect()
}

pub fn is_weather_question(question: &str) -> bool {
    split_into_words(question)
        .iter()
        .any(|word| WEATHER_WORDS.contains(&word.as_str()))
}

/// Guesses for the place name, best first: what is left of the question once the weather
/// and filler words are removed, then each single word of it.
/// "what is the weather in new delhi tomorrow" gives ["new delhi", "new", "delhi"].
pub fn place_candidates(question: &str) -> Vec<String> {
    let place_words: Vec<String> = split_into_words(question)
        .into_iter()
        .filter(|word| {
            !WEATHER_WORDS.contains(&word.as_str()) && !FILLER_WORDS.contains(&word.as_str())
        })
        .collect();
    if place_words.is_empty() {
        return Vec::new();
    }

    let mut candidates = vec![place_words
        .iter()
        .take(3)
        .cloned()
        .collect::<Vec<_>>()
        .join(" ")];
    for word in place_words {
        if !candidates.contains(&word) {
            candidates.push(word);
        }
    }
    candidates.truncate(WEATHER_PLACE_TRIES);
    candidates
}

/// The first match of a place lookup reply.
fn parse_place(body: &Value) -> Option<Place> {
    let first = body["results"].as_array()?.first()?;
    let mut parts: Vec<&str> = Vec::new();
    for key in ["name", "admin1", "country"] {
        if let Some(part) = first[key].as_str().filter(|part| !part.is_empty()) {
            if !parts.contains(&part) {
                parts.push(part);
            }
        }
    }
    Some(Place {
        label: parts.join(", "),
        latitude: first["latitude"].as_f64()?,
        longitude: first["longitude"].as_f64()?,
    })
}

/// WMO weather codes, in words.
fn weather_text(code: i64) -> &'static str {
    match code {
        0 => "clear sky",
        1 => "mainly clear",
        2 => "partly cloudy",
        3 => "overcast",
        45 | 48 => "fog",
        51..=57 => "drizzle",
        61..=67 => "rain",
        71..=77 => "snow",
        80..=82 => "rain showers",
        85 | 86 => "snow showers",
        95..=99 => "thunderstorm",
        _ => "unknown conditions",
    }
}

/// A forecast reply as one short paragraph. `None` when it has no daily numbers.
fn describe_forecast(label: &str, forecast: &Value) -> Option<String> {
    let daily = &forecast["daily"];
    let days = daily["time"].as_array()?;
    if days.is_empty() {
        return None;
    }

    let mut text = format!("Weather for {label}.");
    if let Some(now) = forecast["current"]["temperature_2m"].as_f64() {
        let code = forecast["current"]["weather_code"].as_i64().unwrap_or(-1);
        text.push_str(&format!(" Now: {now:.0}°C, {}.", weather_text(code)));
    }
    for (index, date) in days.iter().take(WEATHER_DAYS).enumerate() {
        let date = date.as_str().unwrap_or("?");
        let high = daily["temperature_2m_max"][index].as_f64();
        let low = daily["temperature_2m_min"][index].as_f64();
        let code = daily["weather_code"][index].as_i64().unwrap_or(-1);
        let rain = daily["precipitation_probability_max"][index].as_f64();
        if let (Some(high), Some(low)) = (high, low) {
            text.push_str(&format!(
                " {date}: {low:.0} to {high:.0}°C, {}",
                weather_text(code)
            ));
            if let Some(rain) = rain {
                text.push_str(&format!(", chance of rain {rain:.0}%"));
            }
            text.push('.');
        }
    }
    Some(text)
}

async fn find_place(name: &str, lang: &str) -> Option<Place> {
    let language = lang.split('-').next().unwrap_or("en");
    let response = get_backup(
        "Open-Meteo places",
        "https://geocoding-api.open-meteo.com/v1/search",
        &[
            ("name", name),
            ("count", "1"),
            ("language", language),
            ("format", "json"),
        ],
    )
    .await?;
    parse_place(&response.json::<Value>().await.ok()?)
}

pub async fn search(query: &str, lang: &str) -> Vec<SerpItem> {
    for candidate in place_candidates(query) {
        let Some(place) = find_place(&candidate, lang).await else {
            continue;
        };

        let latitude = place.latitude.to_string();
        let longitude = place.longitude.to_string();
        let days = WEATHER_DAYS.to_string();
        let response = get_backup(
            "Open-Meteo forecast",
            "https://api.open-meteo.com/v1/forecast",
            &[
                ("latitude", &latitude),
                ("longitude", &longitude),
                ("current", "temperature_2m,weather_code"),
                (
                    "daily",
                    "weather_code,temperature_2m_max,temperature_2m_min,precipitation_probability_max",
                ),
                ("timezone", "auto"),
                ("forecast_days", &days),
            ],
        )
        .await;
        let Some(response) = response else {
            return Vec::new();
        };
        let Ok(forecast) = response.json::<Value>().await else {
            return Vec::new();
        };

        return describe_forecast(&place.label, &forecast)
            .map(|content| {
                vec![SerpItem {
                    title: format!("Weather forecast for {} (Open-Meteo)", place.label),
                    source: "https://open-meteo.com/".to_string(),
                    content,
                    needs_page: false,
                }]
            })
            .unwrap_or_default();
    }
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn weather_questions_are_recognised() {
        assert!(is_weather_question("what is the weather in Pune tomorrow?"));
        assert!(is_weather_question("will it rain in Delhi"));
        assert!(!is_weather_question("who won the match yesterday"));
    }

    #[test]
    fn the_place_is_what_is_left_of_the_question() {
        assert_eq!(
            place_candidates("what is the weather going to be like in pune tomorrow morning"),
            vec!["pune"]
        );
        assert_eq!(
            place_candidates("weather in New Delhi"),
            vec!["new delhi", "new", "delhi"]
        );
    }

    #[test]
    fn a_question_without_a_place_has_no_candidates() {
        assert!(place_candidates("what is the weather today").is_empty());
    }

    #[test]
    fn a_place_is_read_with_its_region_and_country() {
        let reply = json!({"results":[{"name":"Pune","admin1":"Maharashtra","country":"India",
            "latitude":18.51957,"longitude":73.85535}]});
        let place = parse_place(&reply).unwrap();
        assert_eq!(place.label, "Pune, Maharashtra, India");
        assert!((place.latitude - 18.51957).abs() < 1e-9);
        assert!(parse_place(&json!({})).is_none());
    }

    #[test]
    fn a_forecast_becomes_one_short_paragraph() {
        let forecast = json!({
            "current": {"temperature_2m": 31.2, "weather_code": 2},
            "daily": {
                "time": ["2026-10-13", "2026-10-14"],
                "weather_code": [2, 61],
                "temperature_2m_max": [33.6, 30.1],
                "temperature_2m_min": [23.3, 22.0],
                "precipitation_probability_max": [0.0, 60.0]
            }
        });
        let text = describe_forecast("Pune, Maharashtra, India", &forecast).unwrap();
        assert!(text.starts_with("Weather for Pune, Maharashtra, India."));
        assert!(text.contains("Now: 31°C, partly cloudy."));
        assert!(text.contains("2026-10-13: 23 to 34°C, partly cloudy, chance of rain 0%."));
        assert!(text.contains("2026-10-14: 22 to 30°C, rain, chance of rain 60%."));
    }

    #[test]
    fn a_forecast_without_days_gives_nothing() {
        assert!(describe_forecast("X", &json!({})).is_none());
        assert!(describe_forecast("X", &json!({"daily": {"time": []}})).is_none());
    }
}
