# todos.md — Grok 多角色語音配音 App 任務清單

---

## Phase 0 — 初始化

- [ ] 建立 repo：`grok-voice-studio`
- [ ] 建立 Rust workspace
- [ ] 建立 Tauri v2 + React + TypeScript GUI
- [ ] 設定 pnpm / npm scripts
- [ ] 設定 Rust lint：`cargo fmt`, `cargo clippy`
- [ ] 設定前端 lint：ESLint, Prettier
- [ ] 建立 `.env.example`
- [ ] 建立 `docs/`：plan/spec/todos/test/final
- [ ] 建立 CI：build frontend + cargo test

---

## Phase 1 — Domain Models

- [ ] 定義 `Project`
- [ ] 定義 `Character`
- [ ] 定義 `VoiceProfile`
- [ ] 定義 `ScriptSegment`
- [ ] 定義 `AudioAsset`
- [ ] 定義 `TimelineTrack`
- [ ] 定義 `TimelineClip`
- [ ] 定義 `ExportPreset`
- [ ] 定義 project JSON schema
- [ ] 實作 project load/save
- [ ] 實作 project auto-save

建議模型：

```rust
pub struct Project {
    pub id: String,
    pub title: String,
    pub characters: Vec<Character>,
    pub segments: Vec<ScriptSegment>,
    pub timeline: Timeline,
    pub export_presets: Vec<ExportPreset>,
}

pub struct Character {
    pub id: String,
    pub name: String,
    pub role_type: RoleType,
    pub voice_profile: VoiceProfile,
    pub color: String,
}

pub struct ScriptSegment {
    pub id: String,
    pub character_id: String,
    pub text: String,
    pub language: String,
    pub emotion_hint: Option<String>,
    pub speech_tags: Vec<String>,
    pub order: u32,
    pub audio_asset_id: Option<String>,
}
```

---

## Phase 2 — Script Parser

- [ ] 支援 `角色: 台詞`
- [ ] 支援 `角色（語氣）: 台詞`
- [ ] 支援 `[旁白] 台詞`
- [ ] 支援空行當段落間隔
- [ ] 支援註解：`# comment`
- [ ] 支援 stage direction：`（停頓兩秒）`
- [ ] 自動建立未知角色
- [ ] Parser 錯誤顯示行號
- [ ] 單元測試：中文冒號 `：` 與英文冒號 `:` 都可解析

範例輸入：

```txt
旁白：深夜的城市，只剩雨聲。
阿明（緊張）：你聽到了嗎？[pause] 那不是風聲。
小雅（笑）：別自己嚇自己。
怪物（低沉）：你們終於來了。
```

解析輸出：

```json
[
  { "character": "旁白", "text": "深夜的城市，只剩雨聲。" },
  { "character": "阿明", "emotion_hint": "緊張", "text": "你聽到了嗎？[pause] 那不是風聲。" },
  { "character": "小雅", "emotion_hint": "笑", "text": "別自己嚇自己。" },
  { "character": "怪物", "emotion_hint": "低沉", "text": "你們終於來了。" }
]
```

---

## Phase 3 — xAI Grok TTS Provider

- [ ] 建立 `TtsProvider` trait
- [ ] 實作 `XaiTtsProvider`
- [ ] 支援 API key 從環境變數 `XAI_API_KEY` 讀取
- [ ] GUI 支援 API key 設定
- [ ] API key 儲存在 OS keychain
- [ ] 實作 `GET /v1/tts/voices`
- [ ] 實作 `GET /v1/tts/voices/{voice_id}`
- [ ] 實作 `POST /v1/tts`
- [ ] 支援 `voice_id`
- [ ] 支援 `language` / `auto`
- [ ] 支援 `output_format.codec`: mp3 / wav / pcm
- [ ] 支援 `sample_rate`: 24000 / 44100 / 48000
- [ ] 支援 `bit_rate`
- [ ] 支援 inline speech tags
- [ ] 支援單句 retry
- [ ] 支援 rate limit backoff
- [ ] 支援 timeout
- [ ] 支援錯誤分類：auth / quota / network / validation / provider_error

REST 生成流程：

```http
POST https://api.x.ai/v1/tts
Authorization: Bearer $XAI_API_KEY
Content-Type: application/json

{
  "text": "你好，這是一段測試配音。",
  "voice_id": "eve",
  "language": "zh",
  "output_format": {
    "codec": "mp3",
    "sample_rate": 24000,
    "bit_rate": 128000
  }
}
```

---

## Phase 4 — Audio Cache

- [ ] 設計 cache key：`provider + voice_id + language + text + output_format`
- [ ] 產生 SHA-256 cache key
- [ ] 每句輸出到 `project/assets/audio/segments/{segment_id}.mp3`
- [ ] 建立 cache index SQLite table
- [ ] 支援 cache hit / miss 顯示
- [ ] 支援刪除單句 cache
- [ ] 支援重生單句音訊
- [ ] 支援全專案清除 cache
- [ ] 支援移動 project 後相對路徑仍有效

---

## Phase 5 — GUI MVP

### Layout

- [ ] 左側：Project / Character List
- [ ] 中間：Script Editor
- [ ] 右側：Voice Settings / Segment Inspector
- [ ] 下方：Generate Queue / Audio Player

### Pages

- [ ] Home：最近專案
- [ ] Project：編輯專案
- [ ] Characters：角色與聲音設定
- [ ] Script：劇本輸入與解析
- [ ] Generate：批次配音
- [ ] Timeline：音訊排列與混音
- [ ] Export：輸出設定
- [ ] Settings：API key、Provider、FFmpeg path

### GUI 功能

- [ ] 新增角色
- [ ] 刪除角色
- [ ] 修改角色 voice
- [ ] 從 xAI 同步 voices
- [ ] 編輯劇本
- [ ] 一鍵解析劇本
- [ ] 顯示 segments 表格
- [ ] 預覽單句
- [ ] 批次生成
- [ ] 暫停 / 取消生成
- [ ] 失敗重試
- [ ] 播放單句
- [ ] 播放全文
- [ ] 顯示進度百分比
- [ ] 顯示預估字數與成本欄位，成本需由使用者自行填入單價或串接官方 pricing

---

## Phase 6 — Story Mode

- [ ] 建立 Story Mode text area
- [ ] 設計 prompt：故事轉多角色配音劇本
- [ ] LLM 回傳 JSON schema
- [ ] JSON schema validation
- [ ] 若 JSON 壞掉，自動修復一次
- [ ] 產生角色清單
- [ ] 產生 segments
- [ ] 使用者可接受 / 修改 / 重新生成
- [ ] 支援風格：熱血、恐怖、童話、新聞播報、漫畫解說、短影音旁白
- [ ] 支援長故事分章節

LLM 輸出 schema：

```json
{
  "title": "string",
  "characters": [
    {
      "name": "string",
      "role_type": "narrator|character|system",
      "voice_hint": "string",
      "personality": "string"
    }
  ],
  "segments": [
    {
      "character": "string",
      "text": "string",
      "emotion_hint": "string",
      "pause_after_ms": 500
    }
  ]
}
```

---

## Phase 7 — Timeline / Mixdown

- [ ] 建立 timeline data model
- [ ] 匯入每句音訊長度
- [ ] 依 segment order 自動排列
- [ ] 支援 pause_after_ms
- [ ] 支援手動拖曳 clip
- [ ] 支援 waveform overview
- [ ] 支援 BGM 匯入
- [ ] 支援 SFX 匯入
- [ ] 支援角色音量
- [ ] 支援 clip 音量
- [ ] 支援 fade in/out
- [ ] 支援 normalize loudness
- [ ] 支援輸出單一 WAV
- [ ] 支援輸出 MP3
- [ ] 支援輸出 stem：每個角色各一軌

FFmpeg mixdown 範例：

```bash
ffmpeg -i seg1.wav -i seg2.wav -filter_complex "[0:a][1:a]concat=n=2:v=0:a=1[out]" -map "[out]" output.wav
```

---

## Phase 8 — Subtitles

- [ ] 根據 segment 音訊長度產生 SRT
- [ ] 支援 VTT
- [ ] 支援 ASS 字幕
- [ ] 支援角色名稱顯示或隱藏
- [ ] 支援字幕斷句
- [ ] 支援匯出「每句台詞 JSON」給剪輯工具

SRT 範例：

```srt
1
00:00:00,000 --> 00:00:03,200
旁白：深夜的城市，只剩雨聲。

2
00:00:03,700 --> 00:00:06,800
阿明：你聽到了嗎？那不是風聲。
```

---

## Phase 9 — Provider Plugin

- [ ] Provider config UI
- [ ] xAI Provider
- [ ] OpenAI-compatible Provider
- [ ] ElevenLabs Provider
- [ ] Local TTS Provider
- [ ] Provider health check
- [ ] Provider fallback chain
- [ ] 每個角色可指定 provider
- [ ] 每句可覆蓋 provider

---

## Phase 10 — Production

- [ ] Windows installer
- [ ] 自動更新
- [ ] Crash log
- [ ] Log viewer
- [ ] 匯出 debug bundle
- [ ] 專案備份
- [ ] 匯出 portable project zip
- [ ] 大型劇本壓力測試
- [ ] 隱私與授權聲明
- [ ] Custom voice 使用者同意流程

