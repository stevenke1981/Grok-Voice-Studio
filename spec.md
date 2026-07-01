# spec.md — Grok 多角色語音生成配音 App 規格書

---

## 1. 產品定位

**Grok Voice Studio** 是一款桌面 GUI 配音工具，讓使用者可以直接輸入對話、故事、小說、短影音旁白或漫畫解說稿，自動轉成多角色配音腳本，並使用 Grok / xAI TTS 或其他可插拔 TTS 引擎生成語音。

核心目標：

1. 不只是文字轉語音，而是「多角色配音工作台」。
2. 支援故事轉劇本、角色聲音管理、逐句預覽、批次生成。
3. 支援時間線、字幕、BGM、SFX、音量調整與最終輸出。
4. 架構可擴充到本地模型與其他雲端 TTS。

---

## 2. 使用者情境

### 2.1 短影音解說

使用者貼上一段故事：

```txt
一名少女在雷雨夜走進廢棄學校，她聽見走廊盡頭傳來熟悉的聲音……
```

App 會轉成：

```txt
旁白：雷雨夜，廢棄學校裡只剩閃電照亮走廊。
少女（緊張）：有人在那裡嗎？
神秘聲音（低沉）：你終於回來了。
```

再由使用者指定：

- 旁白：Rex / Sal
- 少女：Ara / Eve
- 神秘聲音：Leo

最後輸出：

- `final_mix.mp3`
- `subtitles.srt`
- `project.json`

---

## 3. 功能需求

### 3.1 Project Management

每個專案是一個資料夾：

```txt
my-project/
  project.json
  assets/
    audio/
      segments/
      mixdown/
    bgm/
    sfx/
  exports/
    final.mp3
    final.wav
    subtitles.srt
```

需求：

- 建立新專案
- 開啟既有專案
- 自動儲存
- 匯出 portable project bundle
- 專案版本欄位：未來 schema migration

---

### 3.2 Script Input

支援三種輸入模式：

#### A. Dialogue Mode

使用者直接輸入：

```txt
旁白：故事開始了。
小明：我們走吧。
小美（害怕）：等一下，我聽到聲音了。
```

Parser 規則：

- `角色: 台詞`
- `角色：台詞`
- `角色（語氣）: 台詞`
- `[角色] 台詞`
- `旁白:` 作為 narrator
- 空行可視為段落 pause

#### B. Story Mode

使用者貼上完整故事，App 呼叫 LLM 轉成多角色 JSON。

#### C. JSON Import Mode

支援匯入結構化腳本：

```json
{
  "characters": [],
  "segments": []
}
```

---

### 3.3 Character Management

角色欄位：

| 欄位 | 說明 |
|---|---|
| id | UUID |
| name | 角色名稱 |
| role_type | narrator / character / system |
| voice_provider | xai / openai / elevenlabs / local |
| voice_id | Provider voice ID |
| language | zh / en / ja / auto |
| style_prompt | 語氣描述，例如「溫柔、緊張、低沉」 |
| volume_db | 預設音量 |
| pan | 左右聲道位置 |
| color | GUI 顯示顏色 |

內建 xAI fallback voices：

| voice_id | 建議用途 |
|---|---|
| eve | 活潑、熱情、女性角色 |
| ara | 溫暖、親切、對話角色 |
| rex | 清晰、專業、商業旁白 |
| sal | 中性、平衡、通用旁白 |
| leo | 權威、低沉、命令感角色 |

實作上不應硬編死 voice 清單，應優先呼叫 `GET /v1/tts/voices` 同步最新 Voice Library。

---

### 3.4 TTS Generation

#### xAI REST TTS

Endpoint：

```txt
POST https://api.x.ai/v1/tts
```

Request：

```json
{
  "text": "你好，這是一段中文配音。",
  "voice_id": "ara",
  "language": "zh",
  "output_format": {
    "codec": "mp3",
    "sample_rate": 24000,
    "bit_rate": 128000
  }
}
```

Response：

- 直接回傳 audio bytes
- Content-Type 依格式可能是 audio/mpeg / audio/wav 等

需求：

- 每個 segment 單獨生成
- 每個 segment 有重試按鈕
- 支援批次 queue
- 支援 rate limit backoff
- 支援 timeout
- 支援取消任務
- 支援 cache

---

### 3.5 Streaming Preview

Endpoint：

```txt
wss://api.x.ai/v1/tts
```

用途：

- GUI 中按「快速預覽」時使用。
- 邊產生邊播放，降低等待感。
- 批次正式輸出仍建議 REST，方便 cache 與檔案管理。

需求：

- WebSocket client
- 傳送 text.delta
- 接收 audio.delta
- 播放 buffer
- 最後存成 preview cache 或丟棄

---

### 3.6 Speech Tags

支援 xAI TTS inline speech tags：

```txt
阿明：你聽到了嗎？[pause] 那不是風聲。[sigh]
```

GUI 應提供快捷插入：

- `[pause]`
- `[long-pause]`
- `[laugh]`
- `[chuckle]`
- `[sigh]`
- `[breath]`
- `[inhale]`
- `[exhale]`

限制：

- 不同 Provider 不一定支援同樣 tags。
- Provider 不支援時，應由 compatibility layer 轉換或提示。

---

## 4. GUI 規格

### 4.1 主視窗布局

```txt
┌────────────────────────────────────────────────────────────┐
│ Top Bar: Project | Save | Provider | Sync Voices | Export  │
├───────────────┬─────────────────────────┬──────────────────┤
│ Characters    │ Script Editor            │ Inspector        │
│ - 旁白        │                         │ Segment Settings │
│ - 小明        │ 旁白：...                │ Voice            │
│ - 小美        │ 小明：...                │ Emotion          │
│               │                         │ Speech Tags      │
├───────────────┴─────────────────────────┴──────────────────┤
│ Generate Queue / Player / Timeline Preview                  │
└────────────────────────────────────────────────────────────┘
```

---

### 4.2 必要頁面

#### Home

- 最近專案
- 新建專案
- 開啟專案
- 範例專案

#### Script

- Monaco Editor 或 CodeMirror
- 語法高亮：角色、語氣、speech tag
- Parse 按鈕
- 顯示 parse errors
- Story Mode 轉換按鈕

#### Characters

- 角色列表
- voice provider
- voice_id 下拉選單
- 語言選擇
- 試聽按鈕
- 預設情緒設定

#### Generate

- segments table
- 每句狀態：pending / cached / generating / done / failed
- 單句播放
- 單句重生
- 批次生成
- 暫停 / 取消

#### Timeline

- 角色軌
- 旁白軌
- BGM 軌
- SFX 軌
- waveform
- clip 拖曳
- clip 間距
- fade in / fade out

#### Export

- 格式：WAV / MP3 / FLAC
- sample rate
- bitrate
- loudness normalization
- SRT / VTT / ASS
- 匯出 stems

#### Settings

- xAI API Key
- Provider 設定
- FFmpeg 路徑
- cache 位置
- 自動儲存
- log level

---

## 5. Backend API 設計

Tauri commands：

```rust
#[tauri::command]
async fn parse_script(input: String) -> Result<ParsedScript, AppError>;

#[tauri::command]
async fn sync_voices(provider: String) -> Result<Vec<VoiceInfo>, AppError>;

#[tauri::command]
async fn generate_segment(project_id: String, segment_id: String) -> Result<AudioAsset, AppError>;

#[tauri::command]
async fn generate_all(project_id: String) -> Result<JobId, AppError>;

#[tauri::command]
async fn cancel_job(job_id: String) -> Result<(), AppError>;

#[tauri::command]
async fn export_mixdown(project_id: String, preset: ExportPreset) -> Result<ExportResult, AppError>;
```

---

## 6. Database Schema

SQLite tables：

```sql
CREATE TABLE projects (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  path TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE audio_cache (
  cache_key TEXT PRIMARY KEY,
  provider TEXT NOT NULL,
  voice_id TEXT NOT NULL,
  language TEXT NOT NULL,
  text_hash TEXT NOT NULL,
  file_path TEXT NOT NULL,
  duration_ms INTEGER,
  created_at TEXT NOT NULL
);

CREATE TABLE jobs (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  status TEXT NOT NULL,
  progress_current INTEGER NOT NULL,
  progress_total INTEGER NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
```

---

## 7. Audio Pipeline

### 7.1 Segment Generation

```txt
segment text
  -> normalize
  -> provider request
  -> audio bytes
  -> save to cache
  -> decode duration
  -> update project
```

### 7.2 Mixdown

```txt
segments + timeline
  -> generate concat list
  -> insert silence / fades
  -> mix BGM / SFX
  -> loudness normalize
  -> export final.wav / final.mp3
```

### 7.3 Subtitle

```txt
segment start time + duration
  -> SRT / VTT entries
```

---

## 8. 錯誤處理

錯誤類型：

| 類型 | GUI 顯示 | 解法 |
|---|---|---|
| MissingApiKey | 尚未設定 API Key | 到 Settings 設定 |
| AuthFailed | API Key 無效 | 重新輸入 |
| RateLimited | API 限速 | 等待或降低並行 |
| QuotaExceeded | 額度不足 | 更換 key 或稍後再試 |
| TextTooLong | 單句過長 | 自動拆句 |
| ProviderUnavailable | 供應商暫時不可用 | 重試或切 Provider |
| FfmpegMissing | 找不到 FFmpeg | 設定 FFmpeg 路徑 |
| ExportFailed | 輸出失敗 | 查看 log |

---

## 9. 安全與合規

- API key 不可寫入明文 project.json。
- API key 儲存在 OS keychain。
- Custom voice 需要使用者確認授權。
- 禁止引導使用者複製未授權真人聲音。
- 專案匯出不包含 API key。
- Log 不記錄完整 API key。
- 可選擇不記錄原始故事文本。

---

## 10. MVP 驗收規格

MVP 完成時至少要做到：

- Windows GUI 可執行。
- 可輸入多角色對話。
- 可解析角色與台詞。
- 可同步 xAI voice list。
- 可設定每個角色 voice。
- 可單句生成與播放。
- 可批次生成。
- 可合併輸出 MP3 / WAV。
- 可產生 SRT。
- 可儲存與重新開啟專案。

