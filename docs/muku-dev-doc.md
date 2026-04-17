# Muku（ムク）— 開発ドキュメント

## プロダクト概要

ローカルLLMを搭載したWindows向けデスクトップ常駐型AIタスク管理アプリ。
ユーザーはチャット（自然言語）でのみタスクを操作し、AIがタスクの登録・更新・優先度判定・進捗管理を自律的に行う。
タスク一覧は読み取り専用のリストビューで確認できる。

**コンセプト:** 「AIが管理する → UIで確認する」（従来のtodoアプリの主従を逆転）

---

## ターゲットユーザー

- PCで業務中のWindowsデスクトップワーカー
- マルチタスクで忙しく、タスク管理にUIを操作する時間を割きたくない層
- サブスク疲れ・プライバシー意識の高いユーザー

---

## 技術スタック

| レイヤー | 技術 | 備考 |
|---------|------|------|
| デスクトップフレームワーク | Tauri v2 | Rust + WebView2ベース。軽量で高速 |
| フロントエンド | React + TypeScript + Vite | Tauri公式テンプレート対応 |
| ローカルLLM推論 | llama.cpp (サーバーモード) | Tauriのsidecarとしてバンドル |
| LLMモデル | Qwen3.5 2B / 4B / 9B (GGUF, Q4_K_M) | 初回起動時にダウンロード。設定画面で切替可能 |
| データ永続化 | SQLite (via Tauri SQLプラグイン) | ローカルにタスク・会話履歴を保存 |
| 通知 | Tauri Notificationプラグイン | Windows ネイティブ通知 |
| グローバルショートカット | Tauri Global Shortcutプラグイン | ホットキーでウィンドウ呼び出し |
| システムトレイ | Tauri System Tray API | 常駐アプリとして動作 |
| 配布 | Microsoft Store | EXE/MSIインストーラー経由 |

---

## アーキテクチャ

```
┌─────────────────────────────────────────────────┐
│                  Tauri App                       │
│                                                  │
│  ┌──────────────────────┐  ┌──────────────────┐  │
│  │   Frontend (WebView) │  │  Rust Backend    │  │
│  │                      │  │                  │  │
│  │  ┌────────────────┐  │  │  ┌────────────┐  │  │
│  │  │  Chat View     │  │  │  │  Tauri      │  │  │
│  │  │  (入力・会話)   │◄─┼──┤  │  Commands  │  │  │
│  │  └────────────────┘  │  │  │            │  │  │
│  │                      │  │  └─────┬──────┘  │  │
│  │  ┌────────────────┐  │  │        │         │  │
│  │  │  Task View     │  │  │  ┌─────▼──────┐  │  │
│  │  │  (一覧・確認)   │◄─┼──┤  │  SQLite    │  │  │
│  │  └────────────────┘  │  │  │  (タスクDB) │  │  │
│  │                      │  │  └────────────┘  │  │
│  └──────────────────────┘  └───────┬──────────┘  │
│                                    │              │
│  ┌─────────────────────────────────▼────────────┐│
│  │         llama.cpp server (Sidecar)            ││
│  │         Qwen3.5 GGUF (2B / 4B / 9B)           ││
│  │         localhost:18080                        ││
│  └───────────────────────────────────────────────┘│
└─────────────────────────────────────────────────┘
```

### データフロー

1. ユーザーがチャットで自然言語を入力
2. Frontend → Tauri Command（invoke）でRustバックエンドに送信
3. Rustバックエンドが現在のタスク一覧をSQLiteから取得し、システムプロンプト＋ユーザー入力＋タスクコンテキストを構築
4. llama.cppサーバー（localhost:18080）にHTTP POSTで推論リクエスト
5. LLMがJSON形式（message + actions配列）で応答
6. Rustバックエンドがactionsをパースし、SQLiteのタスクデータを更新
7. 更新結果をFrontendに返却 → Chat ViewとTask Viewを更新

---

## プロジェクト構成

```
muku/
├── src/                          # Frontend (React + TypeScript)
│   ├── App.tsx                   # メインレイアウト（Chat / Tasks切り替え）
│   ├── components/
│   │   ├── ChatView.tsx          # チャットUI
│   │   ├── TaskListView.tsx      # タスク一覧（読み取り専用）
│   │   ├── MessageBubble.tsx     # メッセージ表示コンポーネント
│   │   └── TaskCard.tsx          # タスクカードコンポーネント
│   ├── hooks/
│   │   ├── useChat.ts            # チャットロジック
│   │   └── useTasks.ts           # タスク状態管理
│   ├── lib/
│   │   └── invoke.ts             # Tauri invoke ラッパー
│   ├── types/
│   │   └── index.ts              # 型定義
│   └── styles/
│       └── global.css            # グローバルスタイル
│
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json           # Tauri設定
│   ├── tauri.microsoftstore.conf.json  # MS Store用追加設定
│   ├── src/
│   │   ├── main.rs               # エントリーポイント
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── chat.rs           # チャット処理コマンド
│   │   │   └── tasks.rs          # タスクCRUDコマンド
│   │   ├── llm/
│   │   │   ├── mod.rs
│   │   │   ├── client.rs         # llama.cppサーバーとの通信
│   │   │   └── prompt.rs         # システムプロンプト構築
│   │   ├── db/
│   │   │   ├── mod.rs
│   │   │   └── models.rs         # タスク・会話のデータモデル
│   │   └── tray.rs               # システムトレイ設定
│   │
│   ├── binaries/                 # llama.cppサーバーバイナリ (sidecar)
│   │   └── llama-server-x86_64-pc-windows-msvc.exe
│   │
│   └── models/                   # LLMモデルファイル配置先
│       └── .gitkeep              # qwen*-Q4_K_M.gguf (git管理外)
│
├── package.json
├── tsconfig.json
├── vite.config.ts
└── README.md
```

---

## データモデル

### tasks テーブル

```sql
CREATE TABLE tasks (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    priority    TEXT NOT NULL DEFAULT 'medium',  -- 'high' | 'medium' | 'low'
    status      TEXT NOT NULL DEFAULT 'todo',    -- 'todo' | 'done'
    category    TEXT,
    due         TEXT,                            -- 自然言語の日時文字列
    created_at  TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);
```

### messages テーブル

```sql
CREATE TABLE messages (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    role        TEXT NOT NULL,     -- 'user' | 'assistant'
    content     TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);
```

---

## LLM連携仕様

### llama.cppサーバーの起動（sidecar）

Tauriのsidecar機能を使い、アプリ起動時にllama.cppサーバーをバックグラウンドで起動する。

```rust
// src-tauri/src/main.rs (概要)
use tauri::Manager;
use tauri_plugin_shell::ShellExt;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let sidecar = app.shell()
                .sidecar("llama-server")
                .expect("llama-server sidecar not found")
                .args([
                    "-m", "models/qwen3.5-4b-Q4_K_M.gguf",
                    "--host", "127.0.0.1",
                    "--port", "18080",
                    "-c", "4096",        // context length
                    "-ngl", "99",        // GPU layers (auto fallback)
                ]);

            let (_rx, _child) = sidecar.spawn()
                .expect("Failed to spawn llama-server");

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### システムプロンプト

```
あなたは「ムク」という名前のAIタスクマネージャーです。
ユーザーのタスク管理を自然な会話を通じて行います。

必ず以下のJSON形式で応答してください。他のテキストは含めないでください：
{
  "message": "ユーザーへの応答メッセージ",
  "actions": [
    {
      "type": "add" | "complete" | "delete" | "update",
      "task": {
        "id": "一意のID（addの場合は新規生成）",
        "title": "タスクのタイトル",
        "priority": "high" | "medium" | "low",
        "due": "期限（任意）",
        "category": "カテゴリ（任意）"
      }
    }
  ]
}

ルール:
- actionsが不要な会話では空配列を返す
- completeとdeleteではidのみ必須
- updateではidと変更するフィールドのみ含める
- 文脈からpriority、category、dueを推測する
- ユーザーの言語に合わせて応答する
- 簡潔かつ親しみやすいトーンで
```

### llama.cppへのリクエスト形式

```json
POST http://127.0.0.1:18080/v1/chat/completions
Content-Type: application/json

{
  "model": "qwen3.5",
  "messages": [
    {
      "role": "system",
      "content": "<システムプロンプト>\n\n現在のタスク一覧:\n<JSON>"
    },
    { "role": "user", "content": "..." },
    { "role": "assistant", "content": "..." },
    { "role": "user", "content": "<最新の入力>" }
  ],
  "temperature": 0.3,
  "max_tokens": 1024,
  "response_format": { "type": "json_object" }
}
```

### レスポンスのパース（Rust側）

```rust
#[derive(Deserialize)]
struct LlmResponse {
    message: String,
    actions: Vec<TaskAction>,
}

#[derive(Deserialize)]
struct TaskAction {
    #[serde(rename = "type")]
    action_type: String,  // "add" | "complete" | "delete" | "update"
    task: TaskData,
}

#[derive(Deserialize)]
struct TaskData {
    id: Option<String>,
    title: Option<String>,
    priority: Option<String>,
    due: Option<String>,
    category: Option<String>,
}
```

---

## UI仕様

### 全体レイアウト

- ウィンドウサイズ: 400 x 600（リサイズ可能、最小 360 x 500）
- ダークテーマ基調（背景 #0A0A0B）
- 上部ヘッダー: アプリ名 + Chat / Tasks タブ切り替え
- メインエリア: 選択中のビューを表示

### Chat View

- メッセージ一覧（上スクロール）
- 下部に入力フィールド + 送信ボタン
- サジェストチップ（「今日のタスクは？」「全部見せて」等）
- AI応答中はタイピングインジケーター（ドットアニメーション）
- ユーザーメッセージ: 右寄せ、紫系グラデーション背景
- AIメッセージ: 左寄せ、半透明白背景

### Task View（読み取り専用）

- Active / Done の2セクション
- 各タスクカード: タイトル、優先度ドット（赤/橙/緑）、期限、カテゴリタグ
- タスクの追加・編集・削除はできない（「タスクの変更はChatから」の案内表示）
- Done セクションは半透明 + 取り消し線

### グローバルショートカット

- `Alt + Space` でウィンドウの表示/非表示をトグル
- 入力フィールドに自動フォーカス

### システムトレイ

- トレイアイコンからの操作: 表示 / 非表示 / 終了
- 最小化時はトレイに格納（タスクバーから消える）

---

## Tauriプラグイン一覧

```json
// src-tauri/Cargo.toml に追加する依存関係
{
  "dependencies": {
    "tauri-plugin-shell": "2",          // sidecar (llama.cpp) 起動
    "tauri-plugin-sql": "2",            // SQLite
    "tauri-plugin-notification": "2",   // Windows通知
    "tauri-plugin-global-shortcut": "2",// ホットキー
    "tauri-plugin-autostart": "2",      // PC起動時の自動起動
    "tauri-plugin-store": "2",          // 設定の永続化
    "tauri-plugin-single-instance": "2",// 多重起動防止
    "tauri-plugin-window-state": "2"    // ウィンドウ位置・サイズの記憶
  }
}
```

---

## Tauri設定ファイル

### tauri.conf.json

```json
{
  "productName": "Muku",
  "version": "1.0.0",
  "identifier": "com.takumi.muku",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build"
  },
  "app": {
    "withGlobalTauri": true,
    "windows": [
      {
        "title": "Muku",
        "width": 400,
        "height": 600,
        "minWidth": 360,
        "minHeight": 500,
        "decorations": true,
        "resizable": true,
        "visible": true
      }
    ],
    "trayIcon": {
      "iconPath": "icons/tray-icon.png",
      "tooltip": "Muku - AIタスクマネージャー"
    }
  },
  "bundle": {
    "active": true,
    "targets": ["msi", "nsis"],
    "publisher": "Takumi Apps",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.ico"
    ],
    "externalBin": ["binaries/llama-server"],
    "resources": ["models/*"]
  }
}
```

### tauri.microsoftstore.conf.json（Microsoft Store用オーバーライド）

```json
{
  "bundle": {
    "windows": {
      "webviewInstallMode": {
        "type": "offlineInstaller"
      }
    }
  }
}
```

### Microsoft Storeビルドコマンド

```bash
# 通常ビルド後、MS Store用設定でバンドル
npm run tauri build -- --no-bundle
npm run tauri bundle -- --config src-tauri/tauri.microsoftstore.conf.json
```

---

## マネタイズ

### 価格モデル: 買い切り

- **価格**: 2,000〜3,000円（$15〜$20 USD）
- **サブスク不要**: ローカルLLMのためAPIコストゼロ、ランニングコストなし
- **セールスポイント**: 買い切り / サブスクなし / ネット不要 / 完全プライベート

### Microsoft Store手数料

- アプリ売上の15%（ゲーム以外）
- 実質手取り: 2,500円の場合 → 約2,125円/本

---

## セールスポイント（ストア掲載文案の素材）

1. **チャットで完結** — タスクの追加・完了・変更はすべて自然言語で
2. **買い切り、サブスクなし** — 一度買えばずっと使える
3. **完全オフライン** — ネット不要、AIはPC上で動作
4. **プライバシー最優先** — タスクデータは一切外部に送信されない
5. **常駐＆即起動** — Alt+Spaceで瞬時に呼び出し、作業を中断しない

---

## 開発フェーズ

### Phase 1: 基盤構築
- [ ] Tauri v2 + React + TypeScript プロジェクトセットアップ
- [ ] SQLiteスキーマ作成（tasks, messages）
- [ ] 基本的なChat UIとTask List UIの実装
- [ ] Chat / Tasks タブ切り替え

### Phase 2: LLM統合
- [ ] llama.cppバイナリのsidecar組み込み
- [ ] Qwen3.5 GGUFモデルの動作検証
- [ ] システムプロンプトの調整・テスト
- [ ] JSONレスポンスのパースとタスクDB反映

### Phase 3: デスクトップ体験
- [ ] システムトレイ常駐
- [ ] グローバルショートカット（Alt+Space）
- [ ] ウィンドウ状態の記憶（位置・サイズ）
- [ ] 多重起動防止
- [ ] PC起動時の自動起動（オプション）

### Phase 4: 通知・リマインド
- [ ] タスク期限ベースのWindows通知
- [ ] 定期的なリマインド（設定可能な間隔）

### Phase 5: リリース準備
- [ ] アプリアイコンの作成
- [ ] Microsoft Store開発者アカウント登録
- [ ] MS Store用ビルド設定（offlineInstaller）
- [ ] ストア掲載情報（説明文・スクリーンショット）
- [ ] コード署名
- [ ] Microsoft Storeへのアップロード・審査提出

---

## コスト最適化メモ

### LLM呼び出しの最適化
- 単純な完了操作（「〇〇終わった」）はキーワードマッチでルールベース処理し、LLMを呼ばない
- 会話履歴は直近10件のみをコンテキストに含める
- タスク一覧はアクティブなもののみをプロンプトに含める（完了済みは除外）

### モデルサイズとパフォーマンス
- Q4_K_M量子化で約7〜8GB（ディスク）
- RAM使用量: 推論時8〜10GB程度
- CPU推論: 初回応答まで数秒〜10秒程度（許容範囲）
- GPU (CUDA/Vulkan): 1〜3秒程度
- 最小動作環境: RAM 16GB以上推奨

---

## 参考リンク

- Tauri v2公式: https://v2.tauri.app/ja/
- Tauri Sidecar: https://v2.tauri.app/ja/develop/sidecar/
- Tauri System Tray: https://v2.tauri.app/ja/learn/system-tray/
- Tauri Global Shortcut: https://v2.tauri.app/ja/plugin/global-shortcut/
- Tauri Microsoft Store配布: https://v2.tauri.app/ja/distribute/microsoft-store/
- llama.cpp: https://github.com/ggml-org/llama.cpp
- Qwen: https://qwenlm.github.io/
