# CDP Browser

Dalang integrates a headless Chromium browser via the Chrome DevTools Protocol (CDP) for web application testing.

## Features

- Navigate to URLs and wait for page load
- Extract DOM text content for LLM analysis
- Execute arbitrary JavaScript in page context
- Lazy initialization — only launched when needed

## Architecture

```rust
pub struct DalangBrowser {
    browser: Browser,
    active_page: Arc<Mutex<Option<Page>>>,
}
```

The browser is wrapped in `Arc<Mutex>` for safe concurrent access from the engine's async context.

## Available Tool Calls

### `browser-navigate`

Navigate to a URL and wait for the page to fully load.

```json
{ "tool": "browser-navigate", "args": { "url": "https://example.com" } }
```

### `browser-extract-dom`

Extract the visible text content from the page (uses `document.body.innerText`).

```json
{ "tool": "browser-extract-dom", "args": {} }
```

This returns plain text instead of full HTML to reduce token usage.

### `browser-evaluate-js`

Execute JavaScript in the page context and return the result.

```json
{
  "tool": "browser-evaluate-js",
  "args": { "script": "document.title" }
}
```

## Use Cases

| Use Case             | Tool Sequence                                             |
| -------------------- | --------------------------------------------------------- |
| **Web Audit**        | navigate → extract-dom → evaluate-js                      |
| **XSS Detection**    | navigate → evaluate-js (check DOM for unsanitized input)  |
| **SPA Analysis**     | navigate → wait → extract-dom (after JS rendering)        |
| **Token Leak Check** | navigate → evaluate-js (`localStorage`, `sessionStorage`) |

## Headless Mode

The browser runs in **headless mode** by default, suitable for servers and CI environments without a display.
