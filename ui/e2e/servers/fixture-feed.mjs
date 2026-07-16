// Static fixture server for the e2e suite. Serves two podcast feeds:
//  - /feed.xml   "Transcript E2E Podcast" — episode 1 with a
//                <podcast:transcript> VTT tag (incl. an XSS probe cue),
//                episode 2 without a transcript; more episodes can be
//                published at runtime via POST /control/publish.
//  - /feed2.xml  "Second Fixture Podcast" — one plain episode, used by the
//                add/delete-podcast UI tests.
// Loopback only.
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

00:00:15.000 --> 00:00:20.000
Dangerous <script>window.__xss=1</script> content with xanadu99 marker
`

// Minimal MP3: ID3v2 header followed by silent MPEG frames. The browser DOES
// decode this for the click-to-seek test, so it needs enough frames (~26ms
// each) to cover the transcript's 20s — otherwise currentTime clamps to the
// audio duration and seeking to a segment start can never reach it.
const id3 = Buffer.concat([Buffer.from('ID3'), Buffer.from([3, 0, 0, 0, 0, 0, 10]), Buffer.alloc(10)])
const frame = Buffer.concat([Buffer.from([0xff, 0xfb, 0x90, 0x00]), Buffer.alloc(413)])
const EPISODE_MP3 = Buffer.concat([id3, ...Array(900).fill(frame)])

const item = ({ title, guid, slug, pubDate, transcript }) => `    <item>
      <title>${title}</title>
      <guid isPermaLink="false">${guid}</guid>
      <pubDate>${pubDate}</pubDate>
      <description>Fixture episode ${title}</description>
      <enclosure url="${BASE}/audio/${slug}.mp3" length="${EPISODE_MP3.length}" type="audio/mpeg"/>
      <itunes:duration>20</itunes:duration>
${transcript ? `      <podcast:transcript url="${BASE}/transcript.vtt" type="text/vtt" language="en"/>\n` : ''}    </item>`

const feed = (title, items) => `<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd" xmlns:podcast="https://podcastindex.org/namespace/1.0">
  <channel>
    <title>${title}</title>
    <link>${BASE}/</link>
    <description>Fixture feed for the playwright suite</description>
    <language>en</language>
${items.join('\n')}
  </channel>
</rss>
`

const feed1Items = [
    item({
        title: 'Smoke Episode 1',
        guid: 'smoke-episode-1',
        slug: 'smoke-episode-1',
        pubDate: 'Wed, 15 Jul 2026 10:00:00 +0000',
        transcript: true,
    }),
    item({
        title: 'Plain Episode 2',
        guid: 'plain-episode-2',
        slug: 'plain-episode-2',
        pubDate: 'Tue, 14 Jul 2026 10:00:00 +0000',
        transcript: false,
    }),
]
let publishedCounter = 0

const feed2Items = [
    item({
        title: 'Second Podcast Episode',
        guid: 'second-podcast-episode-1',
        slug: 'second-podcast-episode-1',
        pubDate: 'Mon, 13 Jul 2026 10:00:00 +0000',
        transcript: false,
    }),
]

http.createServer((req, res) => {
    const url = new URL(req.url, BASE)

    if (req.method === 'POST' && url.pathname === '/control/publish') {
        let body = ''
        req.on('data', chunk => (body += chunk))
        req.on('end', () => {
            const { title } = JSON.parse(body || '{}')
            publishedCounter += 1
            const slug = `published-${publishedCounter}`
            feed1Items.unshift(
                item({
                    title: title ?? `Published Episode ${publishedCounter}`,
                    guid: slug,
                    slug,
                    pubDate: new Date().toUTCString(),
                    transcript: false,
                })
            )
            res.writeHead(200, { 'Content-Type': 'application/json' })
            res.end(JSON.stringify({ slug }))
        })
        return
    }

    const respond = (body, type) => {
        res.writeHead(200, { 'Content-Type': type, 'Content-Length': Buffer.byteLength(body) })
        res.end(body)
    }

    if (url.pathname === '/feed.xml') {
        return respond(feed('Transcript E2E Podcast', feed1Items), 'application/rss+xml')
    }
    if (url.pathname === '/feed2.xml') {
        return respond(feed('Second Fixture Podcast', feed2Items), 'application/rss+xml')
    }
    if (url.pathname === '/transcript.vtt') {
        return respond(TRANSCRIPT_VTT, 'text/vtt')
    }
    if (url.pathname.startsWith('/audio/') && url.pathname.endsWith('.mp3')) {
        return respond(EPISODE_MP3, 'audio/mpeg')
    }

    res.writeHead(404)
    res.end('not found')
}).listen(PORT, HOST, () => console.log(`fixture feed listening on ${BASE}`))
