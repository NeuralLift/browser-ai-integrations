# Action Protocol Design Doc

## 1. Overview

The Action Protocol defines the communication schema between the Backend (AI Controller) and the Browser Extension (Executor) for browser automation tasks. It enables the AI to observe the page state via snapshots and perform actions like navigation, clicking, typing, and scrolling.

## 2. ActionCommand Schema

ActionCommands are sent from the Backend to the Extension to trigger specific browser actions.

### navigate_to

Navigates the current tab to a specified URL.

```json
{
  "type": "navigate_to",
  "url": "https://example.com"
}
```

### click_element

Clicks an element identified by its reference ID.

```json
{
  "type": "click_element",
  "ref": 1
}
```

### type_text

Types text into an element identified by its reference ID.

```json
{
  "type": "type_text",
  "ref": 2,
  "text": "Hello World"
}
```

### scroll_to

Scrolls the page to specific coordinates.

```json
{
  "type": "scroll_to",
  "x": 0,
  "y": 500
}
```

## 3. ActionResult Schema

ActionResults are sent from the Extension back to the Backend to report the outcome of an action.

```json
{
  "success": true,
  "error": null,
  "data": {}
}
```

- `success`: Boolean indicating if the action was executed successfully.
- `error`: Optional string containing the error message if `success` is false.
- `data`: Optional object containing any data returned by the action.

## 4. Snapshot Schema

A Snapshot provides a structural representation of the current page's interactive elements and accessibility tree.

```json
{
  "tree": [
    {
      "id": 1,
      "role": "button",
      "name": "Submit",
      "tag": "BUTTON",
      "bounds": {
        "x": 100,
        "y": 200,
        "width": 80,
        "height": 30
      }
    }
  ]
}
```

- `id`: The sequential reference ID assigned to the element.
- `role`: The ARIA role or calculated role of the element (e.g., "link", "button", "textbox").
- `name`: The accessible name of the element (e.g., button text, aria-label, alt text).
- `tag`: The HTML tag name (e.g., "DIV", "A", "BUTTON").
- `bounds`: The bounding box of the element relative to the viewport.

## 5. Ref Assignment Strategy

To ensure consistent and efficient element referencing, the extension follows these rules:

1.  **Depth-First Traversal**: The DOM tree is traversed using a depth-first search (DFS) pattern.
2.  **Sequential IDs**: IDs are assigned sequentially starting from `1` for each interactive element found.
3.  **Interactive Elements Only**: Only elements that can be interacted with are assigned a `ref`. This includes:
    - `<a>`, `<button>`, `<input>`, `<select>`, `<textarea>`
    - Elements with `cursor: pointer` style.
    - Elements with ARIA roles: `button`, `link`, `checkbox`, `menuitem`, etc.
    - Elements with `tabindex` >= 0.
4.  **Viewport-Only Filtering**: Only elements currently visible in the viewport are included in the snapshot to reduce noise and context window usage.
5.  **Exclusion Rules**:
    - Elements with the `data-browser-agent-ui` attribute (used for internal agent UI) are excluded.
    - Hidden elements (`display: none`, `visibility: hidden`).

## 6. WebSocket Message Format

Actions and snapshots are wrapped in the `WsMessage` envelope used by the backend.

### Sending an Action (Backend -> Extension)

```json
{
  "type": "ActionCommand",
  "data": {
    "type": "click_element",
    "ref": 1
  }
}
```

### Receiving a Snapshot (Extension -> Backend)

```json
{
  "type": "Snapshot",
  "data": {
    "tree": [...]
  }
}
```

## 7. Example Flow

1.  **User asks**: "Click the login button."
2.  **Extension sends Snapshot**:
    ```json
    {
      "type": "Snapshot",
      "data": {
        "tree": [
          { "id": 1, "role": "button", "name": "Login", "tag": "BUTTON", "bounds": {...} }
        ]
      }
    }
    ```
3.  **Backend processes** and decides to click ref 1.
4.  **Backend sends ActionCommand**:
    ```json
    {
      "type": "ActionCommand",
      "data": { "type": "click_element", "ref": 1 }
    }
    ```
5.  **Extension executes** click and returns **ActionResult**:
    ```json
    {
      "type": "ActionResult",
      "data": { "success": true }
    }
    ```
