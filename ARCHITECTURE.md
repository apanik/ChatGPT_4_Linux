# Architecture Plan

This project delivers an unofficial Linux desktop client that mirrors the official ChatGPT desktop applications as closely as possible while respecting security and privacy constraints. It is built with **Tauri (Rust + WebKitGTK)** to keep dependencies small and leverage platform capabilities such as xdg-desktop-portal.

## High-level components
- **Main window** (full app):
  - Loads `https://chatgpt.com` directly inside a WebKitGTK webview.
  - Provides menu/shortcuts for New Chat, Find-in-chat (Ctrl+F) and toggling "Open external links in browser".
  - Exposes a Voice button (when available from the web UI) and a screenshot button that invokes the portal flow.
- **Companion window** (quick chat):
  - Minimal wrapper around `https://chatgpt.com` in a compact window.
  - Triggered via a global hotkey (default Alt+Space). If already open, the hotkey focuses the window instead of spawning a duplicate.
  - Remembers size/position; has a reset-to-bottom-center action on reset.
  - Includes New Chat, Open Main Window, Upload File, and Take Screenshot actions.
- **System tray**: Provides quick actions (open companion/main, new chat, quit) and reflects launch-at-login preference.
- **Settings UI** (native):
  - Manages hotkeys, companion behavior (always-on-top, opacity, position reset), external link handling, notifications, launch at login, privacy (clear local data), and About + disclaimer.
- **Permissions**: Microphone, file picker, and screenshot permissions are requested only when the user invokes those actions. Login is handled solely through the official ChatGPT web flow.

## Data flow and state
- **Settings persistence**: Stored as a JSON configuration (`~/.config/codex-chatgpt-client/settings.json`). Managed through a Rust settings service with validation and defaults. Clearing data wipes both this file and the webview cache/cookies.
- **Window state**: Persisted per window (size/position/always-on-top/opacity) in the settings service. Companion window defaults to bottom-center if no stored position exists or after an explicit reset.
- **Hotkey management**: Backed by `tauri-plugin-global-shortcut`. On Wayland, attempts xdg-desktop-portal global shortcuts first; falls back to X11 grab where available. Collisions are detected and surfaced in the settings UI with guidance.

## Platform integration
- **Web loading**: Both windows target `https://chatgpt.com` with hardened permissions (no local file scheme access). External links can be forced to open in the user’s default browser based on the toggle.
- **File uploads & screenshots**: Trigger the portal file picker and portal screenshot/screencast APIs. Captured files are injected into the ChatGPT composer via JavaScript that simulates file input selection/paste. Keyboard shortcuts default to Ctrl+Shift+1 (desktop) and Ctrl+Shift+2 (active window) and are user-configurable.
- **Voice**: Only exposed in the main window. When the site exposes advanced voice, microphone permissions are requested via the webview and propagated through Tauri’s permission system.
- **Notifications**: Rust backend listens for page events (message completion via injected observer) and fires native desktop notifications when enabled and windows are unfocused.
- **Launch at login**: Writes an autostart desktop entry on Linux when enabled.

## Build and packaging
- **Dev workflow**: `npm install` (for the minimal front-end assets) then `npm run tauri dev` / `npm run tauri build`.
- **Packaging**: Tauri’s Linux bundler produces an AppImage by default; `.deb`/`.rpm` are enabled when system tooling is available.
- **Testing**: Rust unit tests cover settings/hotkey parsing; a smoke test ensures windows can be created and the ChatGPT URL is set.

## Incremental implementation plan
1. **Bootstrap**: Tauri scaffold, main window loading chatgpt.com with external link handling and menu shortcuts.
2. **Companion window**: Global hotkey, position persistence, reset rules, and open-main/new-chat actions.
3. **File upload & screenshots**: Portal-based pickers and injection pipeline; add shortcuts and buttons in both windows.
4. **Tray, notifications, settings**: System tray actions, native settings window with toggles, notification pipeline for background responses.
5. **Packaging and docs**: README updates, TROUBLESHOOTING, and build scripts for AppImage/.deb/.rpm.
