# Transcripts

PodFetch supports [Podcasting 2.0 transcripts](https://podcasting2.org/podcast-namespace/tags/transcript)
end to end:

- **Feed transcripts** — when a feed episode carries `<podcast:transcript>`
  tags, PodFetch records them during the feed refresh. Once the episode is
  downloaded, the best matching transcript (preferring VTT over SRT over JSON
  over HTML, and the podcast's language when several are offered) is
  downloaded, archived next to the episode's audio file and parsed into
  segments.
- **Player integration** — the detailed audio player has a *Transcript* tab
  that follows the playback position, supports auto-scroll and lets you jump
  to a segment by clicking it.
- **Full-text search** — every parsed transcript is indexed in the database
  (SQLite FTS5 or PostgreSQL `tsvector`). The episode search page has a
  *Transcripts* mode that finds spoken words and jumps straight to the
  matching position in the episode.
- **Generated transcripts** — episodes without a feed transcript can be
  transcribed with any OpenAI-compatible Whisper API (see below), either
  manually per episode or automatically after each download.
- **RSS re-export** — archived transcripts are included as
  `<podcast:transcript>` tags in the RSS feeds PodFetch generates, so other
  podcast clients can use them too.

## Whisper transcription setup

Generated transcripts use an OpenAI-compatible `audio/transcriptions`
endpoint. Any server that speaks this protocol works. For a fully
self-hosted setup with [whisper.cpp](https://github.com/ggml-org/whisper.cpp)
— including ready-to-run CPU and NVIDIA GPU compose files — see the
[Local transcription with whisper.cpp](./tutorials/WhisperCpp.md) tutorial.
Another good choice is [speaches](https://speaches.ai/) (the successor of
`faster-whisper-server`):

```yaml
services:
  podfetch:
    image: samtv12345/podfetch
    environment:
      - TRANSCRIPTION_API_BASE_URL=http://speaches:8000
      - TRANSCRIPTION_MODEL=Systran/faster-whisper-small
    # ... your existing podfetch configuration

  speaches:
    image: ghcr.io/speaches-ai/speaches:latest-cpu
    volumes:
      - hf-hub-cache:/home/ubuntu/.cache/huggingface/hub

volumes:
  hf-hub-cache:
```

Transcription is enabled as soon as `TRANSCRIPTION_API_BASE_URL` is set. The
UI then shows a *Transcribe* action on downloaded episodes and an
*Auto-transcribe* toggle in each podcast's settings. Jobs run in a background
queue; failures are retried up to three times and their status is pushed live
to the UI.

## Environment variables

| Variable | Required | Default | Description |
|---|---|---|---|
| `TRANSCRIPTION_API_BASE_URL` | yes (to enable the feature) | – | Base URL of the OpenAI-compatible API, e.g. `http://speaches:8000` or `https://api.openai.com` |
| `TRANSCRIPTION_API_KEY` | no | – | Bearer token sent to the transcription API, if it requires one |
| `TRANSCRIPTION_MODEL` | no | `whisper-1` | Model name passed to the API, e.g. `Systran/faster-whisper-small` |

## Re-parsing archived transcripts

Admins can re-parse all archived transcript files (e.g. after a parser
improvement) via `POST /api/v1/settings/transcripts/reparse`.
