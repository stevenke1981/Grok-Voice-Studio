# test.md — Grok 多角色語音配音 App 測試計畫

---

## 1. 測試目標

測試此 App 是否能穩定完成：

1. 多角色劇本解析
2. Grok / xAI TTS 語音生成
3. GUI 操作流程
4. 音訊快取
5. 批次任務與錯誤重試
6. 混音輸出
7. 字幕輸出
8. 專案儲存與重新開啟

---

## 2. 測試分類

| 類型 | 工具 |
|---|---|
| Rust unit test | `cargo test` |
| Parser snapshot test | insta / expect-test |
| Provider mock test | wiremock / mockito |
| Frontend unit test | Vitest |
| GUI E2E | Playwright / Tauri driver |
| Audio integration | FFmpeg + probe |
| Manual QA | Windows GUI checklist |

---

## 3. Parser 測試

### 3.1 中文冒號

輸入：

```txt
旁白：故事開始。
小明：你好。
```

預期：

- 2 segments
- characters：旁白、小明
- segment order 正確

---

### 3.2 英文冒號

輸入：

```txt
Narrator: The story begins.
John: Hello.
```

預期：

- 2 segments
- 支援英文冒號

---

### 3.3 語氣標註

輸入：

```txt
小美（害怕）：你聽到了嗎？
```

預期：

```json
{
  "character": "小美",
  "emotion_hint": "害怕",
  "text": "你聽到了嗎？"
}
```

---

### 3.4 Speech Tags

輸入：

```txt
阿明：等一下。[pause] 我想到了。
```

預期：

- text 保留 `[pause]`
- speech_tags 包含 `pause`

---

### 3.5 註解與空行

輸入：

```txt
# 第一幕
旁白：夜晚。

小明：我們走吧。
```

預期：

- 註解不產生 segment
- 空行可轉為 `pause_after_ms` 或 paragraph separator

---

## 4. xAI TTS Provider 測試

### 4.1 Voice List

測試：

- 呼叫 `GET /v1/tts/voices`
- 回傳 voices array
- GUI 下拉選單更新

Mock response：

```json
{
  "voices": [
    { "voice_id": "eve", "name": "Eve", "language": "multilingual" },
    { "voice_id": "ara", "name": "Ara", "language": "multilingual" }
  ]
}
```

預期：

- provider 回傳 `Vec<VoiceInfo>`
- 無 API key 時回傳 `MissingApiKey`

---

### 4.2 REST TTS 成功

Request：

```json
{
  "text": "你好，這是測試。",
  "voice_id": "ara",
  "language": "zh"
}
```

Mock：回傳 audio bytes。

預期：

- 儲存 `segment.mp3`
- 回傳 file path
- cache index 新增資料

---

### 4.3 REST TTS 失敗

情境：

- 401 Unauthorized
- 429 Rate Limited
- 500 Provider Error
- Network Timeout

預期：

- 錯誤分類正確
- GUI 顯示可理解訊息
- 429 會進入 backoff retry
- 單句可以手動重試

---

### 4.4 Text Too Long

輸入：超過 15,000 characters。

預期：

- 不直接送出
- 顯示 `TextTooLong`
- 提供「自動拆句」功能

---

## 5. Audio Cache 測試

### 5.1 Cache Hit

步驟：

1. 生成同一段文字
2. 第二次再生成

預期：

- 第二次不呼叫 provider
- GUI 顯示 cached
- 音訊檔案 path 相同或可重用

---

### 5.2 Cache Key 改變

變更：

- voice_id
- language
- output format
- text

預期：

- cache key 改變
- 重新生成

---

## 6. GUI 測試

### 6.1 新建專案

步驟：

1. 開啟 App
2. 點 New Project
3. 輸入 title
4. 儲存

預期：

- 建立 project folder
- 產生 project.json
- Home 最近專案出現該專案

---

### 6.2 角色管理

步驟：

1. 新增角色「旁白」
2. 選 voice `rex`
3. 新增角色「少女」
4. 選 voice `ara`

預期：

- 角色列表更新
- project.json 儲存正確

---

### 6.3 劇本解析

輸入：

```txt
旁白：夜晚來臨。
少女：我不怕。
```

按 Parse。

預期：

- segments table 有 2 筆
- 自動匹配角色

---

### 6.4 批次生成

步驟：

1. 選 10 句 segments
2. 按 Generate All
3. 等待完成

預期：

- queue 顯示進度
- 每句狀態變 done
- 失敗項目可 retry

---

## 7. Mixdown 測試

### 7.1 合併輸出 WAV

輸入：3 個 segment audio。

預期：

- 產生 `final.wav`
- `ffprobe` 可讀
- duration 約等於 segments duration + pauses

---

### 7.2 合併輸出 MP3

預期：

- 產生 `final.mp3`
- bitrate 符合 preset
- 可播放

---

### 7.3 BGM 混音

步驟：

1. 匯入 BGM
2. 設定 BGM volume -18 dB
3. Export

預期：

- final 中包含 BGM
- 人聲不被蓋過

---

## 8. Subtitle 測試

### 8.1 SRT 輸出

預期：

- SRT index 從 1 開始
- timestamp 遞增
- 每句文字正確
- 可被剪輯軟體載入

---

### 8.2 角色名稱顯示

設定：顯示角色名稱。

預期：

```srt
1
00:00:00,000 --> 00:00:02,000
旁白：夜晚來臨。
```

設定：隱藏角色名稱。

預期：

```srt
1
00:00:00,000 --> 00:00:02,000
夜晚來臨。
```

---

## 9. 壓力測試

### 9.1 長劇本

輸入：

- 1000 segments
- 10 個角色

預期：

- GUI 不凍結
- queue 可暫停 / 恢復
- 中途關閉重開可恢復

---

### 9.2 大量 Cache

輸入：

- 5000 audio cache items

預期：

- 專案開啟時間可接受
- Cache 清理功能正常

---

## 10. Manual QA Checklist

- [ ] Windows 10 可啟動
- [ ] Windows 11 可啟動
- [ ] 沒有 API key 時錯誤清楚
- [ ] API key 不會出現在 log
- [ ] 可同步 voices
- [ ] 可建立角色
- [ ] 可輸入劇本
- [ ] 可解析劇本
- [ ] 可單句生成
- [ ] 可批次生成
- [ ] 可播放單句
- [ ] 可播放全文
- [ ] 可匯入 BGM
- [ ] 可輸出 WAV
- [ ] 可輸出 MP3
- [ ] 可輸出 SRT
- [ ] 可重新開啟專案
- [ ] 可清除 cache
- [ ] 可匯出 debug bundle

---

## 11. 驗收命令

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm lint
pnpm test
pnpm tauri build
```

音訊檢查：

```bash
ffprobe exports/final.wav
ffprobe exports/final.mp3
```

