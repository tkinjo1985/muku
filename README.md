# Muku（ムク）

ローカル LLM 搭載の Windows 向け常駐型 AI タスク管理アプリ。

チャット（自然言語）でタスクを操作し、AI がタスクの登録・更新・優先度判定・進捗管理を自律的に行います。タスク一覧は読み取り専用のリストビューで確認できる「AIが管理する → UIで確認する」逆転発想のタスクマネージャーです。

## 特徴

- **チャットで完結** — タスクの追加・完了・変更はすべて自然言語で
- **完全オフライン** — ネット不要、AI は PC 上で動作（初回のみモデルダウンロード）
- **プライバシー最優先** — タスクデータは一切外部に送信されない
- **常駐 & 即起動** — システムトレイ常駐、グローバルショートカットで瞬時に呼び出し
- **期限通知** — タスクの期限前・超過時に Windows 通知 + チャット内リマインド
- **モデル選択** — 最軽量（Qwen3.5 2B / 1.3GB）・速度優先（Qwen3.5 4B / 2.7GB）・精度優先（Qwen3.5 9B / 5.7GB）を設定から切替

## 技術スタック

| レイヤー | 技術 |
|---------|------|
| デスクトップフレームワーク | Tauri v2 (Rust + WebView2) |
| フロントエンド | React + TypeScript + Vite |
| ローカル LLM 推論 | llama.cpp (Vulkan / CPU) |
| LLM モデル | Qwen3.5 2B / 4B / 9B (GGUF Q4_K_M) |
| データ永続化 | SQLite |
| 通知 | Windows Toast (tauri-winrt-notification) |

## システム要件

| 項目 | 最小 | 推奨 |
|------|------|------|
| OS | Windows 10 (64-bit) | Windows 11 |
| RAM | 8 GB（4B モデル） | 16 GB（9B モデル） |
| ディスク | 4 GB（アプリ + 4B） | 8 GB（アプリ + 9B） |
| GPU | Vulkan 対応 GPU | Vulkan 対応 GPU（VRAM 4 GB+） |
| ランタイム | [Microsoft Visual C++ 再頒布可能パッケージ](https://aka.ms/vs/17/release/vc_redist.x64.exe) | — |

> **Note**: VC++ ランタイムは多くの Windows PC にインストール済みですが、未インストールの場合は llama-server 起動時に `MSVCP140.dll が見つかりません` エラーが出ます。上記リンクからインストールしてください。
>
> **GPU について**: CPU のみでも動作はしますが、2B モデルでも応答が空になる・JSON パースに失敗するなど実用には耐えません。Vulkan 対応 GPU（内蔵 GPU でも可）の利用を強く推奨します。

## ビルド方法

### 前提条件

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://www.rust-lang.org/learn/get-started) (stable)
- [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (MSVC + Windows SDK)

### llama.cpp バイナリの準備

[llama.cpp Releases](https://github.com/ggml-org/llama.cpp/releases) から Windows Vulkan 版をダウンロードし、`src-tauri/binaries/` に配置:

```bash
# 例: b8797 の場合
curl -L -o llama-vulkan.zip https://github.com/ggml-org/llama.cpp/releases/download/b8797/llama-b8797-bin-win-vulkan-x64.zip
unzip llama-vulkan.zip -d src-tauri/binaries/
# llama-server.exe を Tauri sidecar 命名規則に合わせる（任意）
mv src-tauri/binaries/llama-server.exe src-tauri/binaries/llama-server-x86_64-pc-windows-msvc.exe
```

### モデルの準備（開発用）

```bash
# 2B（1.3 GB、最軽量）
curl -L -o src-tauri/models/Qwen3.5-2B-Q4_K_M.gguf \
  https://huggingface.co/unsloth/Qwen3.5-2B-GGUF/resolve/main/Qwen3.5-2B-Q4_K_M.gguf

# 4B（2.7 GB、デフォルト）
curl -L -o src-tauri/models/Qwen3.5-4B-Q4_K_M.gguf \
  https://huggingface.co/unsloth/Qwen3.5-4B-GGUF/resolve/main/Qwen3.5-4B-Q4_K_M.gguf

# 9B（5.7 GB、精度優先）
curl -L -o src-tauri/models/Qwen3.5-9B-Q4_K_M.gguf \
  https://huggingface.co/unsloth/Qwen3.5-9B-GGUF/resolve/main/Qwen3.5-9B-Q4_K_M.gguf
```

> **Note**: リリースビルドではモデルは初回起動時に自動ダウンロードされます。`src-tauri/models/` は開発時のみ必要です。

### ビルド & 起動

```bash
npm install
npm run tauri dev        # 開発モード（ホットリロード対応）
npx tauri build --no-bundle  # リリースバイナリ生成
```

## プロジェクト構成

```
muku/
├── src/                      # Frontend (React + TypeScript)
│   ├── App.tsx               # タブ切替（Chat / Tasks / Settings）
│   ├── components/           # ChatView, TaskListView, SettingsView, ...
│   ├── hooks/                # useChat, useTasks, useLlmStatus
│   ├── lib/                  # DB ヘルパー, invoke ラッパー, settings
│   ├── types/                # 型定義
│   └── styles/               # ダークテーマ CSS
│
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs            # エントリ（トレイ, ショートカット, sidecar）
│   │   ├── llm/              # LLM HTTP クライアント + プロンプト
│   │   ├── llm_init.rs       # モデル DL + sidecar 起動ステートマシン
│   │   ├── notifier.rs       # 期限通知 + 定期リマインド
│   │   ├── job_guard.rs      # Windows JobObject（sidecar ライフサイクル）
│   │   └── commands/         # Tauri コマンド
│   ├── binaries/             # llama-server + DLL（.gitignore）
│   └── models/               # GGUF モデル（.gitignore）
│
├── docs/
│   ├── muku-dev-doc.md       # 詳細設計ドキュメント
│   └── qa-checklist.md       # QA チェックリスト
│
└── design/                   # アイコン原画
```

## ライセンス

[GNU General Public License v3.0](LICENSE)

Copyright (c) 2026 tkdeveloper
