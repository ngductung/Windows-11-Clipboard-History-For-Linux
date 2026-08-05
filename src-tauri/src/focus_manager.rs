//! Focus Manager Module
//! Tracks and restores window focus for proper paste injection on X11.
//! Also provides X11 window activation using EWMH protocols.

use parking_lot::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::OnceLock;
use std::thread;
use std::time::{Duration, Instant};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{AtomEnum, ClientMessageEvent, ConnectionExt, EventMask, InputFocus};

/// Failure ceiling for focus restoration. Fast systems return after two
/// confirmed samples; constrained systems are allowed more time without
/// imposing that time on every paste.
const FOCUS_RESTORE_TIMEOUT: Duration = Duration::from_millis(750);

/// Stores the ID of the window that had focus before we opened
static LAST_FOCUSED_WINDOW: AtomicU32 = AtomicU32::new(0);
static LAST_TARGET_HINT: OnceLock<Mutex<Option<TargetWindowHint>>> = OnceLock::new();

#[derive(Debug, Clone, Default)]
struct TargetWindowHint {
    app_id: Option<String>,
    title: Option<String>,
}

fn target_hint() -> &'static Mutex<Option<TargetWindowHint>> {
    LAST_TARGET_HINT.get_or_init(|| Mutex::new(None))
}

pub fn save_focused_window() {
    if !crate::session::is_x11() {
        return;
    }

    *target_hint().lock() = None;

    // Never reuse a target captured for an older popup invocation if this
    // query fails. Pasting nowhere is safer than redirecting input to a stale
    // application window.
    LAST_FOCUSED_WINDOW.store(0, Ordering::SeqCst);

    match crate::paste_sync::focused_window() {
        Some(window_id) => {
            LAST_FOCUSED_WINDOW.store(window_id, Ordering::SeqCst);
            *target_hint().lock() = x11_window_hint(window_id);
            eprintln!("[FocusManager] Saved focused window: {}", window_id);
        }
        None => eprintln!("[FocusManager] Failed to query the focused X11 window"),
    }
}

pub fn clear_target_hint() {
    *target_hint().lock() = None;
}

pub fn save_target_hint(app_id: Option<String>, title: Option<String>) {
    let normalized = TargetWindowHint {
        app_id: app_id.and_then(non_empty),
        title: title.and_then(non_empty),
    };

    if normalized.app_id.is_some() || normalized.title.is_some() {
        eprintln!(
            "[FocusManager] Saved target hint: app_id={:?}, title={:?}",
            normalized.app_id, normalized.title
        );
        *target_hint().lock() = Some(normalized);
    }
}

pub fn target_matches_any_pattern(patterns: &[String]) -> bool {
    let hint = target_hint().lock().clone();
    let Some(hint) = hint else {
        return false;
    };

    patterns.iter().any(|pattern| {
        let pattern = pattern.trim().to_ascii_lowercase();
        if pattern.is_empty() {
            return false;
        }

        hint.app_id
            .as_deref()
            .is_some_and(|value| value.to_ascii_lowercase().contains(&pattern))
            || hint
                .title
                .as_deref()
                .is_some_and(|value| value.to_ascii_lowercase().contains(&pattern))
    })
}

pub fn restore_focused_window() -> Result<bool, String> {
    let window_id = LAST_FOCUSED_WINDOW.load(Ordering::SeqCst);

    if window_id == 0 {
        return Err("No previous window saved".to_string());
    }

    eprintln!("[FocusManager] Restoring focus to window: {}", window_id);

    crate::paste_sync::restore_and_settle_focus(window_id, FOCUS_RESTORE_TIMEOUT)
}

pub fn get_focused_window() -> Option<u32> {
    crate::paste_sync::focused_window()
}

fn non_empty(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn x11_window_hint(window_id: u32) -> Option<TargetWindowHint> {
    let (conn, _) = x11rb::connect(None).ok()?;
    let app_id = x11_window_class(&conn, window_id);
    let title = x11_window_title(&conn, window_id);

    if app_id.is_some() || title.is_some() {
        Some(TargetWindowHint { app_id, title })
    } else {
        None
    }
}

fn x11_window_class<C: Connection>(conn: &C, window_id: u32) -> Option<String> {
    let reply = conn
        .get_property(
            false,
            window_id,
            AtomEnum::WM_CLASS,
            AtomEnum::STRING,
            0,
            256,
        )
        .ok()?
        .reply()
        .ok()?;
    let raw = String::from_utf8(reply.value).ok()?;
    let class = raw
        .split('\0')
        .filter(|part| !part.trim().is_empty())
        .last()
        .or_else(|| raw.split('\0').find(|part| !part.trim().is_empty()))?;
    Some(class.to_string())
}

fn x11_window_title<C: Connection>(conn: &C, window_id: u32) -> Option<String> {
    let net_wm_name = conn
        .intern_atom(false, b"_NET_WM_NAME")
        .ok()?
        .reply()
        .ok()?
        .atom;
    let utf8_string = conn
        .intern_atom(false, b"UTF8_STRING")
        .ok()?
        .reply()
        .ok()?
        .atom;

    if let Ok(cookie) = conn.get_property(false, window_id, net_wm_name, utf8_string, 0, 256) {
        if let Ok(reply) = cookie.reply() {
            if let Ok(name) = String::from_utf8(reply.value) {
                if !name.trim().is_empty() {
                    return Some(name);
                }
            }
        }
    }

    conn.get_property(
        false,
        window_id,
        AtomEnum::WM_NAME,
        AtomEnum::STRING,
        0,
        256,
    )
    .ok()?
    .reply()
    .ok()
    .and_then(|reply| String::from_utf8(reply.value).ok())
    .and_then(non_empty)
}

// =============================================================================
// X11 Window Activation (EWMH compliant)
// =============================================================================

/// Maximum time to wait for window to be mapped
const WINDOW_MAP_TIMEOUT: Duration = Duration::from_millis(500);

/// Polling interval when waiting for window
const WINDOW_MAP_POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Activates an X11 window using the EWMH _NET_ACTIVE_WINDOW protocol.
/// This is the proper way to request focus and is respected by window managers
/// even with Focus Stealing Prevention enabled.
///
/// # Arguments
/// * `window_id` - The X11 window ID to activate
///
/// # Returns
/// * `Ok(())` if the activation message was sent successfully
/// * `Err(String)` if there was an error
pub fn x11_activate_window_by_id(window_id: u32) -> Result<(), String> {
    let (conn, screen_num) =
        x11rb::connect(None).map_err(|e| format!("X11 connect failed: {}", e))?;

    let screen = conn
        .setup()
        .roots
        .get(screen_num)
        .ok_or("Failed to get screen")?;
    let root = screen.root;

    // Get _NET_ACTIVE_WINDOW atom
    let net_active_window = conn
        .intern_atom(false, b"_NET_ACTIVE_WINDOW")
        .map_err(|e| format!("Failed to intern atom: {}", e))?
        .reply()
        .map_err(|e| format!("Failed to get atom reply: {}", e))?
        .atom;

    // Create the client message event
    // Data format for _NET_ACTIVE_WINDOW:
    // data[0] = source indication (1 = from application, 2 = from pager)
    // data[1] = timestamp (0 = current time)
    // data[2] = requestor's currently active window (0 if none)
    let event = ClientMessageEvent {
        response_type: 33, // ClientMessage
        format: 32,
        sequence: 0,
        window: window_id,
        type_: net_active_window,
        data: [1, 0, 0, 0, 0].into(), // source=1 (application request)
    };

    // Send to root window with SubstructureRedirect | SubstructureNotify
    conn.send_event(
        false,
        root,
        EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
        event,
    )
    .map_err(|e| format!("Failed to send event: {}", e))?;

    conn.flush()
        .map_err(|e| format!("Failed to flush: {}", e))?;

    eprintln!(
        "[FocusManager] Sent _NET_ACTIVE_WINDOW for window {}",
        window_id
    );
    Ok(())
}

/// Waits for a window with the given title to appear and be mapped.
/// Uses polling with timeout instead of a fixed sleep.
///
/// # Arguments
/// * `title` - The window title to search for (substring match)
/// * `timeout` - Maximum time to wait
///
/// # Returns
/// * `Some(window_id)` if found within timeout
/// * `None` if timeout exceeded
pub fn wait_for_window_by_title(title: &str, timeout: Duration) -> Option<u32> {
    let start = Instant::now();

    while start.elapsed() < timeout {
        if let Some(window_id) = find_window_by_title(title) {
            eprintln!(
                "[FocusManager] Found window '{}' with ID {} after {:?}",
                title,
                window_id,
                start.elapsed()
            );
            return Some(window_id);
        }
        thread::sleep(WINDOW_MAP_POLL_INTERVAL);
    }

    eprintln!("[FocusManager] Timeout waiting for window '{}'", title);
    None
}

/// Finds a window by its title using X11 primitives.
/// This is more reliable than xdotool as it directly queries the X server.
fn find_window_by_title(title: &str) -> Option<u32> {
    let (conn, screen_num) = x11rb::connect(None).ok()?;
    let screen = conn.setup().roots.get(screen_num)?;
    let root = screen.root;

    // Get atoms we need
    let net_client_list = conn
        .intern_atom(false, b"_NET_CLIENT_LIST")
        .ok()?
        .reply()
        .ok()?
        .atom;

    let net_wm_name = conn
        .intern_atom(false, b"_NET_WM_NAME")
        .ok()?
        .reply()
        .ok()?
        .atom;

    let utf8_string = conn
        .intern_atom(false, b"UTF8_STRING")
        .ok()?
        .reply()
        .ok()?
        .atom;

    // Get list of all client windows
    let client_list = conn
        .get_property(false, root, net_client_list, AtomEnum::WINDOW, 0, 1024)
        .ok()?
        .reply()
        .ok()?;

    let windows: Vec<u32> = client_list
        .value32()
        .map(|iter| iter.collect())
        .unwrap_or_default();

    // Search each window for matching title
    for window in windows {
        // Try _NET_WM_NAME first (UTF-8)
        if let Ok(cookie) = conn.get_property(false, window, net_wm_name, utf8_string, 0, 256) {
            if let Ok(reply) = cookie.reply() {
                if let Ok(name) = String::from_utf8(reply.value) {
                    if name.contains(title) {
                        return Some(window);
                    }
                }
            }
        }

        // Fall back to WM_NAME (legacy)
        if let Ok(cookie) =
            conn.get_property(false, window, AtomEnum::WM_NAME, AtomEnum::STRING, 0, 256)
        {
            if let Ok(reply) = cookie.reply() {
                if let Ok(name) = String::from_utf8(reply.value) {
                    if name.contains(title) {
                        return Some(window);
                    }
                }
            }
        }
    }

    None
}

/// High-level function to activate a window by title.
/// Waits for the window to appear, then activates it using EWMH.
///
/// # Arguments
/// * `title` - The window title to search for
///
/// # Returns
/// * `Ok(())` if activation was successful
/// * `Err(String)` if window not found or activation failed
pub fn x11_activate_window_by_title(title: &str) -> Result<(), String> {
    let window_id = wait_for_window_by_title(title, WINDOW_MAP_TIMEOUT)
        .ok_or_else(|| format!("Window '{}' not found within timeout", title))?;

    x11_activate_window_by_id(window_id)?;

    // Small delay to let the WM process the activation
    thread::sleep(Duration::from_millis(20));

    Ok(())
}

/// Alternative activation that sets input focus directly.
/// Use this as a fallback if _NET_ACTIVE_WINDOW doesn't work.
pub fn x11_force_input_focus(window_id: u32) -> Result<(), String> {
    let (conn, _) = x11rb::connect(None).map_err(|e| format!("X11 connect failed: {}", e))?;

    // Set input focus with PointerRoot revert mode
    conn.set_input_focus(InputFocus::POINTER_ROOT, window_id, x11rb::CURRENT_TIME)
        .map_err(|e| format!("set_input_focus failed: {}", e))?;

    conn.flush().map_err(|e| format!("Flush failed: {}", e))?;

    eprintln!("[FocusManager] Forced input focus to window {}", window_id);
    Ok(())
}

/// Combined activation strategy that tries multiple methods.
/// This is the most robust approach for X11 focus acquisition.
pub fn x11_robust_activate(title: &str) -> Result<(), String> {
    // Step 1: Wait for window to appear in _NET_CLIENT_LIST
    let window_id = wait_for_window_by_title(title, WINDOW_MAP_TIMEOUT)
        .ok_or_else(|| format!("Window '{}' not found", title))?;

    // Step 2: Try EWMH _NET_ACTIVE_WINDOW (preferred, WM-friendly)
    if let Err(e) = x11_activate_window_by_id(window_id) {
        eprintln!(
            "[FocusManager] EWMH activation failed: {}, trying fallback",
            e
        );
    }

    // Step 3: Small delay for WM to process
    thread::sleep(Duration::from_millis(30));

    // Step 4: Verify focus was acquired, force if not
    match get_focused_window() {
        Some(current_focus) => {
            if current_focus != window_id {
                eprintln!("[FocusManager] Focus not acquired, forcing input focus");
                x11_force_input_focus(window_id)?;
            }
        }
        None => {
            eprintln!(
                "[FocusManager] Could not determine focused window after EWMH activation; forcing input focus as fallback"
            );
            x11_force_input_focus(window_id)?;
        }
    }

    Ok(())
}
