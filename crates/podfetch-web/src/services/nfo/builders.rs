//! Pure NFO XML builders. Domain types in, XML string out. No DB, no FS.

use podfetch_domain::podcast::Podcast;
use podfetch_domain::podcast_episode::PodcastEpisode;
use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use std::io::Cursor;

type XmlWriter = Writer<Cursor<Vec<u8>>>;

fn new_writer() -> XmlWriter {
    Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2)
}

fn finish(writer: XmlWriter) -> String {
    String::from_utf8(writer.into_inner().into_inner()).expect("xml is valid utf-8")
}

fn write_decl(w: &mut XmlWriter) {
    w.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), Some("yes"))))
        .expect("write decl");
}

/// Write `<name>text</name>`. No-op when `text` is None or empty. `quick_xml`
/// escapes the text content (`&`, `<`, `>`, quotes) automatically.
fn write_text_el(w: &mut XmlWriter, name: &str, text: Option<&str>) {
    let Some(text) = text.filter(|t| !t.is_empty()) else {
        return;
    };
    w.write_event(Event::Start(BytesStart::new(name)))
        .expect("start element");
    w.write_event(Event::Text(BytesText::new(text)))
        .expect("text");
    w.write_event(Event::End(BytesEnd::new(name)))
        .expect("end element");
}

/// Write `<uniqueid type="podfetch">id</uniqueid>` when `id` is non-empty.
fn write_uniqueid(w: &mut XmlWriter, id: Option<&str>) {
    let Some(id) = id.filter(|s| !s.is_empty()) else {
        return;
    };
    let mut el = BytesStart::new("uniqueid");
    el.push_attribute(("type", "podfetch"));
    w.write_event(Event::Start(el)).expect("start uniqueid");
    w.write_event(Event::Text(BytesText::new(id)))
        .expect("uniqueid text");
    w.write_event(Event::End(BytesEnd::new("uniqueid")))
        .expect("end uniqueid");
}

/// Kodi `<runtime>` is expressed in whole minutes.
fn runtime_minutes(total_time_secs: i32) -> i64 {
    ((total_time_secs.max(0) as f64) / 60.0).round() as i64
}

/// `date_of_recording` is typically ISO-8601 ("2023-09-07T13:09:00"). Take the
/// `YYYY-MM-DD` prefix (dates are ASCII so byte slicing is char-safe).
fn aired_date(date_of_recording: &str) -> Option<String> {
    date_of_recording.get(..10).map(str::to_string)
}

/// `tvshow.nfo` at the podcast root.
pub fn build_tvshow_nfo(podcast: &Podcast) -> String {
    let mut w = new_writer();
    write_decl(&mut w);
    w.write_event(Event::Start(BytesStart::new("tvshow")))
        .expect("start tvshow");
    write_text_el(&mut w, "title", Some(&podcast.name));
    write_text_el(&mut w, "plot", podcast.summary.as_deref());
    write_text_el(&mut w, "studio", podcast.author.as_deref());
    write_text_el(&mut w, "genre", podcast.keywords.as_deref());
    write_uniqueid(&mut w, podcast.guid.as_deref());
    w.write_event(Event::End(BytesEnd::new("tvshow")))
        .expect("end tvshow");
    finish(w)
}

/// Per-episode `<basename>.nfo`.
pub fn build_episodedetails_nfo(
    podcast: &Podcast,
    episode: &PodcastEpisode,
    position: i64,
) -> String {
    let mut w = new_writer();
    write_decl(&mut w);
    w.write_event(Event::Start(BytesStart::new("episodedetails")))
        .expect("start episodedetails");
    write_text_el(&mut w, "title", Some(&episode.name));
    write_text_el(&mut w, "showtitle", Some(&podcast.name));
    write_text_el(&mut w, "season", Some("1"));
    write_text_el(&mut w, "episode", Some(&position.to_string()));
    write_text_el(&mut w, "plot", Some(&episode.description));
    write_text_el(&mut w, "aired", aired_date(&episode.date_of_recording).as_deref());
    if episode.total_time > 0 {
        write_text_el(&mut w, "runtime", Some(&runtime_minutes(episode.total_time).to_string()));
    }
    if let Some(author) = podcast.author.as_deref().filter(|a| !a.is_empty()) {
        w.write_event(Event::Start(BytesStart::new("actor")))
            .expect("start actor");
        write_text_el(&mut w, "name", Some(author));
        w.write_event(Event::End(BytesEnd::new("actor")))
            .expect("end actor");
    }
    write_uniqueid(&mut w, Some(&episode.guid));
    w.write_event(Event::End(BytesEnd::new("episodedetails")))
        .expect("end episodedetails");
    finish(w)
}

/// Single `album.nfo` at the podcast root. `tracks` are `(episode, position)`
/// ordered by date.
pub fn build_album_nfo(podcast: &Podcast, tracks: &[(PodcastEpisode, i64)]) -> String {
    let mut w = new_writer();
    write_decl(&mut w);
    w.write_event(Event::Start(BytesStart::new("album")))
        .expect("start album");
    write_text_el(&mut w, "title", Some(&podcast.name));
    write_text_el(&mut w, "artist", podcast.author.as_deref());
    write_text_el(&mut w, "genre", podcast.keywords.as_deref());
    write_text_el(&mut w, "review", podcast.summary.as_deref());
    for (episode, position) in tracks {
        w.write_event(Event::Start(BytesStart::new("track")))
            .expect("start track");
        write_text_el(&mut w, "position", Some(&position.to_string()));
        write_text_el(&mut w, "title", Some(&episode.name));
        write_text_el(&mut w, "duration", Some(&episode.total_time.to_string()));
        w.write_event(Event::End(BytesEnd::new("track")))
            .expect("end track");
    }
    w.write_event(Event::End(BytesEnd::new("album")))
        .expect("end album");
    finish(w)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn podcast() -> Podcast {
        Podcast {
            name: "My Podcast".to_string(),
            summary: Some("A show about things & stuff".to_string()),
            author: Some("Jane Host".to_string()),
            keywords: Some("tech, news".to_string()),
            guid: Some("podcast-guid-1".to_string()),
            ..Default::default()
        }
    }

    fn episode() -> PodcastEpisode {
        PodcastEpisode {
            name: "Episode <One>".to_string(),
            description: "Plot & details".to_string(),
            date_of_recording: "2023-09-07T13:09:00".to_string(),
            total_time: 2520, // 42 minutes
            guid: "episode-guid-1".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn tvshow_has_expected_fields_and_escapes() {
        let xml = build_tvshow_nfo(&podcast());
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>"));
        assert!(xml.contains("<title>My Podcast</title>"));
        assert!(xml.contains("<plot>A show about things &amp; stuff</plot>"));
        assert!(xml.contains("<studio>Jane Host</studio>"));
        assert!(xml.contains("<genre>tech, news</genre>"));
        assert!(xml.contains("<uniqueid type=\"podfetch\">podcast-guid-1</uniqueid>"));
    }

    #[test]
    fn tvshow_omits_absent_optional_fields() {
        let p = Podcast {
            name: "Bare".to_string(),
            ..Default::default()
        };
        let xml = build_tvshow_nfo(&p);
        assert!(xml.contains("<title>Bare</title>"));
        assert!(!xml.contains("<studio>"));
        assert!(!xml.contains("<plot>"));
        assert!(!xml.contains("<genre>"));
        assert!(!xml.contains("<uniqueid"));
    }

    #[test]
    fn episodedetails_maps_runtime_minutes_position_and_actor() {
        let xml = build_episodedetails_nfo(&podcast(), &episode(), 7);
        assert!(xml.contains("<title>Episode &lt;One&gt;</title>"));
        assert!(xml.contains("<showtitle>My Podcast</showtitle>"));
        assert!(xml.contains("<season>1</season>"));
        assert!(xml.contains("<episode>7</episode>"));
        assert!(xml.contains("<plot>Plot &amp; details</plot>"));
        assert!(xml.contains("<aired>2023-09-07</aired>"));
        assert!(xml.contains("<runtime>42</runtime>"));
        assert!(xml.contains("<actor>"));
        assert!(xml.contains("<name>Jane Host</name>"));
        assert!(xml.contains("<uniqueid type=\"podfetch\">episode-guid-1</uniqueid>"));
    }

    #[test]
    fn episodedetails_omits_actor_without_author() {
        let p = Podcast {
            name: "P".to_string(),
            ..Default::default()
        };
        let xml = build_episodedetails_nfo(&p, &episode(), 1);
        assert!(!xml.contains("<actor>"));
    }

    #[test]
    fn episodedetails_omits_runtime_when_duration_unknown() {
        let mut e = episode();
        e.total_time = 0;
        let xml = build_episodedetails_nfo(&podcast(), &e, 1);
        assert!(!xml.contains("<runtime>"));
    }

    #[test]
    fn album_omits_absent_optional_fields() {
        let p = Podcast {
            name: "Bare Album".to_string(),
            ..Default::default()
        };
        let tracks = vec![(episode(), 1)];
        let xml = build_album_nfo(&p, &tracks);
        assert!(xml.contains("<title>Bare Album</title>"));
        assert!(!xml.contains("<artist>"));
        assert!(!xml.contains("<genre>"));
        assert!(!xml.contains("<review>"));
        // tracks still present
        assert!(xml.contains("<position>1</position>"));
    }

    #[test]
    fn album_lists_tracks_in_order() {
        let mut e2 = episode();
        e2.name = "Episode Two".to_string();
        e2.total_time = 60;
        let tracks = vec![(episode(), 1), (e2, 2)];
        let xml = build_album_nfo(&podcast(), &tracks);
        assert!(xml.contains("<artist>Jane Host</artist>"));
        assert!(xml.contains("<review>A show about things &amp; stuff</review>"));
        let p1 = xml.find("<position>1</position>").expect("track 1");
        let p2 = xml.find("<position>2</position>").expect("track 2");
        assert!(p1 < p2, "tracks must be ordered");
        assert!(xml.contains("<duration>2520</duration>"));
        assert!(xml.contains("<duration>60</duration>"));
    }
}
