//! Additional metadata sources beyond ID3 tags + folder names.
//!
//! Each module exposes a `load(folder: &Path) -> Option<MetadataPatch>` that
//! reads its specific file shape and produces a partial patch which the
//! `metadata_resolver` then applies in upstream's precedence order:
//!
//!   folderStructure -> audioMetatags -> nfoFile -> txtFiles -> opfFile -> absMetadata
//!
//! Each later source overrides earlier values when defined.

pub mod abs;
pub mod nfo;
pub mod opf;
pub mod txt;

/// Patch applied on top of `ResolvedBookMetadata` by a metadata source.
/// `None` fields are left untouched; `Some` fields overwrite.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MetadataPatch {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub published_year: Option<String>,
    pub published_date: Option<String>,
    pub isbn: Option<String>,
    pub asin: Option<String>,
    pub language: Option<String>,
    pub explicit: Option<bool>,
    pub authors: Option<Vec<String>>,
    pub narrators: Option<Vec<String>>,
    pub series: Option<(String, Option<String>)>,
}
