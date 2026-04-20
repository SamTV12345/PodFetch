//! Static mapping between iTunes category names (as they appear in
//! `<itunes:categories>` inside an RSS feed and therefore in the
//! `podcasts.keywords` column), iTunes genre IDs (used by the Apple RSS
//! top-podcasts feed) and Podcastindex.org category IDs (used by the
//! `/podcasts/trending` endpoint).
//!
//! The three taxonomies are fixed and public, so we can hard-code the mapping
//! instead of performing a lookup at request time.

#[derive(Debug, Clone, Copy)]
pub struct CategoryMapping {
    pub name: &'static str,
    pub itunes_genre_id: u32,
    pub podindex_id: u32,
}

const fn cat(name: &'static str, itunes_genre_id: u32, podindex_id: u32) -> CategoryMapping {
    CategoryMapping {
        name,
        itunes_genre_id,
        podindex_id,
    }
}

/// Top-level categories shared across iTunes and Podcastindex.
/// Names match iTunes / Apple Podcasts categories, which is also the taxonomy
/// used by RSS `<itunes:categories>` tags.
pub const CATEGORY_MAP: &[CategoryMapping] = &[
    cat("Arts", 1301, 1),
    cat("Books", 1301, 3),
    cat("Design", 1301, 4),
    cat("Fashion & Beauty", 1301, 5),
    cat("Food", 1301, 6),
    cat("Performing Arts", 1301, 7),
    cat("Visual Arts", 1301, 8),
    cat("Business", 1321, 9),
    cat("Careers", 1321, 10),
    cat("Entrepreneurship", 1321, 11),
    cat("Investing", 1321, 12),
    cat("Management", 1321, 13),
    cat("Marketing", 1321, 14),
    cat("Non-Profit", 1321, 15),
    cat("Comedy", 1303, 16),
    cat("Comedy Interviews", 1303, 17),
    cat("Improv", 1303, 18),
    cat("Stand-Up", 1303, 19),
    cat("Education", 1304, 20),
    cat("Courses", 1304, 21),
    cat("How To", 1304, 22),
    cat("Language Learning", 1304, 23),
    cat("Self-Improvement", 1304, 24),
    cat("Fiction", 1483, 25),
    cat("Comedy Fiction", 1483, 26),
    cat("Drama", 1483, 27),
    cat("Science Fiction", 1483, 28),
    cat("Health & Fitness", 1512, 29),
    cat("Health", 1512, 29),
    cat("Alternative Health", 1512, 30),
    cat("Fitness", 1512, 31),
    cat("Medicine", 1512, 32),
    cat("Mental Health", 1512, 33),
    cat("Nutrition", 1512, 34),
    cat("Sexuality", 1512, 35),
    cat("Kids & Family", 1305, 36),
    cat("Education for Kids", 1305, 37),
    cat("Parenting", 1305, 38),
    cat("Pets & Animals", 1305, 39),
    cat("Stories for Kids", 1305, 40),
    cat("Leisure", 1502, 41),
    cat("Animation & Manga", 1502, 42),
    cat("Automotive", 1502, 43),
    cat("Aviation", 1502, 44),
    cat("Crafts", 1502, 45),
    cat("Games", 1502, 46),
    cat("Hobbies", 1502, 47),
    cat("Home & Garden", 1502, 48),
    cat("Video Games", 1502, 49),
    cat("Music", 1310, 50),
    cat("Music Commentary", 1310, 51),
    cat("Music History", 1310, 52),
    cat("Music Interviews", 1310, 53),
    cat("News", 1489, 54),
    cat("Business News", 1489, 55),
    cat("Daily News", 1489, 56),
    cat("Entertainment News", 1489, 57),
    cat("News Commentary", 1489, 58),
    cat("Politics", 1489, 59),
    cat("Sports News", 1489, 60),
    cat("Tech News", 1489, 61),
    cat("Religion & Spirituality", 1314, 62),
    cat("Buddhism", 1314, 63),
    cat("Christianity", 1314, 64),
    cat("Hinduism", 1314, 65),
    cat("Islam", 1314, 66),
    cat("Judaism", 1314, 67),
    cat("Religion", 1314, 68),
    cat("Spirituality", 1314, 69),
    cat("Science", 1533, 70),
    cat("Astronomy", 1533, 71),
    cat("Chemistry", 1533, 72),
    cat("Earth Sciences", 1533, 73),
    cat("Life Sciences", 1533, 74),
    cat("Mathematics", 1533, 75),
    cat("Natural Sciences", 1533, 76),
    cat("Nature", 1533, 77),
    cat("Physics", 1533, 78),
    cat("Social Sciences", 1533, 79),
    cat("Society & Culture", 1324, 80),
    cat("Documentary", 1324, 81),
    cat("Personal Journals", 1324, 82),
    cat("Philosophy", 1324, 83),
    cat("Places & Travel", 1324, 84),
    cat("Relationships", 1324, 85),
    cat("Sports", 1545, 86),
    cat("Baseball", 1545, 87),
    cat("Basketball", 1545, 88),
    cat("Cricket", 1545, 89),
    cat("Fantasy Sports", 1545, 90),
    cat("Football", 1545, 91),
    cat("Golf", 1545, 92),
    cat("Hockey", 1545, 93),
    cat("Rugby", 1545, 94),
    cat("Running", 1545, 95),
    cat("Soccer", 1545, 96),
    cat("Swimming", 1545, 97),
    cat("Tennis", 1545, 98),
    cat("Volleyball", 1545, 99),
    cat("Wilderness", 1545, 100),
    cat("Wrestling", 1545, 101),
    cat("Technology", 1318, 102),
    cat("True Crime", 1488, 103),
    cat("TV & Film", 1309, 104),
    cat("After Shows", 1309, 105),
    cat("Film History", 1309, 106),
    cat("Film Interviews", 1309, 107),
    cat("Film Reviews", 1309, 108),
    cat("TV Reviews", 1309, 109),
    cat("History", 1487, 125),
    cat("Government", 1324, 110),
];

fn find_by_name(name: &str) -> Option<&'static CategoryMapping> {
    let needle = name.trim();
    CATEGORY_MAP
        .iter()
        .find(|c| c.name.eq_ignore_ascii_case(needle))
}

/// Map a list of iTunes category names to Podcastindex category IDs.
///
/// Unknown names are silently skipped — that is the desired behavior for RSS
/// feeds that declare non-standard text in their `<itunes:categories>` tag.
pub fn names_to_podindex_ids(names: &[&str]) -> Vec<u32> {
    names
        .iter()
        .filter_map(|n| find_by_name(n).map(|m| m.podindex_id))
        .collect()
}

/// Map an iTunes category name to an iTunes genre ID.
pub fn name_to_itunes_genre(name: &str) -> Option<u32> {
    find_by_name(name).map(|m| m.itunes_genre_id)
}

/// Return all known category names for the UI category picker.
pub fn all_category_names() -> Vec<&'static str> {
    CATEGORY_MAP.iter().map(|c| c.name).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_known_categories_to_podindex_ids() {
        let ids = names_to_podindex_ids(&["News", "Business"]);
        assert_eq!(ids, vec![54, 9]);
    }

    #[test]
    fn unknown_category_is_skipped() {
        let ids = names_to_podindex_ids(&["News", "Nonsense", "Technology"]);
        assert_eq!(ids, vec![54, 102]);
    }

    #[test]
    fn empty_input_yields_empty_output() {
        assert!(names_to_podindex_ids(&[]).is_empty());
    }

    #[test]
    fn itunes_lookup_is_case_insensitive() {
        assert_eq!(name_to_itunes_genre("news"), Some(1489));
        assert_eq!(name_to_itunes_genre("  News "), Some(1489));
    }
}
