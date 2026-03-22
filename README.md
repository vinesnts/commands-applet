# Commands Applet

A powerful and flexible COSMIC desktop applet to run custom commands directly from your panel.

---

## ✨ Features

* 📂 Nested menus and submenus
* ⚡ Run any shell command
* 🖥 Optional terminal execution
* 🚀 Run apps silently (no terminal)
* 🎨 Fully integrated with COSMIC theme
* ⚙️ User-configurable terminal emulator

---

## 📦 Installation

Build and install:

```bash
cargo build --release
cp target/release/commands-applet ~/.local/bin/
```

Install desktop + metadata files (if applicable), then restart panel:

```bash
cosmic-panel --replace
```

---

## ⚙️ Configuration

The applet automatically creates a config file:

```bash
~/.config/commands-applet/commands.json
```

---

## 🧾 Example Configuration

```json
{
  "icon": "display-symbolic",
  "terminal": "auto",
  "menu": [
    {
      "title": "Applications",
      "type": "submenu",
      "icon": "applications-system-symbolic",
      "submenu": [
        {
          "title": "Calculator",
          "command": "flatpak run org.gnome.Calculator",
          "terminal": false
        },
        {
          "title": "Files",
          "command": "nautilus",
          "terminal": false
        }
      ]
    },
    {
      "title": "System",
      "type": "submenu",
      "icon": "utilities-terminal-symbolic",
      "submenu": [
        {
          "title": "Logs",
          "command": "tail -f /var/log/syslog",
          "terminal": true
        }
      ]
    }
  ]
}
```

---

## 🖥 Terminal Behavior

Each command can control how it runs:

### Run WITHOUT terminal

```json
{
  "command": "firefox",
  "terminal": false
}
```

✔ Runs in background
✔ No terminal window
✔ Fully detached process

---

### Run WITH terminal

```json
{
  "command": "htop",
  "terminal": true
}
```

✔ Opens terminal
✔ Keeps session open

---

### Default behavior

If omitted:

```json
{
  "command": "htop"
}
```

👉 Defaults to:

```json
"terminal": false
```

---

## 🧠 How it works

* Commands with `terminal: false` are executed using a fully detached process (`setsid`)
* Commands with `terminal: true` are executed in a terminal emulator
* Terminal emulator is configurable globally

---

## 🖥 Terminal Configuration

Global terminal setting:

```json
{
  "terminal": "auto"
}
```

Supported values:

* `"auto"` (default fallback chain)
* `"gnome-terminal"`
* `"xterm"`
* `"konsole"`

---

## 🔄 Editing & Reloading

From the applet menu:

* **Edit commands.json** → opens config in terminal editor
* **Reload** → reloads config instantly

---

## ⚠️ Security Notice

This applet executes arbitrary shell commands defined in your configuration file.

Only use trusted commands.

---

## 🧑‍💻 Development

Built with:

* Rust 🦀
* COSMIC (`libcosmic`)
* Iced

---

## 📄 License

SPDX-License-Identifier: MPL-2.0

---

## 🚀 Future Ideas

* Notifications on command execution
* Background process tracking
* Command output preview
* Custom terminal commands

---

## ❤️ Contributing

Contributions are welcome!
