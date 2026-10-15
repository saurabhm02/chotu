use std::time::Duration;

use futures::future::BoxFuture;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolRisk {
    Read,
    Change,
    Dangerous,
}

/// One text argument the model must give. Every parameter is required.
#[derive(Debug, Clone)]
pub struct ToolParameter {
    pub name: String,
    pub description: String,
}

pub type ToolFunction =
    Box<dyn Fn(Value) -> BoxFuture<'static, Result<String, String>> + Send + Sync>;

pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ToolParameter>,
    pub risk: ToolRisk,
    pub time_limit: Duration,
    pub max_result_chars: usize,
    pub function: ToolFunction,
}
