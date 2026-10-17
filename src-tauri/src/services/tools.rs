use serde_json::{json, Map, Value};

use crate::models::tools::{Tool, ToolParameter, ToolRisk};

pub struct ToolRegistry {
    tools: Vec<Tool>,
    allow_change_tools: bool,
}

impl ToolRegistry {
    pub fn new(allow_change_tools: bool) -> Self {
        Self {
            tools: Vec::new(),
            allow_change_tools,
        }
    }

    pub fn register(&mut self, tool: Tool) -> Result<(), String> {
        if self.find(&tool.name).is_some() {
            return Err(format!("tool `{}` is already registered", tool.name));
        }
        if tool.risk != ToolRisk::Read && !self.allow_change_tools {
            return Err(format!("tool `{}` is not a read-only tool", tool.name));
        }
        self.tools.push(tool);
        Ok(())
    }

    pub fn find(&self, name: &str) -> Option<&Tool> {
        self.tools.iter().find(|tool| tool.name == name)
    }

    pub fn definitions(&self) -> Vec<Value> {
        self.tools
            .iter()
            .map(|tool| {
                let properties: Map<String, Value> = tool
                    .parameters
                    .iter()
                    .map(|parameter| {
                        let schema = json!({
                            "type": "string",
                            "description": parameter.description,
                        });
                        (parameter.name.clone(), schema)
                    })
                    .collect();
                let required: Vec<&String> = tool
                    .parameters
                    .iter()
                    .map(|parameter| &parameter.name)
                    .collect();
                json!({
                    "type": "function",
                    "function": {
                        "name": tool.name,
                        "description": tool.description,
                        "parameters": {
                            "type": "object",
                            "properties": properties,
                            "required": required,
                        },
                    },
                })
            })
            .collect()
    }

    pub async fn run(&self, name: &str, raw_arguments: &str) -> Result<String, String> {
        let tool = self
            .find(name)
            .ok_or(format!("There is no tool named `{name}`."))?;
        let arguments = parse_arguments(&tool.parameters, raw_arguments)?;
        let result = tokio:: time::timeout(tool.time_limit, (tool.function)(arguments))
            .await
            .map_err(|_| format!("The tool `{name}` took too long and was stopped."))?;
        result.map(|text| cap_result(&text, tool.max_result_chars))
    }
}

fn parse_arguments(parameters: &[ToolParameter], raw_arguments: &str) -> Result<Value, String> {
    let value: Value = if raw_arguments.trim().is_empty() {
        json!({})
    } else {
        serde_json::from_str(raw_arguments)
            .map_err(|_| "The arguments are not valid JSON.".to_string())?
    };
    let object = value
        .as_object()
        .ok_or("The arguments must be a JSON object.")?;
    for parameter in parameters {
        if !object.get(&parameter.name).is_some_and(Value::is_string) {
            return Err(format!(
                "Argument `{}` is required and must be text.",
                parameter.name
            ));
        }
    }
    Ok(value)
}

fn cap_result(text: &str, max_chars: usize) -> String {
    let total = text.chars().count();
    if total <= max_chars {
        return text.to_string();
    }
    let kept: String = text.chars().take(max_chars).collect();
    format!("{kept}\n[{} characters omitted]", total - max_chars)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    fn tool_returning(name: &str, risk: ToolRisk, reply: &str, calls: Arc<AtomicUsize>) -> Tool {
        let reply = reply.to_string();
        Tool {
            name: name.to_string(),
            description: "a fake tool".to_string(),
            parameters: vec![ToolParameter {
                name: "text".to_string(),
                description: "some text".to_string(),
            }],
            risk,
            time_limit: Duration::from_secs(1),
            max_result_chars: 8000,
            function: Box::new(move |_| {
                calls.fetch_add(1, Ordering::SeqCst);
                let reply = reply.clone();
                Box::pin(async move { Ok(reply) })
            }),
        }
    }

    fn read_tool(name: &str) -> Tool {
        tool_returning(name, ToolRisk::Read, "ok", Arc::default())
    }

    #[test]
    fn definitions_use_the_openrouter_tool_format() {
        let mut registry = ToolRegistry::new(false);
        registry.register(read_tool("web_search")).unwrap();

        let definitions = registry.definitions();

        assert_eq!(definitions[0]["type"], "function");
        assert_eq!(definitions[0]["function"]["name"], "web_search");
        let parameters = &definitions[0]["function"]["parameters"];
        assert_eq!(parameters["properties"]["text"]["type"], "string");
        assert_eq!(parameters["required"], json!(["text"]));
    }

    #[test]
    fn the_same_name_cannot_be_registered_twice() {
        let mut registry = ToolRegistry::new(false);
        registry.register(read_tool("web_search")).unwrap();

        assert!(registry.register(read_tool("web_search")).is_err());
    }

    #[test]
    fn change_tools_are_refused_unless_allowed() {
        let change = tool_returning("move_file", ToolRisk::Change, "", Arc::default());
        let dangerous = tool_returning("run_command", ToolRisk::Dangerous, "", Arc::default());
        let mut closed = ToolRegistry::new(false);
        assert!(closed.register(change).is_err());
        assert!(closed.register(dangerous).is_err());

        let change = tool_returning("move_file", ToolRisk::Change, "", Arc::default());
        let mut open = ToolRegistry::new(true);
        assert!(open.register(change).is_ok());
    }

    #[tokio::test]
    async fn an_unknown_tool_is_an_error() {
        let registry = ToolRegistry::new(false);

        let error = registry.run("delete_everything", "{}").await.unwrap_err();

        assert!(error.contains("no tool named `delete_everything`"));
    }

    #[tokio::test]
    async fn bad_arguments_never_reach_the_tool() {
        let calls = Arc::new(AtomicUsize::new(0));
        let mut registry = ToolRegistry::new(false);
        let tool = tool_returning("echo", ToolRisk::Read, "ok", calls.clone());
        registry.register(tool).unwrap();

        for arguments in ["not json", "[1, 2]", "{}", r#"{"text": 5}"#] {
            assert!(
                registry.run("echo", arguments).await.is_err(),
                "{arguments}"
            );
        }
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn good_arguments_run_the_tool() {
        let calls = Arc::new(AtomicUsize::new(0));
        let mut registry = ToolRegistry::new(false);
        let tool = tool_returning("echo", ToolRisk::Read, "done", calls.clone());
        registry.register(tool).unwrap();

        let result = registry.run("echo", r#"{"text": "hi"}"#).await;

        assert_eq!(result, Ok("done".to_string()));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn empty_arguments_work_for_a_tool_without_parameters() {
        let mut registry = ToolRegistry::new(false);
        let mut tool = read_tool("get_current_time");
        tool.parameters.clear();
        registry.register(tool).unwrap();

        assert_eq!(
            registry.run("get_current_time", "").await,
            Ok("ok".to_string())
        );
    }

    #[tokio::test]
    async fn a_long_result_is_cut_with_a_marker() {
        let mut registry = ToolRegistry::new(false);
        let tool = tool_returning("big", ToolRisk::Read, &"a".repeat(20_000), Arc::default());
        registry.register(tool).unwrap();

        let result = registry.run("big", r#"{"text": "x"}"#).await.unwrap();

        assert!(result.starts_with(&"a".repeat(8000)));
        assert!(result.ends_with("[12000 characters omitted]"));
    }

    #[test]
    fn the_cap_counts_characters_not_bytes() {
        assert_eq!(cap_result("ééééééé", 4), "éééé\n[3 characters omitted]");
    }

    #[tokio::test]
    async fn a_slow_tool_is_stopped() {
        let mut registry = ToolRegistry::new(false);
        let mut tool = read_tool("slow");
        tool.time_limit = Duration::from_millis(20);
        tool.function = Box::new(|_| {
            Box::pin(async {
                tokio::time::sleep(Duration::from_millis(500)).await;
                Ok("late".to_string())
            })
        });
        registry.register(tool).unwrap();

        let error = registry.run("slow", r#"{"text": "x"}"#).await.unwrap_err();

        assert!(error.contains("took too long"));
    }

    #[tokio::test]
    async fn a_tool_error_reaches_the_model_as_an_error() {
        let mut registry = ToolRegistry::new(false);
        let mut tool = read_tool("broken");
        tool.function = Box::new(|_| Box::pin(async { Err("search failed".to_string()) }));
        registry.register(tool).unwrap();

        let error = registry
            .run("broken", r#"{"text": "x"}"#)
            .await
            .unwrap_err();

        assert_eq!(error, "search failed");
    }
}
