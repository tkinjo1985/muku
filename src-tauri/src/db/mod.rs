use tauri_plugin_sql::{Migration, MigrationKind};

pub fn migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            description: "create_tasks_and_messages",
            sql: include_str!("../../migrations/0001_init.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "add_due_at_and_last_notified_at",
            sql: include_str!("../../migrations/0002_due_at.sql"),
            kind: MigrationKind::Up,
        },
    ]
}
