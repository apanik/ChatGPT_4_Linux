# Codex ChatGPT Client (Unofficial Linux)

> Unofficial Linux client. Not affiliated with OpenAI.

This project delivers a two-window ChatGPT desktop experience for Linux that mirrors the official macOS/Windows apps as closely as possible. It uses **Tauri (Rust + WebKitGTK)**, prioritizes Wayland via xdg-desktop-portal, and avoids handling credentials directly by loading the official ChatGPT site (`https://chatgpt.com`).

## Features
- Main window that hosts the official ChatGPT web UI with menu shortcuts for New Chat, Find-in-chat (Ctrl+F), and external link handling.
- Companion window launched via a global hotkey (default **Alt+Space**), always-on-top toggle, remembers position, and reset-to-bottom-center logic.
- System tray with quick actions (open companion/main, new chat, settings, quit).
- Native settings window for hotkeys, notifications, link handling, launch-at-login, and privacy (clear data).
- Login through the official ChatGPT flow including Google sign-in (credentials never intercepted).
- Built-in disclaimer in About/Settings.

## Project layout
- `src-tauri/`: Rust backend, window creation, global hotkeys, settings persistence.
- `src/`: Minimal HTML for settings and fallback landing page.
- `ARCHITECTURE.md`: Detailed design notes and roadmap.
- `TROUBLESHOOTING.md`: Common issues (Wayland portals, hotkeys, Google login).

## Developing
Prerequisites: Rust toolchain, Node.js (for the Tauri CLI), and WebKitGTK dependencies (see the [Tauri Linux guide](https://tauri.app/v1/guides/getting-started/prerequisites/#linux)).

```bash
# Install JS deps (CLI only)
npm install

# Run in dev mode (spawns Tauri + webview)
npm run tauri:dev

# Build installers (AppImage by default; .deb/.rpm when toolchains are available)
npm run tauri:build
```

## Security & privacy
- Authentication happens only inside the embedded ChatGPT site or via the external browser if Google blocks the webview. No passwords or tokens are stored by the app.
- Settings live in `~/.config/codex-chatgpt-client/settings.json`. Use **Settings → Clear local data** to wipe settings and cached cookies.
- Microphone and screenshot permissions are only requested when you trigger voice/screenshot actions.

## Platform notes
- **Wayland**: Global hotkeys rely on the available compositor portal; screenshots and file pickers prefer xdg-desktop-portal.
- **X11**: Hotkeys fall back to XGrabKey; screenshots use the native portal when available.

## Roadmap
See `ARCHITECTURE.md` for the incremental plan (companion workflows, screenshot/file injection, notifications, packaging polish).
