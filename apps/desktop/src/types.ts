export interface VoiceProfile {
  provider: string;
  voice_id: string;
  language: string;
  style_prompt?: string;
  volume_db: number;
  pan: number;
}

export interface Character {
  id: string;
  name: string;
  role_type: "narrator" | "character" | "system";
  voice_profile: VoiceProfile;
  color: string;
}

export type SegmentKind = "dialogue" | "sfx";

export interface SfxCue {
  sfx_id: string;
  label: string;
  volume_db?: number;
}

export interface SoundEffect {
  id: string;
  name: string;
  name_en: string;
  category: string;
  duration_ms: number;
  tags: string[];
  file_name: string;
  builtin: boolean;
}

export interface ScriptSegment {
  id: string;
  character_id: string;
  text: string;
  language: string;
  emotion_hint?: string;
  speech_tags: string[];
  segment_kind?: SegmentKind;
  sfx_id?: string;
  sfx_cues?: SfxCue[];
  order: number;
  pause_after_ms: number;
  audio_asset_id?: string;
  audio_path?: string;
  duration_ms?: number;
  status: "pending" | "cached" | "generating" | "done" | "failed";
  error_message?: string;
}

export interface Project {
  version: number;
  id: string;
  title: string;
  script_raw: string;
  characters: Character[];
  segments: ScriptSegment[];
  timeline: { tracks: unknown[] };
  export_presets: unknown[];
  created_at: string;
  updated_at: string;
}

export interface VoiceInfo {
  voice_id: string;
  name: string;
  language?: string;
  description?: string;
  is_custom?: boolean;
  tone?: string;
  use_case?: string;
  gender?: string;
}

export interface ExportOptions {
  codec: "mp3" | "wav" | "flac" | "pcm";
  sample_rate: number;
  bit_rate?: number;
  normalize: boolean;
  subtitle_format: "srt" | "vtt" | "ass";
  show_character_in_subtitle: boolean;
  export_stems: boolean;
}

export interface ExportResult {
  audio_path: string;
  subtitle_path?: string;
  stem_paths: string[];
  duration_ms: number;
}

export interface RecentProject {
  id: string;
  title: string;
  path: string;
}

export interface ProjectStats {
  total_segments: number;
  done_segments: number;
  total_chars: number;
  estimated_cost?: number;
}

export interface GenerateProgressEvent {
  job_id: string;
  current: number;
  total: number;
  segment_id?: string;
  segment?: ScriptSegment;
  status: string;
  error?: string;
  cached: boolean;
}

export interface LogEntry {
  id: number;
  timestamp: string;
  level: string;
  category: string;
  message: string;
}

export interface AppSettings {
  has_api_key: boolean;
  ffmpeg_path?: string;
  default_language: string;
  auto_save: boolean;
  generation_concurrency: number;
  cost_per_1k_chars?: number;
  onboarding_done: boolean;
  ui_language: string;
  use_streaming_tts: boolean;
}