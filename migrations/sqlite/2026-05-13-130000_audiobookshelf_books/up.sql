CREATE TABLE audiobookshelf_books (
    id TEXT NOT NULL PRIMARY KEY,
    library_id TEXT NOT NULL,
    title TEXT NOT NULL,
    subtitle TEXT,
    description TEXT,
    publisher TEXT,
    published_year TEXT,
    published_date TEXT,
    isbn TEXT,
    asin TEXT,
    language TEXT,
    explicit BOOLEAN NOT NULL DEFAULT 0,
    cover_path TEXT,
    duration_seconds REAL NOT NULL DEFAULT 0,
    ino TEXT,
    folder_path TEXT NOT NULL,
    last_scan TIMESTAMP,
    added_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

CREATE INDEX audiobookshelf_books_library_idx ON audiobookshelf_books (library_id);
CREATE UNIQUE INDEX audiobookshelf_books_folder_idx ON audiobookshelf_books (folder_path);

CREATE TABLE audiobookshelf_authors (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    asin TEXT,
    description TEXT,
    image_path TEXT
);

CREATE TABLE audiobookshelf_book_authors (
    book_id TEXT NOT NULL,
    author_id TEXT NOT NULL,
    PRIMARY KEY (book_id, author_id)
);
CREATE INDEX audiobookshelf_book_authors_book_idx ON audiobookshelf_book_authors (book_id);

CREATE TABLE audiobookshelf_narrators (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE audiobookshelf_book_narrators (
    book_id TEXT NOT NULL,
    narrator_id TEXT NOT NULL,
    PRIMARY KEY (book_id, narrator_id)
);
CREATE INDEX audiobookshelf_book_narrators_book_idx ON audiobookshelf_book_narrators (book_id);

CREATE TABLE audiobookshelf_series (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT
);

CREATE TABLE audiobookshelf_book_series (
    book_id TEXT NOT NULL,
    series_id TEXT NOT NULL,
    sequence TEXT,
    PRIMARY KEY (book_id, series_id)
);
CREATE INDEX audiobookshelf_book_series_book_idx ON audiobookshelf_book_series (book_id);

CREATE TABLE audiobookshelf_book_audio_files (
    id TEXT NOT NULL PRIMARY KEY,
    book_id TEXT NOT NULL,
    idx INTEGER NOT NULL,
    ino TEXT,
    path TEXT NOT NULL,
    relative_path TEXT NOT NULL,
    ext TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    duration REAL NOT NULL DEFAULT 0,
    bitrate INTEGER NOT NULL DEFAULT 0,
    codec TEXT NOT NULL,
    channels INTEGER NOT NULL DEFAULT 0,
    sample_rate INTEGER NOT NULL DEFAULT 0,
    track_num INTEGER,
    disc_num INTEGER,
    embedded_cover_path TEXT
);
CREATE INDEX audiobookshelf_book_audio_files_book_idx
    ON audiobookshelf_book_audio_files (book_id);

CREATE TABLE audiobookshelf_book_chapters (
    id TEXT NOT NULL PRIMARY KEY,
    book_id TEXT NOT NULL,
    idx INTEGER NOT NULL,
    start_time REAL NOT NULL,
    end_time REAL NOT NULL,
    title TEXT NOT NULL
);
CREATE INDEX audiobookshelf_book_chapters_book_idx
    ON audiobookshelf_book_chapters (book_id);
