// Static fixture server for the transcript e2e suite: one podcast feed whose
// single episode carries a <podcast:transcript> VTT tag, plus the referenced
// audio and transcript files. Loopback only.
import http from 'node:http'

const PORT = 9123
const HOST = '127.0.0.1'
const BASE = `http://${HOST}:${PORT}`

const TRANSCRIPT_VTT = `WEBVTT

00:00:00.000 --> 00:00:04.000
<v Alice>Welcome to the transcript smoke test episode.

00:00:04.000 --> 00:00:09.500
We are talking about the zephyrquark keyword for full-text search.

00:00:09.500 --> 00:00:15.000
Thanks for listening and goodbye.
`

const FEED_XML = `<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd" xmlns:podcast="https://podcastindex.org/namespace/1.0">
  <channel>
    <title>Transcript E2E Podcast</title>
    <link>${BASE}/</link>
    <description>Fixture feed for the transcript playwright suite</description>
    <language>en</language>
    <item>
      <title>Smoke Episode 1</title>
      <guid isPermaLink="false">smoke-episode-1</guid>
      <pubDate>Wed, 15 Jul 2026 10:00:00 +0000</pubDate>
      <description>Episode with a podcasting 2.0 transcript</description>
      <enclosure url="${BASE}/episode.mp3" length="20940" type="audio/mpeg"/>
      <itunes:duration>15</itunes:duration>
      <podcast:transcript url="${BASE}/transcript.vtt" type="text/vtt" language="en"/>
    </item>
  </channel>
</rss>
`

// Minimal MP3: ID3v2 header followed by silent MPEG frames. The browser DOES
// decode this for the click-to-seek test, so it needs enough frames (~26ms
// each) to cover the transcript's 15s — otherwise currentTime clamps to the
// audio duration and seeking to a segment start can never reach it.
const id3 = Buffer.concat([Buffer.from('ID3'), Buffer.from([3, 0, 0, 0, 0, 0, 10]), Buffer.alloc(10)])
const frame = Buffer.concat([Buffer.from([0xff, 0xfb, 0x90, 0x00]), Buffer.alloc(413)])
const EPISODE_MP3 = Buffer.concat([id3, ...Array(700).fill(frame)])

const routes = {
    '/feed.xml': { body: FEED_XML, type: 'application/rss+xml' },
    '/transcript.vtt': { body: TRANSCRIPT_VTT, type: 'text/vtt' },
    '/episode.mp3': { body: EPISODE_MP3, type: 'audio/mpeg' },
}

http.createServer((req, res) => {
    const route = routes[new URL(req.url, BASE).pathname]
    if (!route) {
        res.writeHead(404)
        res.end('not found')
        return
    }
    res.writeHead(200, { 'Content-Type': route.type, 'Content-Length': Buffer.byteLength(route.body) })
    res.end(route.body)
}).listen(PORT, HOST, () => console.log(`fixture feed listening on ${BASE}`))
