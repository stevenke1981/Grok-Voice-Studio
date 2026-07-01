use uuid::Uuid;

use crate::models::{ScriptSegment, SegmentKind, Timeline, TimelineClip, TimelineTrack};

pub fn build_timeline_from_segments(segments: &[ScriptSegment]) -> Timeline {
    let mut tracks: Vec<TimelineTrack> = Vec::new();
    let mut narration_clips = Vec::new();
    let mut character_clips = Vec::new();
    let mut sfx_clips = Vec::new();
    let mut current_ms: u64 = 0;

    for seg in segments {
        let duration = seg.duration_ms.unwrap_or(if seg.segment_kind == SegmentKind::Sfx {
            1500
        } else {
            2000
        });
        let clip = TimelineClip {
            id: Uuid::new_v4().to_string(),
            segment_id: seg.id.clone(),
            start_ms: current_ms,
            duration_ms: duration,
            volume_db: 0.0,
            fade_in_ms: 0,
            fade_out_ms: 0,
        };
        current_ms += duration + seg.pause_after_ms as u64;

        if seg.segment_kind == SegmentKind::Sfx {
            sfx_clips.push(clip);
        } else if seg.order % 2 == 0 {
            narration_clips.push(clip);
        } else {
            character_clips.push(clip);
        }
    }

    if !narration_clips.is_empty() {
        tracks.push(TimelineTrack {
            id: Uuid::new_v4().to_string(),
            name: "旁白軌".into(),
            track_type: "narration".into(),
            clips: narration_clips,
        });
    }
    if !character_clips.is_empty() {
        tracks.push(TimelineTrack {
            id: Uuid::new_v4().to_string(),
            name: "角色軌".into(),
            track_type: "character".into(),
            clips: character_clips,
        });
    }
    if !sfx_clips.is_empty() {
        tracks.push(TimelineTrack {
            id: Uuid::new_v4().to_string(),
            name: "音效軌".into(),
            track_type: "sfx".into(),
            clips: sfx_clips,
        });
    }

    Timeline { tracks }
}