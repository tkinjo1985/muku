use crate::llm::{build_messages, call_chat, HistoryMessage, LlmResponse, TaskContext};

#[tauri::command]
pub async fn chat_send(
    input: String,
    active_tasks: Vec<TaskContext>,
    history: Vec<HistoryMessage>,
) -> Result<LlmResponse, String> {
    let messages = build_messages(&active_tasks, &history, &input);
    call_chat(&messages).await
}
