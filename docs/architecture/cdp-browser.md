# CDP Browser

Dalang integrates a **full-featured headless Chromium browser** via the Chrome DevTools Protocol (CDP) for autonomous web application testing. The browser gives the AI agent complete control — navigating pages, filling forms, clicking buttons, intercepting network traffic, managing cookies, taking screenshots, and handling multiple tabs.

## Architecture

```rust
pub struct DalangBrowser {
    browser: Browser,
    pages: Vec<Page>,          // All open tabs
    active_idx: usize,         // Currently active tab index
    network_log: Arc<Mutex<Vec<NetworkEntry>>>,  // Captured traffic
    network_logging: bool,     // Whether network capture is active
}
```

| Field | Purpose |
|-------|---------|
| `browser` | Chromiumoxide `Browser` handle (with async event handler loop) |
| `pages` | Vector of all open `Page` (tab) handles |
| `active_idx` | Index of the currently focused tab in `pages` |
| `network_log` | Thread-safe buffer of captured `NetworkEntry` records |
| `network_logging` | Flag indicating if request/response interception is active |

### Network Entry

```rust
pub struct NetworkEntry {
    pub url: String,
    pub method: String,
    pub status: Option<i64>,
    pub mime_type: Option<String>,
    pub request_headers: HashMap<String, String>,
    pub response_headers: HashMap<String, String>,
    pub timestamp: f64,
}
```

## Lazy Initialization

The browser is **never** launched at startup. A `LazyBrowser` wrapper inside the engine defers Chrome initialization until the first `browser-*` tool call:

```rust
struct LazyBrowser {
    inner: Arc<Mutex<Option<DalangBrowser>>>,
}
```

This means CLI-only skills (nmap, sqlmap, etc.) work even without Chrome installed.

## Browser Tools Catalog

The engine exposes **38 browser tools** across **9 categories**. All tools follow the same JSON calling convention:

```json
{ "tool": "browser-<command>", "args": { ... } }
```

### Navigation (7 tools)

| Tool | Args | Description |
|------|------|-------------|
| `browser-navigate` | `{"url": "<url>"}` | Navigate to a URL and wait for page load |
| `browser-get-url` | `{}` | Get the current page URL |
| `browser-get-title` | `{}` | Get the page title |
| `browser-get-html` | `{}` | Get the full page HTML source |
| `browser-go-back` | `{}` | Navigate back in history |
| `browser-go-forward` | `{}` | Navigate forward in history |
| `browser-reload` | `{}` | Reload the current page |

### DOM Extraction (2 tools)

| Tool | Args | Description |
|------|------|-------------|
| `browser-extract-dom` | `{}` | Extract simplified DOM text (uses `document.body.innerText`) |
| `browser-evaluate-js` | `{"script": "<js>"}` | Execute JavaScript in page context and return result |

### DOM Query (4 tools)

| Tool | Args | Description |
|------|------|-------------|
| `browser-query-selector` | `{"selector": "<css>"}` | Find first element matching CSS selector (returns tag, id, text, attributes) |
| `browser-query-selector-all` | `{"selector": "<css>", "limit": 20}` | Find all matching elements (default limit 20) |
| `browser-get-attribute` | `{"selector": "<css>", "attribute": "<attr>"}` | Get a specific attribute value from an element |
| `browser-wait-for-selector` | `{"selector": "<css>", "timeout_ms": 5000}` | Wait until an element appears in the DOM |

### Interaction (9 tools)

| Tool | Args | Description |
|------|------|-------------|
| `browser-click` | `{"selector": "<css>"}` | Click an element |
| `browser-type-text` | `{"selector": "<css>", "text": "<text>", "clear": false}` | Type text into an input (`clear=true` clears first) |
| `browser-hover` | `{"selector": "<css>"}` | Hover over an element |
| `browser-focus` | `{"selector": "<css>"}` | Focus an element |
| `browser-select-option` | `{"selector": "<css>", "value": "<val>"}` | Select a dropdown option by value |
| `browser-press-key` | `{"selector": "<css>", "key": "<key>"}` | Press a key (Enter, Tab, Escape, etc.) |
| `browser-fill-form` | `{"fields": {"#sel1": "val1", "#sel2": "val2"}}` | Fill multiple form fields at once |
| `browser-submit-form` | `{"selector": "form"}` | Submit a form (default: first `<form>`) |
| `browser-scroll` | `{"x": 0, "y": 500, "selector": null}` | Scroll the page or a specific element |

### Screenshots (2 tools)

| Tool | Args | Description |
|------|------|-------------|
| `browser-screenshot` | `{"full_page": false, "selector": null}` | Take a screenshot (returns base64 PNG) |
| `browser-screenshot-to-file` | `{"path": "shot.png", "full_page": false}` | Save screenshot to a file on disk |

### Cookies (3 tools)

| Tool | Args | Description |
|------|------|-------------|
| `browser-get-cookies` | `{}` | List all cookies as JSON |
| `browser-set-cookie` | `{"name": "<n>", "value": "<v>", "domain": "<d>", "path": "/", "http_only": false, "secure": false}` | Set a cookie |
| `browser-delete-cookies` | `{"name": "<n>"}` | Delete a cookie by name (omit name to delete all) |

### Storage (3 tools)

| Tool | Args | Description |
|------|------|-------------|
| `browser-get-storage` | `{"storage_type": "local"}` | Get localStorage or sessionStorage (`"local"` or `"session"`) |
| `browser-set-storage` | `{"storage_type": "local", "key": "<k>", "value": "<v>"}` | Set a storage item |
| `browser-clear-storage` | `{"storage_type": "local"}` | Clear all items in a storage type |

### Network & Headers (5 tools)

| Tool | Args | Description |
|------|------|-------------|
| `browser-set-extra-headers` | `{"headers": {"Authorization": "Bearer tok"}}` | Set extra HTTP headers on all subsequent requests |
| `browser-set-user-agent` | `{"user_agent": "<ua>"}` | Override the User-Agent string |
| `browser-enable-network-log` | `{}` | Start capturing all network requests/responses |
| `browser-get-network-log` | `{"clear": false}` | Get captured network entries as JSON (`clear=true` resets log) |
| `browser-set-viewport` | `{"width": 1280, "height": 720}` | Set the browser viewport dimensions |

### Tab Management (4 tools)

| Tool | Args | Description |
|------|------|-------------|
| `browser-new-tab` | `{"url": "<url>"}` | Open a new tab (URL is optional) |
| `browser-list-tabs` | `{}` | List all open tabs with their URLs |
| `browser-switch-tab` | `{"index": 0}` | Switch to a tab by index |
| `browser-close-tab` | `{"index": null}` | Close a tab by index (default: active tab) |

## Use Cases

| Use Case | Tool Sequence |
|----------|---------------|
| **Full Web Audit** | navigate → get-html → query-selector-all (`a, form, input`) → evaluate-js |
| **XSS Detection** | navigate → fill-form → submit-form → get-html → evaluate-js (check for injected script) |
| **SPA Analysis** | navigate → wait-for-selector → extract-dom → screenshot |
| **Token/Secret Leak** | navigate → get-storage → get-cookies → evaluate-js (`document.cookie`) |
| **Form Brute Force** | navigate → fill-form → submit-form → get-url (check redirect) |
| **Network Interception** | enable-network-log → navigate → get-network-log → analyze API calls |
| **Auth Session Test** | set-cookie / set-extra-headers → navigate → extract-dom → verify access |
| **Multi-Page Flow** | new-tab → switch-tab → navigate → compare across tabs |
| **Visual Regression** | navigate → screenshot-to-file → compare screenshots |
| **Cookie Manipulation** | get-cookies → delete-cookies → set-cookie → reload → verify behaviour |

## Headless Mode

The browser runs in **headless mode** by default with anti-detection flags:

```rust
BrowserConfig::builder()
    .arg("--disable-blink-features=AutomationControlled")
    .arg("--no-sandbox")
    .build()
```

This is suitable for servers and CI environments without a display. The `--disable-blink-features=AutomationControlled` flag helps avoid detection by anti-bot mechanisms on target sites.

## Catalog Injection

The full browser tools catalog is auto-generated by `DalangEngine::browser_tools_catalog()` and injected into **all system prompts** (scan, autonomous, interactive — both CLI and WebSocket modes). This ensures the LLM always knows what browser capabilities are available, regardless of the mode being used.
