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

#[derive(Serialize, Clone, Debug)]
pub struct LlmResponse {
    pub message: String,
    pub actions: Vec<TaskAction>,
}

#[derive(Serialize, Clone, Debug)]
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
    pub due_at: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
}

#[derive(Deserialize)]
struct RawAction {
    #[serde(rename = "type")]
    action_type: String,
    #[serde(default)]
    task: Option<TaskPayload>,
    #[serde(flatten, default)]
    flat: TaskPayload,
}

impl From<RawAction> for TaskAction {
    fn from(r: RawAction) -> Self {
        let task = match r.task {
            Some(mut t) => {
                t.id = t.id.or(r.flat.id);
                t.title = t.title.or(r.flat.title);
                t.priority = t.priority.or(r.flat.priority);
                t.due = t.due.or(r.flat.due);
                t.due_at = t.due_at.or(r.flat.due_at);
                t.category = t.category.or(r.flat.category);
                t
            }
            None => r.flat,
        };
        TaskAction {
            action_type: r.action_type,
            task,
        }
    }
}

#[derive(Deserialize)]
struct RawLlmResponse {
    message: String,
    #[serde(default)]
    actions: Vec<RawAction>,
}

impl From<RawLlmResponse> for LlmResponse {
    fn from(r: RawLlmResponse) -> Self {
        LlmResponse {
            message: r.message,
            actions: r.actions.into_iter().map(TaskAction::from).collect(),
        }
    }
}

pub async fn call_chat(messages: &[ChatMessage]) -> Result<LlmResponse, String> {
    let body = ChatRequest {
        model: "gemma-4-e2b-it",
        messages,
        temperature: 0.3,
        max_tokens: 4096,
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

    serde_json::from_str::<RawLlmResponse>(&content)
        .map(LlmResponse::from)
        .map_err(|e| format!("Failed to parse LLM JSON: {e}. Raw content: {content}"))
}
