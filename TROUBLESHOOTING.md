# Troubleshooting

## Global hotkey conflicts
- The default Alt+Space shortcut may be claimed by your desktop environment. Open **Settings → Companion Hotkey** and choose a different accelerator (e.g., Ctrl+Shift+Space). The app will re-register it automatically.
- On Wayland, some compositors require xdg-desktop-portal support for global shortcuts. Ensure `xdg-desktop-portal` and a matching backend (e.g., `xdg-desktop-portal-gnome`, `...-kde`) are installed and running.

## Wayland portals
- Screenshots and file pickers rely on the portal. If the prompt does not appear, verify `XDG_SESSION_TYPE=wayland` and that your portal backend is healthy (`journalctl -xe | grep portal`).
- If your compositor blocks the global shortcut API, configure a custom shortcut inside your DE to execute the binary with `--toggle-companion` (future flag) as a workaround.

## Google login inside the webview
- The app never intercepts credentials. If Google blocks sign-in inside the embedded webview, open the login flow in your system browser from the ChatGPT page and return to the app once authenticated.
- Clearing cookies: open **Settings → Clear local data**, then retry the login flow.

## Window placement
- Use **Reset to bottom-center** in Settings if the companion opens off-screen. Size/position are stored in `~/.config/codex-chatgpt-client/settings.json`.
