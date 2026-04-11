# UI Redesign — Serial Port Debugger

## Overview

Complete UI redesign of the serial port debugging application. Moving from the current VS Code-inspired multi-panel layout to a **top-tab-bar layout** with **professional tool styling** (dark, information-dense, monospace) and a **settings drawer** pattern.

**Design goals:**
- Maximize content area by showing one functional view at a time
- Professional tool aesthetic (deep dark theme, high-contrast data, monospace typography)
- Efficient workflows — key connection info always visible, settings accessible via drawer
- All icons as SVG files imported via `vite-plugin-solid-svg` (no emoji)

## Layout Structure

Four horizontal layers, top to bottom:

### Layer 1: Title Bar
- App logo (left)
- Connection status capsule: port name · baud rate · parameters (e.g. `COM3 · 115200 · 8N1`)
- Connect/Disconnect button (toggles based on state)
- Settings gear icon (opens drawer) + Theme toggle icon (right)
- Always visible, provides at-a-glance connection state

### Layer 2: Tab Bar
Five tabs, underline indicates active:
1. **Monitor** — data stream viewer
2. **Send** — manual data sending
3. **Quick Commands** — predefined command management
4. **Bridge** — bidirectional port forwarding
5. **Modbus** — protocol frame analysis

Clicking a tab replaces the entire content area below.

### Layer 3: Content Area
- Fills all remaining vertical space between tab bar and status bar
- Each tab has its own sub-toolbar at the top for tab-specific controls
- Content scrolls independently within this zone

### Layer 4: Status Bar
- Single row at bottom, always visible
- Left: connection indicator dot + port name
- Center: TX byte count, RX byte count, frame count
- Right: full connection params (baud · data bits · parity · stop bits · flow control)

## Tab Designs

### Monitor Tab
**Sub-toolbar:** Format toggle group (HEX / ASCII / DEC) | Auto-scroll toggle | Log-to-file toggle | Export button | Clear button

**Content:** Table-aligned data stream:
- Columns: Timestamp | Direction badge (RX green / TX blue) | Data (monospace, letter-spaced) | Annotation (dimmed, right-aligned)
- Modbus frames auto-detected: left orange border + tinted background + parsed info in annotation column (e.g. `Modbus RTU · Addr:01 Func:03 CRC:✓`)
- Auto-scrolls to bottom when enabled

### Send Tab
**Sub-toolbar:** Format toggle (HEX / ASCII / DEC) | Line ending toggle (None / CR / LF / CRLF) | Checksum algorithm select

**Content:**
- Large text input area (monospace, dark background, rounded border)
- Below input: real-time preview of parsed interpretation
- Bottom action bar:
  - Left: Timed-send checkbox + interval input (ms)
  - Right: Clear button + Send button (primary accent color)

### Quick Commands Tab
**Sub-toolbar:** New command button | Import/Export buttons | Batch send mode toggle

**Content:** Table with columns:
- Checkbox | Name | Data (HEX) | Format | Actions (Send / Edit)
- Active looping command: left accent border + tinted background
- Bottom bar: Loop interval input (ms) + "Batch send selected" button

### Bridge Tab
**Sub-toolbar:** Port A selector | bidirectional arrow icon | Port B selector | Start/Stop bridge button

**Content:** Side-by-side dual pane:
- Left pane: A → B traffic with byte counter
- Right pane: B → A traffic with byte counter
- Each pane scrolls independently, shows timestamped hex data

### Modbus Tab
**Sub-toolbar:** Protocol toggle (RTU / ASCII) | Filter button | Export button | Clear button

**Content:** Structured frame table with columns:
- Timestamp | Direction badge | Address | Function code (with name) | Data | Checksum status | View raw link
- Visual indicators:
  - Normal response: subtle green tint background
  - Exception response: red left border + red tint, function code and data in red
  - CRC error: orange left border + orange tint, checksum column shows "CRC ERR" in orange

## Settings Drawer

Triggered by gear icon in title bar. Slides in from right side, 280px wide.

**Overlay:** Semi-transparent backdrop over main content. Click backdrop or X button to close.

**Structure:**
- Header: "Settings" title + close button
- Sub-tabs: Serial | BLE
- Content scrolls within drawer

**Serial settings form:**
- Port: dropdown + refresh button
- Baud rate: dropdown (with common presets)
- Data bits: button group (5 / 6 / 7 / 8)
- Parity: button group (None / Odd / Even)
- Stop bits: button group (1 / 2)
- Flow control: button group (None / HW / SW)
- Connect button (full width, accent color)

**BLE settings form:**
- Service UUID input
- Characteristic UUID input
- Scan button
- Device list (populated after scan)
- Connect button

## Color System

Dark professional palette using OKLch color space (existing DaisyUI theme infrastructure):

| Role | Color | Usage |
|------|-------|-------|
| Background deepest | `#08081a` | Title bar, status bar |
| Background deep | `#0d0d1a` | Tab bar, sub-toolbars, table headers |
| Background main | `#12122a` | Content area |
| Background elevated | `#1a1a2e` | Input fields, inactive buttons |
| Border | `#2a2a4a` | All borders and dividers |
| Text primary | `#e0e0e0` | Data, main content |
| Text secondary | `#888` | Labels, parameters |
| Text muted | `#555` | Timestamps, hints |
| Text inactive | `#6c6c9a` | Inactive tabs, disabled controls |
| Accent primary | `#a78bfa` | Active tab, selected states, primary buttons |
| RX indicator | `#4ade80` | Received data badge, connected status |
| TX indicator | `#60a5fa` | Transmitted data badge |
| Warning | `#f59e0b` | CRC errors, Modbus frame highlights |
| Error | `#ef4444` | Modbus exceptions, error states |

Light theme follows the same structure with inverted luminance — to be derived from existing DaisyUI light theme definitions.

## Icon Strategy

All icons are SVG files in `src/assets/icons/`, imported as SolidJS components via `vite-plugin-solid-svg`.

**Existing icons to reuse:**
- `settings.svg`, `sun.svg`, `moon.svg`, `refresh.svg`, `trash.svg`, `plus.svg`, `search.svg`, `bluetooth.svg`, `serial.svg`, `menu.svg`

**New icons needed:**
- `monitor.svg` — data stream / terminal display
- `send.svg` — outgoing arrow
- `quick-command.svg` — lightning bolt or command prompt
- `bridge.svg` — bidirectional arrows
- `modbus.svg` — protocol / frame icon
- `export.svg` — download or share
- `play.svg` / `stop.svg` — for loop/bridge controls
- `log-file.svg` — file with pencil or record indicator
- `auto-scroll.svg` — arrow pointing down
- `close.svg` — X mark for drawer
- `arrow-bidirectional.svg` — for bridge header
- `filter.svg` — funnel icon
- `edit.svg` — pencil icon for quick command editing
- `import.svg` — upload icon
- `disconnect.svg` — unplug / X on connector

## Component Architecture

### Removed Components
- `ActivityBar` — replaced by title bar icons
- `TopBar` — replaced by title bar + tab bar
- `BridgePanel` (modal) — replaced by Bridge tab

### New / Refactored Components

```
App.tsx
├── TitleBar.tsx          — logo, connection capsule, connect/disconnect, settings gear, theme toggle
├── TabBar.tsx            — 5 tabs with active indicator
├── ContentArea.tsx       — renders active tab's content
│   ├── MonitorTab.tsx    — refactored from RecordPanel + ControlPanel
│   ├── SendTab.tsx       — refactored from SendPanel
│   ├── QuickCommandTab.tsx — refactored from QuickInputPanel
│   ├── BridgeTab.tsx     — refactored from BridgePanel (modal → full tab)
│   └── ModbusTab.tsx     — new dedicated view (was inline in MonitorTab)
├── SettingsDrawer.tsx    — right-side drawer overlay
│   ├── SerialSettings.tsx — refactored from DeviceSetting/SerialSetting
│   └── BleSettings.tsx    — refactored from DeviceSetting/BleSetting
└── StatusBar.tsx         — refactored, simplified
```

### State Changes (`src/stores/ui.ts`)

**Remove:**
- `showSettingPanel` — replaced by drawer open/close
- `showSendPanel` — no longer toggled, always in its tab
- `showQuickInputPanel` — no longer toggled, always in its tab
- `showFullScreen` — app is always full window
- `showBridgePanel` — no longer a modal

**Add:**
- `activeTab: "monitor" | "send" | "quick" | "bridge" | "modbus"` — current tab
- `showSettingsDrawer: boolean` — drawer visibility

**Keep:**
- `theme`, `deviceType`, `activePortId`, `connectedSerialPorts`, `serialConnected`, `bleConnected`
- `readDisplayFormat`, `sendDisplayFormat`, `pinBottom`

## Behavior Notes

- Tab switching is instant — no transitions or animations needed
- Settings drawer slides in with a brief CSS transition (transform translateX)
- Connection capsule in title bar updates reactively from store state
- Monitor tab preserves scroll position and data when switching away and back
- Each tab's sub-toolbar state persists independently (e.g., format selection on Monitor tab stays when switching to Send and back)
- Keyboard shortcut: Ctrl+, (Cmd+,) opens settings drawer
