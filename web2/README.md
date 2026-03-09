# Dalang Web2 (SvelteKit App)

This directory contains Dalang's SvelteKit frontend application.

It is built as an app (not a package) and outputs static assets to `web2/build/`.
Those assets are embedded into the Rust binary by `rust-embed`.

## Development

Install dependencies:

```sh
npm install
```

Start local dev server:

```sh
npm run dev
```

The default Vite dev server runs on `http://localhost:5173`.

## Build

Build production frontend:

```sh
npm run build
```

This generates static output in `web2/build/` via `@sveltejs/adapter-static`.

Preview production build:

```sh
npm run preview
```

## Type and Lint Checks

```sh
npm run check
npm run lint
```

## Route Contract (Sprint 31)

- `/` is the public landing route.
- `/dashboard/*` is reserved for authenticated application workflows.
- Backend APIs remain under `/api/*` and WebSocket at `/api/ws/:session_id`.

## Typography Contract (DAL-3103)

Dalang's brand typography is fully self-hosted and loaded from `web2/static/fonts/*`.

- UI/Headings: `Plus Jakarta Sans` variable font (`PlusJakartaSans-Variable.woff2`, `PlusJakartaSans-Variable-ext.woff2`)
- Data/Logs: `JetBrains Mono` variable font (`JetBrainsMono-Variable.woff2`)
- Cipher/Javanese Reveal: `Noto Sans Javanese` (`javanese.woff2`, `latin.woff2`, `latin-ext.woff2`)

Fallback chains used in `layout.css`:

- Desktop UI chain: `Plus Jakarta Sans, Inter, Segoe UI, Noto Sans, system-ui, sans-serif`
- Mobile UI chain guidance: `Plus Jakarta Sans, Noto Sans, Roboto, system-ui, sans-serif`
- Desktop Javanese chain: `Noto Sans Javanese, Noto Sans Javanese UI, Noto Sans, Segoe UI, sans-serif`
- Mobile Javanese chain guidance: `Noto Sans Javanese, Noto Sans, Roboto, sans-serif`
