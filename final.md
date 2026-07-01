# final.md — Grok 多角色語音生成配音 App 最終建議

---

## 最終建議方案

建議把這個 App 做成：

> **Tauri v2 + React/TypeScript GUI + Rust Backend + SQLite + FFmpeg + xAI Grok TTS Provider**

這樣可以同時滿足：

- 有 GUI
- Windows 桌面可打包
- Rust 後端穩定
- 可管理多角色
- 可輸入故事或對話
- 可批次生成語音
- 可輸出 MP3 / WAV / SRT
- 未來可接本地模型與其他 TTS Provider

---

## 核心功能清單

第一版一定要有：

1. GUI 專案管理
2. 多角色設定
3. 劇本編輯器
4. `角色：台詞` 自動解析
5. Story Mode：故事轉多角色劇本
6. Grok / xAI TTS 語音生成
7. 單句預覽
8. 批次生成
9. 音訊快取
10. 合併輸出 MP3 / WAV
11. 產生 SRT 字幕
12. API key 安全保存

第二版再加入：

1. 多軌 timeline
2. waveform
3. BGM / SFX
4. 淡入淡出
5. 多 Provider plugin
6. Custom voice 管理
7. 長劇本任務恢復

---

## 推薦 MVP 畫面

```txt
┌─────────────────────────────────────────────────────────────┐
│ Grok Voice Studio                         Sync Voices Export │
├───────────────┬────────────────────────────┬────────────────┤
│ Characters    │ Script Editor               │ Inspector      │
│ 旁白  rex     │ 旁白：夜晚來臨。            │ Voice: rex     │
│ 少女  ara     │ 少女：我不怕。              │ Emotion: calm  │
│ 魔王  leo     │ 魔王：你終於來了。          │ Tags: [pause]  │
├───────────────┴────────────────────────────┴────────────────┤
│ Generate Queue: 12 / 30 done                                  │
│ [Preview] [Generate Selected] [Generate All] [Export MP3]      │
└─────────────────────────────────────────────────────────────┘
```

---

## 最小可行版本開發順序

### Step 1 — 建立專案

```bash
pnpm create tauri-app grok-voice-studio
cd grok-voice-studio
pnpm install
pnpm tauri dev
```

### Step 2 — Rust Workspace

```txt
crates/
  core/
  providers/xai/
  audio/
  storage/
```

### Step 3 — 先做 Parser

先完成：

```txt
旁白：深夜的城市，只剩雨聲。
阿明（緊張）：你聽到了嗎？[pause] 那不是風聲。
小雅（笑）：別自己嚇自己。
```

轉成：

```json
[
  { "character": "旁白", "text": "深夜的城市，只剩雨聲。" },
  { "character": "阿明", "emotion": "緊張", "text": "你聽到了嗎？[pause] 那不是風聲。" },
  { "character": "小雅", "emotion": "笑", "text": "別自己嚇自己。" }
]
```

### Step 4 — 接 xAI TTS

核心 API：

```http
POST https://api.x.ai/v1/tts
Authorization: Bearer $XAI_API_KEY
Content-Type: application/json
```

Body：

```json
{
  "text": "你好，這是一段配音測試。",
  "voice_id": "ara",
  "language": "zh",
  "output_format": {
    "codec": "mp3",
    "sample_rate": 24000,
    "bit_rate": 128000
  }
}
```

### Step 5 — 做 GUI 最小版

只要先做這四塊：

```txt
角色列表 | 劇本輸入 | 生成佇列 | 播放/輸出
```

先不要急著做完整 timeline，第一版只要能依順序合併音訊即可。

---

## 最推薦的 App 架構

```txt
Frontend React
  ├─ ProjectPage
  ├─ CharacterPanel
  ├─ ScriptEditor
  ├─ SegmentTable
  ├─ GenerateQueue
  └─ ExportPanel

Tauri Commands
  ├─ parse_script
  ├─ sync_voices
  ├─ generate_segment
  ├─ generate_all
  ├─ export_mixdown
  └─ save_project

Rust Backend
  ├─ core models
  ├─ xAI provider
  ├─ cache manager
  ├─ ffmpeg audio mixer
  ├─ subtitle exporter
  └─ sqlite storage
```

---

## 重要設計原則

### 1. 不要把整篇故事一次丟 TTS

要拆成 segment：

- 方便多角色
- 方便重試
- 方便 cache
- 方便字幕
- 方便時間線

---

### 2. 不要把 voice 寫死

雖然可以內建 `eve`, `ara`, `rex`, `sal`, `leo`，但正式 App 應呼叫：

```txt
GET https://api.x.ai/v1/tts/voices
```

取得最新 voice 清單。

---

### 3. 每句都要有 cache

cache key：

```txt
provider + voice_id + language + text + output_format
```

這樣修改其中一句時，不用整部作品重生。

---

### 4. Story Mode 要可編輯

Grok LLM 可以幫你把故事轉劇本，但使用者一定要能手動修改，不能直接生成不可控結果。

---

### 5. Custom Voice 要加授權提示

如果加入自訂聲音功能，必須明確提醒：

- 只能使用自己或已取得授權的聲音
- 不可複製他人聲音
- 專案需記錄 custom voice 的授權來源備註

---

## MVP 完成判定

當以下流程能完整跑通，就算 MVP 完成：

1. 開啟 GUI
2. 建立專案
3. 輸入 3 個角色、30 句台詞
4. 每個角色指定不同 Grok voice
5. 批次生成語音
6. 播放預覽
7. 匯出 `final.mp3`
8. 匯出 `subtitles.srt`
9. 關閉 App
10. 重新開啟專案仍能看到所有資料與音訊

---

## 建議下一步

先做 **MVP v0.1**，不要一開始就做太完整的影片剪輯 timeline。

v0.1 目標：

```txt
劇本輸入 -> 角色解析 -> Grok TTS 生成 -> 音訊合併 -> MP3/SRT 輸出
```

完成 v0.1 後，再把 timeline、BGM、SFX、多軌與 custom voice 加進去。

