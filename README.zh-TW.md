# Grok Voice Studio

**多角色語音配音桌面應用**，使用 [xAI Grok TTS](https://docs.x.ai/) 為劇本角色批次生成語音，支援音效混音與字幕匯出。

| | |
|---|---|
| **English** | [README.md](./README.md) |
| **給 AI 代理** | [AGENTS.md](./AGENTS.md) |

## 功能特色

- **專案管理** — 建立／開啟／儲存本機配音專案
- **劇本編輯** — 對話格式（`角色：台詞`）、情緒標記、語音標籤（`[pause]`、`[sigh]`…）
- **故事模式** — 用 Grok Chat 將散文轉成多角色劇本
- **引導流程** — 輸入劇本 → **解析劇本** → 指定語音 → 生成 → 匯出
- **xAI 語音** — 內建 voice + [自訂語音](https://docs.x.ai/developers/model-capabilities/audio/custom-voices)；可選 [WebSocket 串流 TTS](https://docs.x.ai/developers/model-capabilities/audio/text-to-speech#streaming-tts-websocket)，失敗時自動改 REST
- **音效庫** — 15 種內建音效、可匯入自訂檔、浮動可拖曳視窗
- **批次生成** — 背景佇列、暫停／取消／重試、音訊快取
- **匯出** — MP3／WAV／FLAC + SRT／VTT／ASS（需 FFmpeg）
- **介面語言** — 繁體中文／English

## 環境需求

| 工具 | 說明 |
|------|------|
| [Rust](https://rustup.rs/) | stable |
| [Node.js](https://nodejs.org/) | 20+ |
| [pnpm](https://pnpm.io/) | 9+ |
| [FFmpeg](https://ffmpeg.org/) | 需在 PATH，或於設定指定路徑 |
| xAI API Key | [console.x.ai](https://console.x.ai/) |

API Key 儲存於 Windows 系統金鑰庫（非明文 settings）。

## 快速開始

```bash
git clone https://github.com/stevenke1981/Grok-Voice-Studio.git
cd Grok-Voice-Studio
pnpm install
cp .env.example .env   # 選用；也可在 app 內設定

pnpm dev      # 開發模式
pnpm test     # Rust 測試
pnpm build    # 建置 Windows 安裝程式
```

### 建議操作流程

1. **設定** → 輸入 xAI API Key → **驗證 API Key**
2. **新建專案** → 選擇資料夾
3. 輸入或載入**模板劇本** → 點擊 **解析劇本**（必要步驟）
4. 為各角色指定 **Voice**（內建或自訂語音）
5. **全部生成** → 完成後 **匯出**
6. 需要音效時，點工具列 **音效庫**（可拖曳的獨立視窗）

## 劇本語法

```text
旁白：深夜的城市，只剩雨聲。
音效：雨聲
阿明（緊張）：你聽到了嗎？{雷聲} [pause] 那不是風聲。
小雅：別自己嚇自己。
```

| 語法 | 說明 |
|------|------|
| `角色：台詞` | 對話句 |
| `角色（情緒）：台詞` | 帶情緒提示 |
| `音效：名稱` | 獨立音效段落 |
| `{音效名}` | 行內音效（接在該句 TTS 之後） |
| `[pause]` 等 | 語音標籤 |

模板位於 `examples/templates/` 與 app 內「模板劇本」選單。

## 自訂語音

1. 至 [xAI Voice Library](https://console.x.ai/team/default/voice/voice-library) 建立語音
2. 複製 **Voice ID**
3. 本 app：**同步 Voices** → 在角色面板的「自訂語音」分組選用

設定頁可管理自訂語音列表；Enterprise 方案可透過 API 上傳參考音訊克隆。

## 音效庫

- 工具列或劇本編輯器開啟 **音效庫**（浮動視窗，可拖曳、調整大小）
- **插入行**：`音效：雨聲`
- **行內插入**：`{雷聲}`
- 可匯入 WAV／MP3 等自訂音效

## 專案結構

```
crates/core/           資料模型、解析器、時間軸、音效目錄
crates/providers/xai/  TTS、Chat、自訂語音 API
crates/audio/          FFmpeg 混音、字幕
crates/storage/        快取、設定、金鑰庫、音效儲存
apps/desktop/          Tauri v2 + React 前端
examples/templates/    範例劇本
```

## 設定與資料路徑

| 項目 | 位置 |
|------|------|
| API Key | app 設定或環境變數 `XAI_API_KEY` |
| 應用資料 | `%LOCALAPPDATA%\GrokVoiceStudio\` |
| 專案檔 | 使用者指定資料夾內 `project.json`、`audio/`、`exports/` |

## 開發指令

```bash
cargo test --workspace
pnpm exec tsc --noEmit --prefix apps/desktop
pnpm --dir apps/desktop exec vite build
```

修改使用者可見行為時，請同步更新 `README.md` 與本檔；架構／代理規則變更請更新 `AGENTS.md`。

## 已知限制（v0.1）

- 批次生成目前為循序執行
- API 建立自訂語音需 Enterprise；一般用戶請用 Console
- 自訂語音地區限制請參考 xAI 官方文件

## 連結

- [GitHub](https://github.com/stevenke1981/Grok-Voice-Studio)
- [xAI 語音文件](https://docs.x.ai/developers/model-capabilities/audio)
- [xAI 自訂語音](https://docs.x.ai/developers/model-capabilities/audio/custom-voices)