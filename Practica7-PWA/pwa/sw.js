
// ═══════════════════════════════════════════════════════════════
//  SERVICE WORKER — Dungeon Crawler PWA
//  Estrategia: Stale-While-Revalidate para assets estáticos
// ═══════════════════════════════════════════════════════════════

const CACHE_NAME   = 'dungeon-crawler-v1';
const STATIC_CACHE = 'dungeon-static-v1';

// Assets que siempre se pre-cachean
const PRECACHE_ASSETS = [
  './',
  './index.html',
  './css/index.css',
  './js/game.js',
  './manifest.json',
  './docs/png/logo.png',
  './docs/png/logo/logo_180x180.png',
  './docs/png/logo/logo_192x192.png',
  './docs/png/logo/logo_512x512.png'
  // Fuentes de Google (se cachean en runtime)
];

// ── INSTALL: pre-cachear shell ───────────────────────────────────
self.addEventListener('install', (event) => {
  console.log('[SW] Instalando...');
  event.waitUntil(
    caches.open(STATIC_CACHE).then(cache => {
      console.log('[SW] Pre-cacheando assets estáticos');
      return cache.addAll(PRECACHE_ASSETS);
    })
  );
  self.skipWaiting();
});

// ── ACTIVATE: limpiar caches viejas ─────────────────────────────
self.addEventListener('activate', (event) => {
  console.log('[SW] Activando...');
  event.waitUntil(
    caches.keys().then(keys => {
      return Promise.all(
        keys
          .filter(k => k !== STATIC_CACHE && k !== CACHE_NAME)
          .map(k => {
            console.log('[SW] Eliminando cache obsoleta:', k);
            return caches.delete(k);
          })
      );
    })
  );
  self.clients.claim();
});

// ── FETCH: Stale-While-Revalidate ───────────────────────────────
self.addEventListener('fetch', (event) => {
  const { request } = event;
  const url = new URL(request.url);

  // No interceptar peticiones a la API del juego
  if (url.pathname.startsWith('/login') ||
      url.pathname.startsWith('/item')  ||
      url.pathname.startsWith('/level')) {
    return; // Pasa directo sin cache
  }

  // No interceptar peticiones cross-origin no de fuentes
  if (url.origin !== location.origin &&
      !url.hostname.includes('fonts.googleapis.com') &&
      !url.hostname.includes('fonts.gstatic.com')) {
    return;
  }

  // ── Stale-While-Revalidate ───────────────────────────────────
  event.respondWith(staleWhileRevalidate(request));
});

async function staleWhileRevalidate(request) {
  const cacheName = STATIC_CACHE;

  // 1. Buscar en cache (stale)
  const cachedResponse = await caches.match(request);

  // 2. Fetch en background para revalidar
  const fetchPromise = fetch(request).then(async networkResponse => {
    if (networkResponse && networkResponse.status === 200) {
      const cache = await caches.open(cacheName);
      // Clonar antes de guardar
      cache.put(request, networkResponse.clone());
    }
    return networkResponse;
  }).catch(() => null);

  // 3. Si tenemos cache → devolver inmediatamente (stale)
  //    mientras la red actualiza en background
  if (cachedResponse) {
    return cachedResponse;
  }

  // 4. Si no hay cache → esperar la red
  const networkResponse = await fetchPromise;
  if (networkResponse) {
    return networkResponse;
  }

  // 5. Offline fallback para navegación
  if (request.mode === 'navigate') {
    const fallback = await caches.match('./index.html');
    if (fallback) return fallback;
  }

  // 6. Sin opciones
  return new Response('Sin conexión', {
    status: 503,
    statusText: 'Service Unavailable',
    headers: { 'Content-Type': 'text/plain' },
  });
}