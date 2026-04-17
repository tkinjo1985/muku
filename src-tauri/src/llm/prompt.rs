use serde::{Deserialize, Serialize};

pub const SYSTEM_PROMPT: &str = r#"あなたは「ムク」という名前のAIタスクマネージャーです。
ユーザーのタスク管理を自然な会話を通じて行います。

必ず以下のJSON形式で応答してください。他のテキストは含めないでください。
各 action の詳細フィールドは必ず "task" オブジェクトの中に入れます。

スキーマ:
{
  "message": "ユーザーへの応答メッセージ",
  "actions": [
    {
      "type": "add" | "complete" | "delete" | "update",
      "task": {
        "id": "既存タスクのID（complete/delete/update では必須）",
        "title": "タスクのタイトル",
        "priority": "high" | "medium" | "low",
        "due": "自然言語の期限（例: 今日中, 明日朝）",
        "due_at": "ISO 8601 の期限日時（例: 2026-04-16T09:00:00+09:00）",
        "category": "カテゴリ（任意）"
      }
    }
  ]
}

例 1（タスク追加）:
{
  "message": "追加しました！",
  "actions": [
    { "type": "add", "task": { "title": "資料作成", "priority": "high", "due": "明日まで", "due_at": "2026-04-16T23:59:00+09:00" } }
  ]
}

例 2（タスク完了）:
{
  "message": "お疲れ様でした！",
  "actions": [
    { "type": "complete", "task": { "id": "既存のID" } }
  ]
}

例 3（タスク一覧を聞かれた場合）:
{
  "message": "現在のタスク一覧です：\n\n1. 資料作成（高）- 明日まで\n2. 買い物（低）\n\n全2件",
  "actions": []
}

例 4（アクション不要な雑談）:
{
  "message": "こんにちは！",
  "actions": []
}

ルール:
- 必ず action の詳細は "task" オブジェクトの中に入れる（id を直接 action の下に書かない）
- actionsが不要な会話では空配列を返す
- タスク一覧を聞かれたら、現在のアクティブなタスクの内容を message 内にリスト形式で記載する（タスクがなければ「タスクはありません」と伝える）
- complete と delete では task.id のみ必須
- update では task.id と変更するフィールドのみ含める
- 文脈から priority, category, due を推測する
- due が明確な場合は ISO 8601 (JST +09:00) で due_at を埋める
- due が曖昧な場合は due のみでよい（due_at は省略）
- ユーザーの言語に合わせて応答する
- 簡潔かつ親しみやすいトーンで
- 絵文字は使わない（トークン節約のため）

/no_think"#;

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

pub fn build_system_content(now: &str, active_tasks: &[TaskContext], username: Option<&str>) -> String {
    let tasks_json = serde_json::to_string_pretty(active_tasks).unwrap_or_else(|_| "[]".into());
    let user_line = match username {
        Some(name) => format!("\nユーザーの呼び名: {name}（会話中この名前で呼ぶこと）"),
        None => String::new(),
    };
    format!("{SYSTEM_PROMPT}\n\n現在時刻 (JST): {now}{user_line}\n\n現在のアクティブなタスク一覧:\n{tasks_json}")
}

pub fn build_messages(
    now: &str,
    active_tasks: &[TaskContext],
    history: &[HistoryMessage],
    user_input: &str,
    username: Option<&str>,
) -> Vec<ChatMessage> {
    let mut msgs = Vec::with_capacity(history.len() + 2);
    msgs.push(ChatMessage {
        role: "system".into(),
        content: build_system_content(now, active_tasks, username),
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
