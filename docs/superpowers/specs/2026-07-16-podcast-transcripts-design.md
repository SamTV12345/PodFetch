# Design: Podcasting-2.0-Transkript-Unterstützung

Datum: 2026-07-16
Status: Entwurf, vom Maintainer abgesegnet (Konversation), wartet auf Spec-Review

## Ziel

PodFetch soll Episoden-Transkripte vollständig unterstützen:

1. **Konsumieren**: `<podcast:transcript>`-Tags aus RSS-Feeds parsen (Formate: JSON, VTT, SRT, HTML).
2. **Archivieren**: Transkript-Dateien beim Episoden-Download lokal (Disk/S3) mit ablegen.
3. **Anzeigen**: Transkript im Player mit Auto-Scroll und Klick-zu-Zeitstempel.
4. **Durchsuchen**: Volltextsuche über Transkript-Inhalte mit zeitgenauen Treffern.
5. **Erzeugen**: Fehlende Transkripte optional per Whisper über eine OpenAI-kompatible
   Transcription-API generieren (`/v1/audio/transcriptions`).
6. **Weitergeben**: Vorhandene Transkripte im von PodFetch generierten RSS-Feed als
   `<podcast:transcript>` re-exportieren, damit externe Clients (AntennaPod, ABS-Apps) sie sehen.

## Entscheidungen (mit Begründung)

| Entscheidung | Wahl | Begründung |
|---|---|---|
| Speicherform | Hybrid: Originaldatei archivieren + Segmente in DB | Archiv-Gedanke (Original bleibt), zeitgenaue Suche, Re-Parse nach Parser-Fixes möglich |
| Volltextsuche | Native DB-FTS: SQLite FTS5 / Postgres tsvector+GIN | Keine neue Infrastruktur, passt zur Ein-Binary-Philosophie |
| STT-Anbindung | OpenAI-kompatible HTTP-API (konfigurierbar) | Funktioniert mit faster-whisper/speaches/LocalAI lokal und OpenAI/Groq in der Cloud; kein C++-Build, kein Modell-Management |
| STT-Trigger | Per-Podcast-Opt-in (`auto_transcribe`) + manueller Button | Kontrollierbare Last, keine Überraschungs-Warteschlangen |
| Bevorzugtes Format | JSON > VTT > SRT > HTML/Text; Sprache = Podcast-Sprache | JSON/VTT tragen die reichsten Zeit-/Sprecher-Infos |

## Datenmodell

Migrationen für SQLite **und** Postgres (Muster: `podcast_episode_chapters`).
`migrations/mysql/` wird nicht bedient (MySQL ist nicht aktiv angebunden).

### Tabelle `podcast_episode_transcripts`

Eine Zeile pro Transkript-Variante einer Episode (Feeds liefern oft mehrere Formate/Sprachen).

| Spalte | Typ | Bemerkung |
|---|---|---|
| `id` | UUID (Text) | PK |
| `episode_id` | UUID (Text) | FK auf `podcast_episodes`, ON DELETE CASCADE |
| `source` | Text | `feed` \| `generated` |
| `original_url` | Text, nullable | URL aus dem Tag; NULL bei `generated` |
| `file_path` | Text, nullable | archivierte Kopie; NULL bis heruntergeladen |
| `mime_type` | Text | z.B. `text/vtt`, `application/json` |
| `language` | Text, nullable | aus dem Tag-Attribut |
| `is_preferred` | Bool | genau eine Variante pro Episode wird angezeigt/indexiert |
| `status` | Text | `pending` \| `downloaded` \| `parsed` \| `failed` |
| `error` | Text, nullable | letzter Fehler, sichtbar in der UI |
| `created_at`, `updated_at` | Timestamp | |

Upsert-Schlüssel: (`episode_id`, `original_url`) — Feed-Refreshs erzeugen keine Duplikate.
Generierte Transkripte: eindeutig über (`episode_id`, `source='generated'`), max. eines pro Episode.

**Präferenzregeln** (`is_preferred`, bei jeder Änderung an den Transkripten einer Episode neu bestimmt):
1. Nur Varianten mit `status='parsed'` kommen infrage (nicht parsebare Feed-Transkripte
   blockieren also weder Anzeige noch Whisper-Fallback).
2. `feed` schlägt `generated`; innerhalb `feed`: Format JSON > VTT > SRT > HTML/Text,
   bei Gleichstand die Variante in der Podcast-Sprache.
3. Kommt später ein parsebares Feed-Transkript hinzu (Feed-Refresh), wechselt die
   Präferenz vom generierten auf das Feed-Transkript; die Segmente werden entsprechend
   neu aufgebaut. Das generierte bleibt archiviert.
4. `auto_transcribe` legt nur dann einen Job an, wenn keine Variante mit
   `status IN ('pending','parsed')` existiert — ein `failed`-Feed-Transkript löst
   also den Whisper-Fallback aus.

### Tabelle `podcast_episode_transcript_segments`

Normalisierte Cues des bevorzugten Transkripts.

| Spalte | Typ | Bemerkung |
|---|---|---|
| `id` | UUID (Text) | PK |
| `transcript_id` | UUID (Text) | FK, ON DELETE CASCADE |
| `idx` | Integer | Reihenfolge |
| `start_ms`, `end_ms` | Integer, nullable | NULL bei HTML/Text-Transkripten ohne Zeiten |
| `speaker` | Text, nullable | |
| `text` | Text | |

### Volltext-Index (raw SQL in den Migrationen, nicht im Diesel-Schema)

- **SQLite**: FTS5-Virtual-Table `transcript_segments_fts` als External-Content-Table
  auf die Segment-Tabelle; Synchronisation über INSERT/UPDATE/DELETE-Trigger
  (Standard-FTS5-Muster, Text liegt nicht doppelt in der DB).
- **Postgres**: generierte `tsvector`-Spalte (`GENERATED ALWAYS AS (to_tsvector('simple', text)) STORED`)
  mit GIN-Index. Konfiguration `simple`, da Bibliotheken gemischt-sprachig sind.

### Tabelle `transcription_jobs`

| Spalte | Typ | Bemerkung |
|---|---|---|
| `id` | UUID (Text) | PK |
| `episode_id` | UUID (Text) | FK, UNIQUE (max. ein Job pro Episode) |
| `status` | Text | `pending` \| `running` \| `done` \| `failed` |
| `attempts` | Integer | max. 3 |
| `error` | Text, nullable | |
| `created_at`, `updated_at` | Timestamp | |

### Erweiterung `podcast_settings`

Neue Spalte `auto_transcribe` (Bool, Default `false`): Episoden dieses Podcasts nach dem
Download automatisch transkribieren, **wenn kein Feed-Transkript existiert**.

## Konfiguration

Global über Env (Muster `common-infrastructure/src/config.rs`):

- `TRANSCRIPTION_API_BASE_URL` — z.B. `http://speaches:8000`; **nicht gesetzt ⇒ alle
  Whisper-Funktionen deaktiviert** (UI blendet sie aus, Endpunkte antworten 503;
  gleiche Mechanik wie `GPODDER_INTEGRATION_ENABLED`)
- `TRANSCRIPTION_API_KEY` — optional (lokale Dienste brauchen keinen)
- `TRANSCRIPTION_MODEL` — Default `whisper-1`

Der Konfigurationszustand (nur „aktiviert ja/nein", nie der Key) wird über `/api/v1/sys/config`
ans Frontend gereicht.

## Architektur (Backend)

Dem Kapitel-Muster folgend:

- `podfetch-domain/src/podcast_episode_transcript.rs` — Entities + Trait
  `PodcastEpisodeTranscriptRepository` (upsert, get_by_episode, set_status,
  replace_segments, `search(query, podcast_id, page)`), Trait `TranscriptionJobRepository`
- `podfetch-persistence/src/podcast_episode_transcript.rs` — Diesel-Implementierung;
  `search()` mit zwei handgeschriebenen `sql_query`-Zweigen (FTS5-`MATCH`+`bm25`+`highlight`
  bzw. `websearch_to_tsquery('simple', …)`+`ts_rank`+`ts_headline`)
- `podfetch-web/src/services/transcript/` —
  - `parser.rs`: vier Parser (JSON, VTT, SRT, HTML) → `Vec<ParsedSegment>`;
    VTT/SRT über eine kleine bestehende Crate, JSON/HTML von Hand
  - `service.rs`: Auswahl des bevorzugten Transkripts, Download+Archivierung
    (über `podfetch-storage`, Benennung `<episodendatei-stem>.transcript.<ext>`),
    Parse+Persist, Re-Parse
  - `whisper_client.rs`: multipart-POST an `{base_url}/v1/audio/transcriptions`
    mit `response_format=verbose_json`; Antwort-Segmente → VTT-Datei + Segmente
  - `worker.rs`: Tokio-Worker, arbeitet `transcription_jobs` sequenziell ab (1 Job
    gleichzeitig); beim Serverstart werden `running`-Jobs auf `pending` zurückgesetzt
- `podfetch-web/src/controllers/transcript_controller.rs` — Endpunkte (unten)
- Einbindung in `app_state.rs`, `startup.rs`

## Ingestion-Flows

1. **Feed-Parse** (`usecases/podcast_episode/mod.rs::insert_podcast_episodes`):
   `<podcast:transcript>`-Tags aus `item.extensions()["podcast"]["transcript"]` lesen
   (Attribute `url`, `type`, `language`, `rel`) und als `pending`-Zeilen upserten.
   **Kein HTTP-Fetch hier** — Feed-Refresh bleibt schnell.
2. **Episoden-Download** (`services/download/service.rs`, neben dem bestehenden
   Kapitel-Handling): alle `pending`-Transkripte der Episode herunterladen (Timeout 30 s,
   Größenlimit 20 MB, Content-Type-Plausibilität), archivieren, bevorzugte Variante
   parsen, Segmente ersetzen (delete+insert in Transaktion). Fehler ⇒ `status='failed'`
   + `error`; der Episoden-Download schlägt dadurch **nie** fehl.
3. **Whisper**: Job-Erzeugung (a) automatisch nach Episoden-Download bei
   `auto_transcribe=true` und fehlendem Feed-Transkript, (b) manuell per Endpoint.
   Worker lädt die lokale Audiodatei hoch, speichert Ergebnis als VTT (`source='generated'`,
   `is_preferred` nur wenn kein Feed-Transkript existiert) + Segmente. Max. 3 Versuche.
   Statuswechsel werden per SocketIO-Event `transcriptionStatus` gepusht.
4. **Re-Parse** (Admin): Segmente aller Transkripte aus den archivierten Dateien neu
   aufbauen (nach Parser-Fixes), analog zum bestehenden `resync-db`-Gedanken.

## HTTP-API

| Methode & Pfad | Zweck | Berechtigung |
|---|---|---|
| `GET /api/v1/podcasts/episodes/{id}/transcripts` | Varianten-Liste (Metadaten) | eingeloggt |
| `GET /api/v1/podcasts/episodes/{id}/transcript` | Segmente des bevorzugten Transkripts (JSON) | eingeloggt |
| `GET /api/v1/podcasts/episodes/{id}/transcripts/{tid}/file` | archivierte Originaldatei, korrekter Content-Type; zusätzlich apiKey-Variante für externe Clients | eingeloggt / apiKey |
| `POST /api/v1/podcasts/episodes/{id}/transcribe` | Whisper-Job einreihen; 409 bei laufendem Job, 503 ohne STT-Konfiguration | Rolle Uploader+ |
| `GET /api/v1/transcripts/search?q=&podcast_id=&page=` | Volltextsuche, gruppiert nach Episode (beste 3 Segmente pro Episode), Rang-sortiert, mit Highlight + `start_ms` | eingeloggt |
| `POST /api/v1/settings/transcripts/reparse` | Re-Parse aus Archivdateien | Admin |

**RSS-Re-Export** (`rss.rs`): pro Episode mit vorhandenem (`parsed`/`downloaded`) Transkript
ein `<podcast:transcript url="…" type="…"/>`-Tag auf die apiKey-Datei-Route.

## UI

- **Player** (`DetailedAudioPlayer`): Transkript-Tab neben der Kapiteltabelle
  (`PodcastEpisodeChapterTable`-Muster). Segmentliste; aktives Segment folgt der
  Wiedergabeposition (Auto-Scroll, abschaltbar); Klick auf Segment springt zur Stelle
  (bestehende Seek-Mechanik). Ohne Zeitstempel (HTML): reine Textanzeige.
- **Suche** (`EpisodeSearchPage`): Umschalter „Titel/Beschreibung | Transkripte".
  Transkript-Treffer: Episode, Highlight-Satz, Zeitstempel; Klick startet Wiedergabe
  an der Stelle (`PlayHandler` + Seek).
- **Podcast-Einstellungen**: Schalter „Automatisch transkribieren" neben
  auto_download/auto_cleanup; nur sichtbar, wenn STT konfiguriert (via `/sys/config`).
- **Episoden-Eintrag**: Transkript-Indikator; Aktion „Transkribieren" mit Status
  (pending/running/failed inkl. Fehlertext) über SocketIO-Updates.
- **i18n**: neue Keys in allen 7 Sprachdateien (da, de, en, es, fr, pl, zh).

## Fehlerbehandlung

- Transkript-Fehler sind für umgebende Flows (Feed-Refresh, Episoden-Download)
  immer non-fatal; jeder Fehler landet sichtbar in `status`/`error`.
- Nicht parsebare Dateien werden trotzdem archiviert (`status='downloaded'`).
- Whisper-Queue liegt in der DB und überlebt Neustarts.

## Tests

- **Parser**: Fixtures für alle vier Formate + kaputte Varianten (fehlende Zeiten,
  leere Datei, kaputtes Encoding).
- **Feed-Parse**: RSS-Fixture mit mehreren transcript-Tags → korrekte Upserts,
  idempotent bei zweitem Refresh.
- **Suche**: Integrationstest je DB-Backend (SQLite in-memory; Postgres über die
  bestehende CI-Matrix): indexieren → suchen → Ranking/Highlight prüfen.
- **Whisper-Client**: Mock-Server; Erfolg, Fehler, Retry, Zurücksetzen von
  `running`-Jobs beim Start.
- **E2E-Smoke**: Episode mit Transkript-Feed → Download → Segmente vorhanden →
  Suche findet sie mit Zeitstempel.

## Bewusst NICHT im ersten Wurf

Transkript-Editing, Diarisierungs-Nachbearbeitung, Übersetzungen, Einbetten der
Transkripte in Audiodateien, Whisper-Parallelität > 1, MySQL-Migrationen.
