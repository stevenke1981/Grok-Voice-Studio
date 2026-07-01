# Grok Voice Studio

**Multi-character voice dubbing desktop app** powered by [xAI Grok TTS](https://docs.x.ai/).  
Build dialogue projects, assign voices per character, batch-generate audio, mix SFX, and export MP3/WAV/FLAC with subtitles.

| | |
|---|---|
| **中文說明** | [README.zh-TW.md](./README.zh-TW.md) |
| **Agent guide** | [AGENTS.md](./AGENTS.md) |
| **License** | MIT (see repository) |

## Features

- **Project workflow** — create/open/save dubbing projects on disk
- **Script editor** — dialogue format (`角色：台詞`), emotion hints, speech tags (`[pause]`, `[sigh]`, …)
- **Story mode** — convert prose to multi-character script via Grok Chat
- **Parse → assign voices → generate → export** — guided stepper in the UI
- **xAI TTS** — built-in voices + [custom voices](https://docs.x.ai/developers/model-capabilities/audio/custom-voices) (`GET /v1/custom-voices`); optional [streaming WebSocket](https://docs.x.ai/developers/model-capabilities/audio/text-to-speech#streaming-tts-websocket) with REST fallback
- **SFX library** — 15 built-in sounds, import custom clips, compose in script (`音效：雨聲`, inline `{雷聲}`)
- **Batch generation** — background jobs, pause/cancel/retry, SQLite audio cache
- **Export** — mixdown + SRT/VTT/ASS, optional per-character stems (FFmpeg)
- **i18n** — UI in English and Traditional Chinese

## Requirements

| Tool | Version |
|------|---------|
| [Rust](https://rustup.rs/) | stable (2021 edition) |
| [Node.js](https://nodejs.org/) | 20+ |
| [pnpm](https://pnpm.io/) | 9+ |
| [FFmpeg](https://ffmpeg.org/) | on `PATH` (or set in settings) |
| xAI API key | [console.x.ai](https://console.x.ai/) |

Windows keyring is used to store the API key securely.

## Quick start

```bash
git clone https://github.com/stevenke1981/Grok-Voice-Studio.git
cd Grok-Voice-Studio
pnpm install          # installs apps/desktop deps
cp .env.example .env  # optional; key can be set in-app

# Dev (Tauri + Vite hot reload)
pnpm dev

# Tests
pnpm test

# Production installer (Windows NSIS)
pnpm build
```

### First-run workflow

1. **Settings** → enter xAI API Key → **Verify API Key**
2. **New project** → pick a folder
3. Write or load a **template script** → **Parse script**
4. Assign **voice** per character (built-in or custom)
5. **Generate all** → **Export** when done
6. Optional: open floating **SFX library** from the toolbar to insert sounds

## Script syntax

```text
旁白：深夜的城市，只剩雨聲。
音效：雨聲
阿明（緊張）：你聽到了嗎？{雷聲} [pause] 那不是風聲。
小雅：別自己嚇自己。
```

| Syntax | Meaning |
|--------|---------|
| `角色：台詞` | Dialogue line |
| `角色（情緒）：台詞` | Dialogue with emotion hint |
| `音效：名稱` | Standalone SFX segment |
| `{音效名}` | Inline SFX after the line’s TTS |
| `[pause]` etc. | Speech tags (kept in TTS text) |

Preset templates live in `examples/templates/` and `apps/desktop/src/templates/`.

## Architecture

```
crates/core/           Project model, script parser, timeline, SFX catalog
crates/providers/xai/  Grok TTS + Chat + custom voices API
crates/audio/          FFmpeg mixdown, subtitle writers
crates/storage/        SQLite cache, settings, keyring, SFX store
apps/desktop/          Tauri v2 + React UI
  src-tauri/           Rust commands & services
  src/                 React components, hooks, i18n
examples/templates/    Sample dialogue & story scripts
```

### Key Tauri commands

`parse_script_command`, `start_generate_job`, `export_mixdown`, `sync_voices`, `list_sfx_library`, `list_custom_voices`, `get_logs`

Full command list: `apps/desktop/src-tauri/src/lib.rs`

## Configuration

| Setting | Location |
|---------|----------|
| API key | In-app Settings (Windows Credential Manager) or `XAI_API_KEY` env |
| FFmpeg | Settings or `FFMPEG_PATH` env |
| App data | `%LOCALAPPDATA%\GrokVoiceStudio\` (cache, SFX, settings) |
| Project files | User-chosen folder (`project.json`, `audio/`, `exports/`) |

## Development

```bash
cargo test --workspace
pnpm exec tsc --noEmit --prefix apps/desktop
pnpm --dir apps/desktop exec vite build
cargo fmt --all
```

When changing behavior, update **both** `README.md` and `README.zh-TW.md` if user-facing, and `AGENTS.md` if workflow/architecture changes.

## Known limitations (v0.1)

- Batch TTS concurrency is configurable in Settings (1–5); higher values increase API load
- Custom voice **create via API** requires xAI Enterprise; free tier uses [Voice Library console](https://console.x.ai/team/default/voice/voice-library)
- Custom voices region: US only (per xAI docs)

## Links

- [xAI TTS docs](https://docs.x.ai/developers/model-capabilities/audio)
- [xAI Custom Voices](https://docs.x.ai/developers/model-capabilities/audio/custom-voices)
- [Tauri v2](https://v2.tauri.app/)

## Contributing

Issues and PRs welcome at [github.com/stevenke1981/Grok-Voice-Studio](https://github.com/stevenke1981/Grok-Voice-Studio).