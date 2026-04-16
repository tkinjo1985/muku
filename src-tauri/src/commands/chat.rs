use chrono::{FixedOffset, Utc};

use crate::llm::{build_messages, call_chat, HistoryMessage, LlmResponse, TaskContext};

fn now_jst() -> String {
    let jst = FixedOffset::east_opt(9 * 3600).unwrap();
    Utc::now()
        .with_timezone(&jst)
        .format("%Y-%m-%dT%H:%M:%S%:z")
        .to_string()
}

#[tauri::command]
pub async fn chat_send(
    input: String,
    active_tasks: Vec<TaskContext>,
    history: Vec<HistoryMessage>,
    username: String,
) -> Result<LlmResponse, String> {
    let now = now_jst();
    let uname = if username.trim().is_empty() { None } else { Some(username.trim().to_string()) };
    let messages = build_messages(&now, &active_tasks, &history, &input, uname.as_deref());
    call_chat(&messages).await
}
