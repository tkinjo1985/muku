use serde::{Deserialize, Serialize};

use super::prompt::ChatMessage;

const LLAMA_URL: &str = "http://127.0.0.1:18080/v1/chat/completions";

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: &'a [ChatMessage],
    temperature: f32,
    max_tokens: u32,
    response_format: ResponseFormat<'a>,
}

#[derive(Serialize)]
struct ResponseFormat<'a> {
    #[serde(rename = "type")]
    kind: &'a str,
}

#[derive(Deserialize)]
struct ChatCompletion {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChoiceMessage,
}

#[derive(Deserialize)]
struct ChoiceMessage {
    content: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct LlmResponse {
    pub message: String,
    #[serde(default)]
    pub actions: Vec<TaskAction>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TaskAction {
    #[serde(rename = "type")]
    pub action_type: String,
    pub task: TaskPayload,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct TaskPayload {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(default)]
    pub due: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
}

pub async fn call_chat(messages: &[ChatMessage]) -> Result<LlmResponse, String> {
    let body = ChatRequest {
        model: "gemma-4-e4b-it",
        messages,
        temperature: 0.3,
        max_tokens: 1024,
        response_format: ResponseFormat { kind: "json_object" },
    };

    let client = reqwest::Client::new();
    let resp = client
        .post(LLAMA_URL)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("LLM request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("LLM returned {status}: {text}"));
    }

    let completion: ChatCompletion = resp
        .json()
        .await
        .map_err(|e| format!("Invalid LLM response: {e}"))?;

    let content = completion
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "LLM returned empty choices".to_string())?;

    serde_json::from_str::<LlmResponse>(&content).map_err(|e| {
        format!("Failed to parse LLM JSON: {e}. Raw content: {content}")
    })
}
