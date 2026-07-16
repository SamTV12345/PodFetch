// OpenAI-compatible mock transcription endpoint for the e2e suite. Responds
// after a short delay so the UI's pending state is observable. Tests can flip
// it into failure mode via POST /control/mode {"mode":"fail"} to exercise the
// job retry path. Loopback only.
import http from 'node:http'

const PORT = 9998
const HOST = '127.0.0.1'
const RESPONSE_DELAY_MS = 1200

let mode = 'ok'

const RESPONSE = JSON.stringify({
    language: 'en',
    segments: [
        { start: 0.0, end: 4.0, text: ' Generated hello from the mock whisper server.' },
        { start: 4.0, end: 9.0, text: ' The magic word is quixotron for search.' },
    ],
})

http.createServer((req, res) => {
    if (req.method === 'GET' && req.url === '/health') {
        res.writeHead(200)
        res.end('ok')
        return
    }
    if (req.method === 'POST' && req.url === '/control/mode') {
        let body = ''
        req.on('data', chunk => (body += chunk))
        req.on('end', () => {
            mode = JSON.parse(body || '{}').mode === 'fail' ? 'fail' : 'ok'
            res.writeHead(200)
            res.end(mode)
        })
        return
    }
    if (req.method === 'POST' && req.url === '/v1/audio/transcriptions') {
        req.resume() // drain the multipart upload
        req.on('end', () => {
            setTimeout(() => {
                if (mode === 'fail') {
                    res.writeHead(500, { 'Content-Type': 'application/json' })
                    res.end(JSON.stringify({ error: 'mock whisper is in failure mode' }))
                    return
                }
                res.writeHead(200, { 'Content-Type': 'application/json' })
                res.end(RESPONSE)
            }, RESPONSE_DELAY_MS)
        })
        return
    }
    res.writeHead(404)
    res.end('not found')
}).listen(PORT, HOST, () => console.log(`mock whisper listening on http://${HOST}:${PORT}`))
