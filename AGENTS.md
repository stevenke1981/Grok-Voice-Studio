# AGENTS.md — Grok Voice Studio

Instructions for coding agents (Cursor, Grok, Codex, etc.) working in this repository.

## Project summary

**Grok Voice Studio** — Tauri v2 desktop app for multi-character xAI TTS dubbing.

| Layer | Path | Responsibility |
|-------|------|----------------|
| Core | `crates/core/` | `Project`, `ScriptSegment`, parser, timeline, `sfx` catalog |
| xAI | `crates/providers/xai/` | TTS, Grok Chat (story mode), custom voices API |
| Audio | `crates/audio/` | FFmpeg concat, SRT/VTT/ASS |
| Storage | `crates/storage/` | SQLite cache, keyring API key, settings, `SfxStore` |
| Desktop | `apps/desktop/` | React UI + `src-tauri/` commands |

**Human docs:** [README.md](./README.md) (EN), [README.zh-TW.md](./README.zh-TW.md) (zh-TW)

## Commands (run from repo root)

```bash
pnpm install              # once
pnpm dev                  # Tauri dev (port 1420)
pnpm test                 # cargo test --workspace
pnpm build                # release installer
pnpm exec tsc --noEmit --prefix apps/desktop
```

Windows: prefer PowerShell-native commands (`Get-Content`, `Select-String`); avoid `head`/`tail`/`grep`.

## Critical user workflow

Users **must parse script** before generation works:

1. Enter `script_raw` in dialogue editor
2. Invoke `parse_script_command` → populates `project.segments`
3. Assign `voice_profile.voice_id` per character
4. `start_generate_job` (requires API key for dialogue segments; SFX-only may not)

UI exposes this via `WorkflowGuide` — keep parse step prominent when `segments.length === 0`.

## Domain rules

### Script parsing (`crates/core/src/parser.rs`)

- Dialogue: `角色：台詞`, `角色（情緒）：台詞`, `[角色] 台詞`
- SFX line: `音效：名稱`, `【音效】名稱`, `[SFX] name`
- Inline SFX: `{名稱}` stripped from TTS text → `segment.sfx_cues`
- SFX segments: `segment_kind: Sfx`, no TTS — resolved from `SfxStore`

### Voices (`crates/providers/xai/`)

- Built-in: `GET https://api.x.ai/v1/tts/voices`
- Custom: `GET https://api.x.ai/v1/custom-voices` (separate list)
- `sync_voices` / `list_all_voices()` merges both; `VoiceInfo.is_custom`
- TTS uses same `voice_id` for both types
- Streaming: `wss://api.x.ai/v1/tts` via `streaming_tts.rs` (`text.delta` / `audio.delta`); setting `use_streaming_tts` (default on), REST fallback on failure
- Retries: `crates/providers/xai/src/retry.rs` — up to 5 attempts with exponential backoff; honors `Retry-After` on HTTP 429; retries transient 5xx/network/timeouts; `synthesize_preferred` wraps streaming+REST as one attempt
- Retry UI hook: `crates/core/src/retry_notify.rs` installed in `lib.rs` setup — logs to **日誌**, emits `api-retry` (all categories), and `generate-progress` status `retrying` during batch TTS
- Retry jitter: `retry.rs` `backoff_delay_with_jitter` desyncs parallel batch workers; `sync_voices` / `preview_voice` use `with_retry_context`
- Batch stats in `settings.json`: `last_batch_retry_count`, `suggested_concurrency`; `apply_concurrency_suggestion` / `dismiss_concurrency_suggestion` commands
- Story mode Chat: `chat.rs` uses same retry policy; JSON parse failures get one extra LLM attempt via `story_to_script_with_retry`

### Generation (`apps/desktop/src-tauri/src/services/generation.rs`)

- `useGeneration` hook must reload project via `get_project` on `generate-progress` events (avoid stale React closure)
- `start_generate_job` validates: non-empty segments, API key if pending dialogue exists
- SFX segments: `resolve_sfx_segment` — no API key
- Batch jobs honor `generation_concurrency` (Settings, 1–5) via `Semaphore` + `JoinSet`; `persist_project` runs once after batch
- TTS text: `grok_voice_core::build_tts_text` applies `emotion_hint` or `style_prompt` as `[tag]` prefix

### Export (`crates/audio/src/ffmpeg.rs`)

- `concat_segments(..., sfx_paths)` interleaves voice + inline/standalone SFX

### Logging

- In-memory ring buffer: `log_store.rs`, commands `get_logs` / `clear_logs`
- UI: topbar **日誌** button

### SFX UI

- **Floating** `SfxLibraryWindow` (draggable/resizable) — not embedded in `ScriptEditor` panel
- State: `showSfxLibrary` in `App.tsx`

## Tauri command registry

Source of truth: `apps/desktop/src-tauri/src/lib.rs`

Do not add commands without registering in `invoke_handler` and wiring TypeScript types if needed.

## Frontend conventions

- i18n: `apps/desktop/src/i18n/index.ts` — add keys to **both** `zh` and `en`
- Types: `apps/desktop/src/types.ts`
- CSS: `apps/desktop/src/App.css`
- Prefer functional components + hooks; invoke backend via `@tauri-apps/api/core`

## Testing & verification

Minimum before claiming done:

```bash
cargo test --workspace
pnpm exec tsc --noEmit --prefix apps/desktop
```

For UI changes: run `pnpm dev` and verify workflow (parse gate, SFX window, voice groups).

## What not to commit

- `target/`, `node_modules/`, `dist/`, `*.db`
- `terminals/`, `mcps/` (IDE harness)
- API keys, `.env` (only `.env.example`)

## Safe change patterns

| Task | Start here |
|------|------------|
| Parser / script syntax | `crates/core/src/parser.rs` + tests in same file |
| New segment fields | `models.rs` + `types.ts` + `apply_parsed_script` |
| TTS / voices / streaming WS | `crates/providers/xai/` (`streaming_tts.rs`) |
| New Tauri command | `commands.rs` → `lib.rs` → React invoke |
| UI panel | `apps/desktop/src/components/` |
| Export / subtitles | `crates/audio/` |

## API references

- [xAI TTS](https://docs.x.ai/developers/model-capabilities/audio)
- [xAI Custom Voices](https://docs.x.ai/developers/model-capabilities/audio/custom-voices)

## Agent workflow

1. Read `AGENTS.md` + skim README for user-facing behavior
2. Locate code via `crates/` layout or ripgrep (graph MCP if indexed)
3. Keep diffs scoped; match existing naming and patterns
4. Update i18n (zh + en) for UI strings
5. Run `cargo test` and `tsc` before finishing
6. Update README/AGENTS if behavior or architecture changes materially

Do not ask the user to choose between numbered options — pick the highest-impact next step and continue unless blocked on credentials or destructive actions.