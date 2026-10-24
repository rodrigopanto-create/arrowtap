"use strict";

// Bump the version whenever any cached game or website asset changes.
const CACHE_PREFIX = `tap-away-arcade-${self.registration.scope}-`;
const CACHE_NAME = `${CACHE_PREFIX}v1`;
const ASSETS = [
    "./",
    "./index.html",
    "./gl.js",
    "./highscores.js",
    "./pwa.js",
    "./manifest.webmanifest",
    "./icons/icon-192.png",
    "./icons/icon-512.png",
    "./icons/maskable-512.png",
    "./icons/apple-touch-icon.png",
    "./target/wasm32-unknown-unknown/release/arrow.wasm"
];

self.addEventListener("install", (event) => {
    event.waitUntil(caches.open(CACHE_NAME).then((cache) => cache.addAll(ASSETS)));
});

self.addEventListener("activate", (event) => {
    event.waitUntil(
        caches.keys().then((keys) => Promise.all(
            keys.filter((key) => key.startsWith(CACHE_PREFIX) && key !== CACHE_NAME)
                .map((key) => caches.delete(key))
        )).then(() => self.clients.claim())
    );
});

self.addEventListener("fetch", (event) => {
    if (event.request.method !== "GET"
        || new URL(event.request.url).origin !== self.location.origin) {
        return;
    }
    const url = event.request.mode === "navigate"
        ? new URL("./", self.registration.scope).href
        : event.request;
    event.respondWith(caches.open(CACHE_NAME).then(async (cache) => {
        const cached = await cache.match(url);
        return cached || fetch(event.request);
    }));
});
