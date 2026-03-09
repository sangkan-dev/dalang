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
