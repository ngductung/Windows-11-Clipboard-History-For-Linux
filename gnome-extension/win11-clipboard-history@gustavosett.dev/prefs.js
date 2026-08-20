import Adw from 'gi://Adw?version=1';
import Gdk from 'gi://Gdk?version=4.0';
import Gtk from 'gi://Gtk?version=4.0';

import { ExtensionPreferences } from 'resource:///org/gnome/Shell/Extensions/js/extensions/prefs.js';

const KEYBINDING_NAME = 'toggle-clipboard';
const WAYLAND_FOCUS_SETTLE_KEY = 'wayland-focus-settle-ms';
const EMPTY_SHORTCUT_LABEL = 'Disabled';
const RECORDING_LABEL = 'Press shortcut';
const MODIFIER_KEYVALS = new Set([
  Gdk.KEY_Shift_L,
  Gdk.KEY_Shift_R,
  Gdk.KEY_Control_L,
  Gdk.KEY_Control_R,
  Gdk.KEY_Alt_L,
  Gdk.KEY_Alt_R,
  Gdk.KEY_Meta_L,
  Gdk.KEY_Meta_R,
  Gdk.KEY_Super_L,
  Gdk.KEY_Super_R,
  Gdk.KEY_Hyper_L,
  Gdk.KEY_Hyper_R,
  Gdk.KEY_ISO_Level3_Shift,
  Gdk.KEY_ISO_Level5_Shift,
]);

export default class Win11ClipboardHistoryPreferences extends ExtensionPreferences {
  fillPreferencesWindow(window) {
    const settings = this.getSettings();

    const page = new Adw.PreferencesPage({
      title: 'Launcher',
      icon_name: 'preferences-desktop-keyboard-shortcuts-symbolic',
    });

    const launcherGroup = new Adw.PreferencesGroup({
      title: 'Launcher',
      description: 'Configure how the clipboard history window opens.',
    });
    page.add(launcherGroup);

    launcherGroup.add(this._createShortcutRow(settings));
    launcherGroup.add(this._createWaylandFocusSettleRow(settings));

    window.add(page);
  }

  _createWaylandFocusSettleRow(settings) {
    const adjustment = new Gtk.Adjustment({
      lower: 0,
      upper: 500,
      step_increment: 10,
      page_increment: 50,
      value: settings.get_uint(WAYLAND_FOCUS_SETTLE_KEY),
    });

    const row = new Adw.SpinRow({
      title: 'Wayland paste delay',
      subtitle: 'Lower is faster; increase if paste misses the focused editor.',
      adjustment,
      numeric: true,
      digits: 0,
    });

    row.connect('notify::value', () => {
      settings.set_uint(WAYLAND_FOCUS_SETTLE_KEY, Math.round(row.value));
    });

    return row;
  }

  _createShortcutRow(settings) {
    let recording = false;

    const shortcutButton = new Gtk.Button({
      label: this._shortcutLabel(settings),
      valign: Gtk.Align.CENTER,
    });

    const clearButton = new Gtk.Button({
      icon_name: 'edit-clear-symbolic',
      tooltip_text: 'Disable shortcut',
      valign: Gtk.Align.CENTER,
    });

    const resetButton = new Gtk.Button({
      icon_name: 'edit-undo-symbolic',
      tooltip_text: 'Reset shortcut',
      valign: Gtk.Align.CENTER,
    });

    const row = new Adw.ActionRow({
      title: 'Keyboard shortcut',
      subtitle: this._shortcutSubtitle(settings),
      activatable_widget: shortcutButton,
    });

    const stopRecording = () => {
      recording = false;
      shortcutButton.remove_css_class('suggested-action');
      shortcutButton.label = this._shortcutLabel(settings);
      row.subtitle = this._shortcutSubtitle(settings);
    };

    const startRecording = () => {
      recording = true;
      shortcutButton.add_css_class('suggested-action');
      shortcutButton.label = RECORDING_LABEL;
      row.subtitle = 'Press the shortcut you want to use. Esc cancels.';
      shortcutButton.grab_focus();
    };

    const controller = new Gtk.EventControllerKey();
    controller.connect('key-pressed', (_controller, keyval, _keycode, state) => {
      if (!recording)
        return false;

      if (keyval === Gdk.KEY_Escape) {
        stopRecording();
        return true;
      }

      if (keyval === Gdk.KEY_BackSpace || keyval === Gdk.KEY_Delete) {
        settings.set_strv(KEYBINDING_NAME, []);
        stopRecording();
        return true;
      }

      if (MODIFIER_KEYVALS.has(keyval)) {
        row.subtitle = 'Press a normal key together with the modifiers.';
        return true;
      }

      const modifiers = state & Gtk.accelerator_get_default_mod_mask();
      if (!Gtk.accelerator_valid(keyval, modifiers)) {
        row.subtitle = 'Shortcut is not valid. Try a key with Super, Ctrl, Alt, or Shift.';
        return true;
      }

      settings.set_strv(KEYBINDING_NAME, [Gtk.accelerator_name(keyval, modifiers)]);
      stopRecording();
      return true;
    });
    shortcutButton.add_controller(controller);

    shortcutButton.connect('clicked', startRecording);
    clearButton.connect('clicked', () => {
      settings.set_strv(KEYBINDING_NAME, []);
      stopRecording();
    });
    resetButton.connect('clicked', () => {
      settings.reset(KEYBINDING_NAME);
      stopRecording();
    });

    row.add_suffix(shortcutButton);
    row.add_suffix(clearButton);
    row.add_suffix(resetButton);

    return row;
  }

  _currentShortcut(settings) {
    return settings.get_strv(KEYBINDING_NAME)[0] ?? '';
  }

  _shortcutLabel(settings) {
    const shortcut = this._currentShortcut(settings);
    if (!shortcut)
      return EMPTY_SHORTCUT_LABEL;

    const parsed = Gtk.accelerator_parse(shortcut);
    if (parsed[0] && Gtk.accelerator_valid(parsed[1], parsed[2]))
      return Gtk.accelerator_get_label(parsed[1], parsed[2]);

    return shortcut;
  }

  _shortcutSubtitle(settings) {
    const shortcut = this._currentShortcut(settings);
    if (!shortcut)
      return 'No shortcut is currently assigned.';

    const parsed = Gtk.accelerator_parse(shortcut);
    if (parsed[0] && Gtk.accelerator_valid(parsed[1], parsed[2]))
      return 'Click the button and press a new shortcut to change it.';

    return `Current: ${shortcut}`;
  }
}
