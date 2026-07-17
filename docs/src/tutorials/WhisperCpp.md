# Local transcription with whisper.cpp

This tutorial sets up fully local, self-hosted episode transcription for
PodFetch using [whisper.cpp](https://github.com/ggml-org/whisper.cpp) — no
cloud API, no API key, and everything runs in Docker via `docker compose`.
If you haven't read it yet, the [Transcripts](../transcripts.md) page explains
what PodFetch does with transcripts once they exist.

## How it works

PodFetch talks to any OpenAI-compatible `audio/transcriptions` endpoint: it
POSTs the episode's audio file to `{TRANSCRIPTION_API_BASE_URL}/v1/audio/transcriptions`
and expects a `verbose_json` response with timed segments.

whisper.cpp ships an HTTP server (`whisper-server`) whose transcription
endpoint lives at `/inference` by default — but it can be remapped with
`--inference-path`. Pointing it at `/v1/audio/transcriptions` turns
whisper.cpp into a drop-in replacement for the OpenAI API. Its `verbose_json`
response contains exactly the `segments` (with `start`/`end` in seconds) and
`language` fields PodFetch parses, so no adapter or proxy is needed.

The official Docker images already contain `ffmpeg`, so the server can accept
MP3/M4A/OGG uploads directly when started with `--convert`.

## Choosing a model

whisper.cpp uses ggml model files from
[huggingface.co/ggerganov/whisper.cpp](https://huggingface.co/ggerganov/whisper.cpp).
Bigger models are more accurate but slower — on CPU dramatically so. Rough
guidance:

| Model | Download size | RAM (approx.) | Recommendation |
|---|---|---|---|
| `tiny` | 75 MiB | ~0.4 GiB | Fastest, noticeably inaccurate. Only for very weak hardware. |
| `base` | 142 MiB | ~0.6 GiB | Quick tests. |
| `small` | 466 MiB | ~1.2 GiB | **Best CPU default** — good accuracy at usable speed. |
| `medium` | 1.5 GiB | ~2.9 GiB | Better accuracy, slow on CPU. |
| `large-v3` | 2.9 GiB | ~4.4 GiB | Best accuracy. Realistically needs a GPU. |
| `large-v3-turbo` | 1.5 GiB | ~2.6 GiB | **Best GPU default** — near `large-v3` accuracy, much faster. |

All of these are multilingual. English-only variants (`tiny.en`, `base.en`,
`small.en`, `medium.en`) are slightly better for purely English podcasts.
Quantized variants (e.g. `small-q8_0`) cut RAM and size further at a small
accuracy cost — see the whisper.cpp README for the full list.

> **Keep an eye on the 10-minute budget.** PodFetch aborts a transcription
> request after 10 minutes. A 1-hour episode on `small` typically finishes
> well within that on a modern CPU, but `medium`/`large` on CPU will not.
> If jobs fail with timeouts, pick a smaller model or use the GPU setup.

## CPU deployment

A complete, self-contained `docker-compose.yml`. It runs three services:

- **`podfetch`** — the PodFetch server, with transcription pointed at the
  whisper.cpp container.
- **`whisper-model-downloader`** — a one-shot init container that downloads
  the ggml model into a shared volume on first start and exits. On later
  starts it sees the file already exists and exits immediately.
- **`whisper`** — `whisper-server`, started only after the model download
  completed successfully.

```yaml
services:
  podfetch:
    image: samuel19982/podfetch:latest
    user: ${UID:-1000}:${GID:-1000}
    restart: unless-stopped
    ports:
      - "80:8000"
    volumes:
      - podfetch-podcasts:/app/podcasts
      - podfetch-db:/app/db
    environment:
      - POLLING_INTERVAL=60
      - DATABASE_URL=sqlite:///app/db/podcast.db
      - TRANSCRIPTION_API_BASE_URL=http://whisper:8080
    depends_on:
      - whisper

  whisper-model-downloader:
    image: ghcr.io/ggml-org/whisper.cpp:main
    volumes:
      - whisper-models:/models
    command: >-
      [ -f /models/ggml-small.bin ] ||
      ./models/download-ggml-model.sh small /models

  whisper:
    image: ghcr.io/ggml-org/whisper.cpp:main
    restart: unless-stopped
    volumes:
      - whisper-models:/models
    command: >-
      whisper-server
      --model /models/ggml-small.bin
      --host 0.0.0.0 --port 8080
      --inference-path /v1/audio/transcriptions
      --convert
      --language auto
    depends_on:
      whisper-model-downloader:
        condition: service_completed_successfully

volumes:
  podfetch-podcasts:
  podfetch-db:
  whisper-models:
```

Start it:

```bash
docker compose up -d
```

The first start downloads the model (~466 MiB for `small`); watch it with
`docker compose logs -f whisper-model-downloader`. To switch models later,
change the model name in **both** the downloader command and the
`--model` flag, then `docker compose up -d` again.

Notes on the compose file:

- The whisper.cpp image's entrypoint is `bash -c`, which is why `command`
  is a single shell string rather than an argument list.
- `whisper` is intentionally **not** exposed with a `ports:` mapping —
  PodFetch reaches it over the internal compose network at
  `http://whisper:8080`, and the server has no authentication, so it should
  not be reachable from outside. If you want to test it from the host,
  add `ports: ["8080:8080"]` temporarily.
- If you already run PodFetch, you can also just add the two `whisper*`
  services and the `whisper-models` volume to your existing compose file and
  set `TRANSCRIPTION_API_BASE_URL` on your existing PodFetch service.

## NVIDIA GPU deployment

With a GPU, `large-v3-turbo` transcribes an hour-long episode in a couple of
minutes. Requirements on the host: an NVIDIA GPU with current drivers and the
[NVIDIA Container Toolkit](https://docs.nvidia.com/datacenter/cloud-native/container-toolkit/latest/install-guide.html)
(check with `docker run --rm --gpus all nvidia/cuda:12.4.1-base-ubuntu22.04 nvidia-smi`).

Only the `whisper*` services change compared to the CPU setup — use the
`main-cuda` image, request the GPU, and load a bigger model:

```yaml
  whisper-model-downloader:
    image: ghcr.io/ggml-org/whisper.cpp:main-cuda
    volumes:
      - whisper-models:/models
    command: >-
      [ -f /models/ggml-large-v3-turbo.bin ] ||
      ./models/download-ggml-model.sh large-v3-turbo /models

  whisper:
    image: ghcr.io/ggml-org/whisper.cpp:main-cuda
    restart: unless-stopped
    volumes:
      - whisper-models:/models
    command: >-
      whisper-server
      --model /models/ggml-large-v3-turbo.bin
      --host 0.0.0.0 --port 8080
      --inference-path /v1/audio/transcriptions
      --convert
      --language auto
    depends_on:
      whisper-model-downloader:
        condition: service_completed_successfully
    deploy:
      resources:
        reservations:
          devices:
            - driver: nvidia
              count: 1
              capabilities: [gpu]
```

> **Apple Silicon / macOS:** Docker containers on macOS cannot use the GPU
> (Metal is not available inside Docker). The CPU compose file above works
> fine on a Mac; for GPU-speed transcription on Apple hardware run
> `whisper-server` natively outside Docker instead and point
> `TRANSCRIPTION_API_BASE_URL` at it (e.g. `http://host.docker.internal:8080`).

## What the server flags mean

| Flag | Why it matters |
|---|---|
| `--model /models/ggml-….bin` | The ggml model file to load. Fixed for the lifetime of the server — see the note about `TRANSCRIPTION_MODEL` below. |
| `--host 0.0.0.0` | By default the server only listens on `127.0.0.1`, which would make it unreachable from the PodFetch container. |
| `--port 8080` | Must match the port in `TRANSCRIPTION_API_BASE_URL`. |
| `--inference-path /v1/audio/transcriptions` | Remaps the endpoint from `/inference` to the OpenAI-compatible path PodFetch calls. **Without this flag PodFetch gets a 404.** |
| `--convert` | Converts uploads to 16-kHz WAV with the ffmpeg bundled in the image. **Without this flag MP3 episodes are rejected** — the server natively only accepts WAV. |
| `--language auto` | Auto-detect the spoken language per episode. **The default is `en`**, which produces garbage (or unwanted translations) for non-English podcasts. If your library is entirely one language you can pin it, e.g. `--language de`. |
| `--threads N` (optional) | CPU threads for inference, defaults to 4. Raise it on beefier machines. |

## PodFetch configuration

Only one variable is required:

| Variable | Value |
|---|---|
| `TRANSCRIPTION_API_BASE_URL` | `http://whisper:8080` (service name + port inside the compose network) |
| `TRANSCRIPTION_API_KEY` | Leave unset — `whisper-server` has no authentication. |
| `TRANSCRIPTION_MODEL` | Leave unset. PodFetch sends it, but whisper.cpp **ignores** the request's model field — the model is fixed by the server's `--model` flag. To change models, change the compose file, not this variable. |

As soon as `TRANSCRIPTION_API_BASE_URL` is set, downloaded episodes get a
*Transcribe* action and each podcast's settings gain an *Auto-transcribe*
toggle. See [Transcripts](../transcripts.md) for how generated transcripts
are archived, searched and re-exported.

## Verifying the setup

1. **Server up?**

   ```bash
   docker compose logs whisper | tail -n 5
   ```

   You should see `whisper server listening at http://0.0.0.0:8080`.

2. **Endpoint answers?** (add `ports: ["8080:8080"]` to the `whisper`
   service temporarily, or run this from any container on the same network)

   ```bash
   curl -s http://localhost:8080/v1/audio/transcriptions \
     -F file=@some-audio.mp3 \
     -F response_format=verbose_json | head -c 400
   ```

   The response should be JSON starting with `"task": "transcribe"` and
   containing a `segments` array.

3. **End to end:** open a downloaded episode in PodFetch, click
   *Transcribe*, and watch `docker compose logs -f whisper` — you'll see the
   request arrive and the segments being decoded. When the job finishes, the
   player's *Transcript* tab shows the result and the episode becomes
   findable via transcript search.

## Troubleshooting

- **Transcript is in English although the podcast isn't / is gibberish** —
  the server is running without `--language auto` and defaulted to English.
  Add the flag and restart.
- **PodFetch logs `HTTP status 404`** — `--inference-path
  /v1/audio/transcriptions` is missing, so the server only serves
  `/inference`.
- **Jobs fail after exactly 10 minutes** — the episode takes longer to
  transcribe than PodFetch's client-side timeout. Use a smaller model, raise
  `--threads`, or switch to the GPU setup. Failed jobs are retried up to
  three times, but they will keep hitting the same limit.
- **Server rejects the upload / errors about WAV** — `--convert` is missing.
- **`whisper` never starts** — the model download failed (network, disk
  space). Check `docker compose logs whisper-model-downloader`; the file must
  exist in the volume before the server starts. Rerun with
  `docker compose up -d` after fixing the cause.
- **Container is OOM-killed** — the model doesn't fit in RAM (see the table
  above). Pick a smaller or quantized model.
- **GPU container falls back to CPU** — verify the NVIDIA Container Toolkit
  works (`docker run --rm --gpus all … nvidia-smi`) and that your compose
  version honors `deploy.resources.reservations.devices` (Compose v2 does).
