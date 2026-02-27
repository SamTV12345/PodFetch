const SHELL_CACHE = 'podfetch-shell-v1';
const RUNTIME_CACHE = 'podfetch-runtime-v1';
const SHELL_FILES = [
  '/ui/',
  '/ui/index.html',
  '/ui/manifest.webmanifest',
  '/ui/pwa-192x192.png',
  '/ui/pwa-512x512.png',
  '/ui/apple-touch-icon-180x180.png',
  '/ui/favicon.ico'
];

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(SHELL_CACHE).then((cache) => cache.addAll(SHELL_FILES)).then(() => self.skipWaiting())
  );
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((keys) =>
      Promise.all(
        keys
          .filter((key) => key !== SHELL_CACHE && key !== RUNTIME_CACHE)
          .map((key) => caches.delete(key))
      )
    ).then(() => self.clients.claim())
  );
});

self.addEventListener('fetch', (event) => {
  const request = event.request;
  if (request.method !== 'GET') {
    return;
  }
  if (request.headers.has('range')) {
    return;
  }

  const url = new URL(request.url);
  if (url.origin !== self.location.origin) {
    return;
  }
  if (!url.pathname.startsWith('/ui/')) {
    return;
  }

  const destination = request.destination;
  const shouldCache =
    destination === 'document' ||
    destination === 'script' ||
    destination === 'style' ||
    destination === 'image' ||
    destination === 'font';

  if (!shouldCache) {
    return;
  }

  event.respondWith(
    caches.match(request).then((cachedResponse) => {
      const networkFetch = fetch(request)
        .then((networkResponse) => {
          if (!networkResponse || networkResponse.status !== 200 || networkResponse.type === 'opaque') {
            return networkResponse;
          }

          const responseClone = networkResponse.clone();
          caches.open(RUNTIME_CACHE).then((cache) => cache.put(request, responseClone));
          return networkResponse;
        })
        .catch(() => cachedResponse);

      return cachedResponse || networkFetch;
    })
  );
});
