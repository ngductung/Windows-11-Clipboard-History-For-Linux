import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Meta from 'gi://Meta';
import Shell from 'gi://Shell';

import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import { Extension } from 'resource:///org/gnome/shell/extensions/extension.js';

const KEYBINDING_NAME = 'toggle-clipboard';
const WINDOW_TITLE = 'Clipboard History';
const LOG_PREFIX = '[Win11ClipboardHistoryPointer]';
const WINDOW_PADDING = 10;

export default class Win11ClipboardHistoryExtension extends Extension {
  enable() {
    this._settings = this.getSettings();
    log(`${LOG_PREFIX} enabled`);

    Main.wm.addKeybinding(
      KEYBINDING_NAME,
      this._settings,
      Meta.KeyBindingFlags.NONE,
      Shell.ActionMode.NORMAL | Shell.ActionMode.OVERVIEW | Shell.ActionMode.POPUP,
      () => this._openAtPointer()
    );
  }

  disable() {
    Main.wm.removeKeybinding(KEYBINDING_NAME);
    log(`${LOG_PREFIX} disabled`);
    this._settings = null;
  }

  _openAtPointer() {
    const [x, y] = global.get_pointer();
    const command = this._settings.get_string('command') || 'win11-clipboard-history';
    const targetWindow = global.display.focus_window;
    const targetHint = this._targetHint(targetWindow);
    log(`${LOG_PREFIX} keybinding fired at ${Math.round(x)}, ${Math.round(y)} using ${command}`);

    try {
      const existingWindow = this._findClipboardWindow();
      const existingWindowVisible = existingWindow?.showing_on_its_workspace?.() ?? false;
      if (existingWindowVisible) {
        log(`${LOG_PREFIX} window is visible, delegating close without moving`);
        this._spawnApp(command, Math.round(x), Math.round(y), targetHint);
        return;
      }

      if (existingWindow) {
        try {
          log(`${LOG_PREFIX} pre-positioning hidden window to ${Math.round(x)}, ${Math.round(y)}`);
          this._moveWindowToPointer(existingWindow, Math.round(x), Math.round(y));
        } catch (error) {
          logError(error, `${LOG_PREFIX} failed to pre-position hidden clipboard window`);
        }
      }

      this._spawnApp(command, Math.round(x), Math.round(y), targetHint);
    } catch (error) {
      logError(error, 'Failed to open win11-clipboard-history at pointer');
    }
  }

  _spawnApp(command, x, y, targetHint = {}) {
    const [, argv] = GLib.shell_parse_argv(command);
    argv.push('--show-at', `${x}`, `${y}`);
    if (targetHint.appId)
      argv.push('--target-app', targetHint.appId);
    if (targetHint.title)
      argv.push('--target-title', targetHint.title);
    Gio.Subprocess.new(argv, Gio.SubprocessFlags.NONE);
  }

  _targetHint(window) {
    if (!window || this._isClipboardWindow(window))
      return {};

    return {
      appId:
        window.get_gtk_application_id?.() ||
        window.get_wm_class?.() ||
        window.get_wm_class_instance?.() ||
        '',
      title: window.get_title?.() || '',
    };
  }

  _findClipboardWindow() {
    for (const actor of global.get_window_actors()) {
      const window = actor.meta_window;
      if (this._isClipboardWindow(window))
        return window;
    }

    for (const window of global.display.list_all_windows?.() ?? []) {
      if (this._isClipboardWindow(window))
        return window;
    }

    return null;
  }

  _isClipboardWindow(window) {
    if (!window)
      return false;

    const title = window.get_title?.() ?? '';
    const wmClass = window.get_wm_class?.() ?? '';
    const wmClassInstance = window.get_wm_class_instance?.() ?? '';

    return (
      title.includes(WINDOW_TITLE) ||
      wmClass.toLowerCase().includes('clipboard') ||
      wmClassInstance.toLowerCase().includes('clipboard')
    );
  }

  _moveWindowToPointer(window, x, y) {
    if (!window || !this._settings)
      return;

    const monitor = this._getMonitorGeometryForPoint(x, y);
    if (!monitor)
      return;

    const frame = window.get_frame_rect();
    const frameWidth = Math.max(frame?.width ?? 360, 1);
    const frameHeight = Math.max(frame?.height ?? 480, 1);

    const minX = monitor.x + WINDOW_PADDING;
    const minY = monitor.y + WINDOW_PADDING;
    const maxX = Math.max(minX, monitor.x + monitor.width - frameWidth - WINDOW_PADDING);
    const maxY = Math.max(minY, monitor.y + monitor.height - frameHeight - WINDOW_PADDING);

    const safeX = Math.min(Math.max(x, minX), maxX);
    const safeY = Math.min(Math.max(y, minY), maxY);

    window.move_frame(true, safeX, safeY);
    log(`${LOG_PREFIX} moved window to ${safeX}, ${safeY}`);
  }

  _getMonitorGeometryForPoint(x, y) {
    const monitors = Main.layoutManager.monitors;

    for (const monitor of monitors) {
      if (
        x >= monitor.x &&
        x < monitor.x + monitor.width &&
        y >= monitor.y &&
        y < monitor.y + monitor.height
      )
        return monitor;
    }

    return Main.layoutManager.currentMonitor ?? monitors[0];
  }
}
