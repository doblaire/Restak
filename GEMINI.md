# Restak – Project Context & Architecture Guide

## 1. Project Overview
**Restak** is a lightweight, high-performance game launcher and front-end interface built with **Tauri v2 (Rust backend)** and a responsive **Web Frontend (vanilla HTML, CSS, JavaScript)**.

Restak is designed to act as a sleek, seamless front-end for your games library, supporting:
- **Steam Games**: Automatic manifest resolution, Steam DRM auto-initialization, App ID launching (`steam://run/<id>`).
- **Emulated Titles / ROMs**: Direct batch execution with emulators like Dolphin (GameCube/Wii), mGBA (Game Boy Advance), RetroArch, etc.
- **Native Applications**: Executables and desktop bundles.

---

## 2. Frontend Architecture & Design System

### A. Responsive Proportional Scaling System
- **Design Baseline**: Built against a Figma 1920×1080 reference canvas.
- **Dynamic Scale Factor**: In `renderer.js`, `canvasScale` calculates `min(window.innerWidth / 1920, window.innerHeight / 1080)` clamped between `0.55` and `1.4`.
- **CSS Variable**: `--ui-scale` is dynamically set on the root `:root` / `<html>`.
- **Element Scaling**: Critical dimensions (font sizes, gaps, paddings, modal containers) use `calc(Xpx * var(--ui-scale, 1))`.

### B. 7-Card Stacking Shelf
- **Grid Layout**: `.game-shelf` uses a 7-column CSS grid:
  ```css
  display: grid;
  grid-template-columns: repeat(7, minmax(0, 1fr));
  gap: calc(24px * var(--ui-scale, 1));
  padding: calc(30px * var(--ui-scale, 1)) 60px 60px 60px;
  ```
- **Behavior**: On default displays (including 14" MacBook and 1080p monitors), games form rows of 7 between the 60px side gutters. Adding more titles automatically wraps into subsequent rows with vertical scrolling (`overflow-y: auto; overflow-x: hidden`).

### C. Corner Anchoring
- **Top Bar**: Fixed header with `position: absolute; top: max(0px, var(--safe-top)); width: 100%`.
- **Top-Left**: `.stak-info` (Stak Title + dot + count) anchored 60px from the left edge.
- **Top-Right**: `.system-time` (Live clock) anchored 60px from the right edge.
- **Top-Center**: `.nav-bar` stays horizontally centered (`left: 50%; transform: translateX(-50%) ...`).

### D. Navigation Bar Tray & Drag Physics
- **Morphing SVG Tray**: The background is an SVG path (`#navTrayPath`) that morphs between 536px width (default) and 900px width (search expanded) using custom bezier curves.
- **Critical Transform Order**:
  ```css
  .nav-bar {
    transform: translateX(-50%) scale(var(--ui-scale, 1)) translateY(0);
  }
  .nav-bar.nav-hidden {
    transform: translateX(-50%) scale(var(--ui-scale, 1)) translateY(-102px);
  }
  .nav-bar.nav-hidden.nav-peek {
    transform: translateX(-50%) scale(var(--ui-scale, 1)) translateY(-84px);
  }
  ```
  > **Note**: `scale()` MUST precede `translateY()` so vertical translations operate in local coordinate units. This ensures `translateY(-84px)` leaves exactly 18 local pixels (`18px * scale`) peeking into the viewport on all displays regardless of scale factor.
- **Drag-to-Tuck Physics**:
  - Pulling up past -20px tucks the navbar offscreen.
  - Hovering within 50px of the top edge peeks the bottom grab zone down.
  - Clicking or dragging the peek tab restores the full navigation bar.
- **Spring Animations**: Easing curves derived from Figma (stiffness 1200, damping 200, mass 1).
- **Search Button**: Excluded from `:hover` scale (`scale(1.08)`) via `.nav-btn:not(.nav-btn-search):hover` so it doesn't pop or grow when retracted, while participating in the `-70px` tuck and spring recoil bounce when opening the navbar.

---

## 3. Backend Architecture (`src-tauri/`)

- **Tauri Core**: Rust-based engine with IPC handlers registered in `src-tauri/src/lib.rs`.
- **Commands**:
  - `launch_game(game)`: Handles launching Steam titles, emulators, and native executables. Manages background exit tracking and emits `game-status` and `game-exited` events.
  - `scan_local_games()`: Automatically scans filesystem locations for installed titles.
  - `get_saved_library()` / `save_library(games)`: Persists user games to `games.json` in the user's app data directory.
  - `toggle_borderless_fullscreen()`: Hotkey (`F` / `F11`) handler for fullscreen desktop coverage.
- **Steam Integration**:
  - Manifest ACF file parser searches `steamapps/appmanifest_*.acf` to match game folders with Steam App IDs.
  - Verifies Steam client background process before launching to guarantee `SteamAPI_Init()` succeeds without crashes.

---

## 4. Cross-Platform Guidelines (Windows & macOS)

### macOS Nuances:
- App bundles (`.app`) launched via LaunchServices (`open -a`).
- Steam path: `~/Library/Application Support/Steam/steamapps`.
- Safe area inset handling for MacBook screen notches (`--safe-top`).

### Windows Adaptations Needed:
- Executables are `.exe` or `.lnk` shortcuts.
- Launching: Use `std::process::Command::new(exec_path)` or `cmd /c start "" "<path>"`.
- Steam path: Check Windows Registry `HKCU\Software\Valve\Steam\SteamPath` or `C:\Program Files (x86)\Steam\steamapps`.
- Emulators: Detect standard Windows install directories (e.g. `%APPDATA%`, `C:\Program Files\`, `%LOCALAPPDATA%`).
- Display engine on Windows uses Microsoft Edge WebView2 (automatically provided by Windows 10/11).

---

## 5. Development & Build Commands

```bash
# Sync frontend assets into dist/
npm run build:dist

# Run development mode with live hot-reloading
npm run tauri dev

# Build production executable
npm run tauri build
```

*When editing `index.html`, `styles.css`, or `renderer.js`, always run `npm run build:dist` to ensure embedded web assets in `dist/` are up to date before compiling Rust.*

---

## 6. Icon Assets & Source of Truth
- **Strict Rule**: NEVER invent, generate, draw, or insert arbitrary placeholder SVG icons.
- **Canonical Repository**: Always and exclusively pull from `icons/` (`auto.svg`, `back.svg`, `close.svg`, `collapse.svg`, `edit.svg`, `file.svg`, `folder.svg`, `home.svg`, `info.svg`, `nut.svg`, `plus box.svg`, `refetch.svg`, `search.svg`, `staks.svg`, `undo or redo.svg`, `upload.svg`) unless the user explicitly provides a new SVG or requests otherwise.
