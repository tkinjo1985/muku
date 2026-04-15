use serde::{Deserialize, Serialize};

pub const SYSTEM_PROMPT: &str = r#"あなたは「ムク」という名前のAIタスクマネージャーです。
ユーザーのタスク管理を自然な会話を通じて行います。

必ず以下のJSON形式で応答してください。他のテキストは含めないでください：
{
  "message": "ユーザーへの応答メッセージ",
  "actions": [
    {
      "type": "add" | "complete" | "delete" | "update",
      "task": {
        "id": "一意のID（addの場合は省略可）",
        "title": "タスクのタイトル",
        "priority": "high" | "medium" | "low",
        "due": "自然言語の期限（例: 今日中, 明日朝）",
        "due_at": "ISO 8601 の期限日時（例: 2026-04-16T09:00:00+09:00）",
        "category": "カテゴリ（任意）"
      }
    }
  ]
}

ルール:
- actionsが不要な会話では空配列を返す
- complete と delete では id のみ必須
- update では id と変更するフィールドのみ含める
- 文脈から priority, category, due を推測する
- due が明確に判明する場合は ISO 8601 (JST +09:00) で due_at を必ず埋める
- due が曖昧な場合は due のみでよい（due_at は省略）
- ユーザーの言語に合わせて応答する
- 簡潔かつ親しみやすいトーンで"#;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TaskContext {
    pub id: String,
    pub title: String,
    pub priority: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due_at: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HistoryMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

pub fn build_system_content(now: &str, active_tasks: &[TaskContext]) -> String {
    let tasks_json = serde_json::to_string_pretty(active_tasks).unwrap_or_else(|_| "[]".into());
    format!("{SYSTEM_PROMPT}\n\n現在時刻 (JST): {now}\n\n現在のアクティブなタスク一覧:\n{tasks_json}")
}

pub fn build_messages(
    now: &str,
    active_tasks: &[TaskContext],
    history: &[HistoryMessage],
    user_input: &str,
) -> Vec<ChatMessage> {
    let mut msgs = Vec::with_capacity(history.len() + 2);
    msgs.push(ChatMessage {
        role: "system".into(),
        content: build_system_content(now, active_tasks),
    });
    for m in history {
        msgs.push(ChatMessage {
            role: m.role.clone(),
            content: m.content.clone(),
        });
    }
    msgs.push(ChatMessage {
        role: "user".into(),
        content: user_input.to_string(),
    });
    msgs
}
