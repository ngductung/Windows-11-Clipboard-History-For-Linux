# GNOME Wayland Pointer Launcher

This extension is for GNOME Wayland sessions where normal applications cannot read the global pointer position.
It binds `Super+V`, reads the pointer from GNOME Shell, and launches:

```bash
win11-clipboard-history --show-at X Y
```

Install it with:

```bash
scripts/install-gnome-extension.sh
```

If the app is not available as `win11-clipboard-history`, pass the command to use:

```bash
scripts/install-gnome-extension.sh "/path/to/win11-clipboard-history"
```

The install script also frees `Super+V` from GNOME's notification shortcut and disables the app's old GNOME custom shortcut binding so the extension owns the shortcut.

Customize the shortcut from the extension preferences:

```bash
gnome-extensions prefs win11-clipboard-history@gustavosett.dev
```

Click the shortcut button, then press the key combination you want to use.
Press `Esc` to cancel recording, or `Backspace`/`Delete` to disable the shortcut.

On GNOME Wayland, log out and back in after reinstalling the extension so GNOME Shell reloads extension code and schema cleanly.
