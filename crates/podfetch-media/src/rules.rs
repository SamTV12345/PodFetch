//! Decisions the download pipeline makes from settings and file facts.

/// Returns the whole-second duration to store when the probed duration is
/// valid and differs from the stored `total_time`.
pub fn duration_to_store(current_total_time: i32, probed_seconds: f64) -> Option<i32> {
    if !probed_seconds.is_finite() || probed_seconds <= 0.0 {
        return None;
    }
    let rounded = probed_seconds.round();
    if rounded < 1.0 || rounded > f64::from(i32::MAX) {
        return None;
    }
    let rounded = rounded as i32;
    (rounded != current_total_time).then_some(rounded)
}

/// Combine the per-podcast and instance-wide `episode_numbering` flags.
/// Numbering applies when either layer turns it on; a missing settings row
/// counts as "off" for that layer.
pub fn numbering_from(per_podcast: Option<bool>, global: Option<bool>) -> bool {
    per_podcast.unwrap_or(false) || global.unwrap_or(false)
}

/// Resolve the effective `use_one_cover_for_all_episodes`. The per-podcast
/// value is honoured only when its settings row is activated; otherwise the
/// instance-wide default applies. This mirrors `resolve_nfo_format` /
/// `resolve_cover_filename` so every per-podcast download option shares the
/// same `activated`-gated override rule. `per_podcast` is `(activated, value)`.
pub fn use_one_cover_from(per_podcast: Option<(bool, bool)>, global: bool) -> bool {
    match per_podcast {
        Some((activated, value)) if activated => value,
        _ => global,
    }
}

/// Whether an image should be downloaded for this episode.
///
/// Always `false` when `use_one_cover` is forced. Otherwise downloads
/// when the episode carries its own `<itunes:image>` or the podcast
/// main image hasn't been fetched yet.
pub fn should_download_image(
    use_one_cover: bool,
    episode_has_own_image: bool,
    podcast_main_image_downloaded: bool,
) -> bool {
    !use_one_cover && (episode_has_own_image || !podcast_main_image_downloaded)
}

/// Decide the title to embed and whether to (re)write the
/// `episode_numbering_processed` flag. Returns `None` when numbering is on
/// and the episode was already processed (leave the existing title alone).
pub fn resolve_episode_title(
    name: &str,
    episode_numbering_processed: bool,
    numbering_enabled: bool,
    index: usize,
) -> Option<(String, bool)> {
    if numbering_enabled {
        if episode_numbering_processed {
            None
        } else {
            Some((format!("{index} - {name}"), true))
        }
    } else {
        Some((name.to_string(), false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_to_store_uses_probed_value_when_it_differs() {
        assert_eq!(duration_to_store(1397, 1593.6), Some(1594));
        assert_eq!(duration_to_store(0, 90.2), Some(90));
        assert_eq!(duration_to_store(1594, 1593.7), None);
        assert_eq!(duration_to_store(1397, 0.0), None);
        assert_eq!(duration_to_store(1397, -5.0), None);
        assert_eq!(duration_to_store(1397, f64::NAN), None);
        assert_eq!(duration_to_store(1397, f64::INFINITY), None);
    }

    #[test]
    fn resolve_episode_title_handles_all_numbering_states() {
        // numbering on, not yet processed -> prefix + mark processed
        assert_eq!(
            resolve_episode_title("Ep", false, true, 5),
            Some(("5 - Ep".to_string(), true))
        );
        // numbering on, already processed -> leave as-is
        assert_eq!(resolve_episode_title("Ep", true, true, 5), None);
        // numbering off -> plain title, mark not-processed
        assert_eq!(
            resolve_episode_title("Ep", false, false, 5),
            Some(("Ep".to_string(), false))
        );
        assert_eq!(
            resolve_episode_title("Ep", true, false, 5),
            Some(("Ep".to_string(), false))
        );
    }

    #[test]
    fn numbering_from_ors_per_podcast_and_global_default() {
        // global default forces numbering even with no per-podcast row or an explicit off
        assert!(numbering_from(None, Some(true)));
        assert!(numbering_from(Some(false), Some(true)));
        // per-podcast enables it regardless of the (off/absent) global default
        assert!(numbering_from(Some(true), Some(false)));
        assert!(numbering_from(Some(true), None));
        // both layers off / absent -> disabled
        assert!(!numbering_from(Some(false), Some(false)));
        assert!(!numbering_from(None, None));
    }

    #[test]
    fn use_one_cover_from_respects_activated_gate() {
        // no per-podcast row -> instance-wide default applies
        assert!(use_one_cover_from(None, true));
        assert!(!use_one_cover_from(None, false));
        // per-podcast row but NOT activated -> still falls back to the global
        // default (mirrors NFO/cover resolution; the activated flag is the
        // explicit-override switch).
        assert!(use_one_cover_from(Some((false, false)), true));
        assert!(!use_one_cover_from(Some((false, true)), false));
        // per-podcast row activated -> the per-podcast value wins
        assert!(use_one_cover_from(Some((true, true)), false));
        assert!(!use_one_cover_from(Some((true, false)), true));
    }

    #[test]
    fn decision_matrix_all_combinations() {
        struct Case {
            use_one_cover: bool,
            episode_own: bool,
            main_downloaded: bool,
            expected: bool,
        }
        let cases = [
            // use_one_cover = true → never download
            Case {
                use_one_cover: true,
                episode_own: true,
                main_downloaded: true,
                expected: false,
            },
            Case {
                use_one_cover: true,
                episode_own: true,
                main_downloaded: false,
                expected: false,
            },
            Case {
                use_one_cover: true,
                episode_own: false,
                main_downloaded: true,
                expected: false,
            },
            Case {
                use_one_cover: true,
                episode_own: false,
                main_downloaded: false,
                expected: false,
            },
            // use_one_cover = false → download when episode has own image OR main not yet downloaded
            Case {
                use_one_cover: false,
                episode_own: true,
                main_downloaded: true,
                expected: true,
            },
            Case {
                use_one_cover: false,
                episode_own: true,
                main_downloaded: false,
                expected: true,
            },
            Case {
                use_one_cover: false,
                episode_own: false,
                main_downloaded: true,
                expected: false,
            },
            Case {
                use_one_cover: false,
                episode_own: false,
                main_downloaded: false,
                expected: true,
            },
        ];
        for c in &cases {
            let got = should_download_image(c.use_one_cover, c.episode_own, c.main_downloaded);
            assert_eq!(
                got, c.expected,
                "cover={} own={} main_downloaded={} → expected {} got {}",
                c.use_one_cover, c.episode_own, c.main_downloaded, c.expected, got,
            );
        }
    }
}
