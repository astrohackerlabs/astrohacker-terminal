use std::ffi::{c_void, CString};
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use crate::ffi::{self, TsWebContents};
use crate::proto::{self, Msg, TermSurfMessage};

// --- Tab registry ---

struct TabEntry {
    handle: TsWebContents,
    tab_id: i64,
    pane_id: String,
    inspected_tab_id: i64,
    last_url: String,
}

static PDF_INPUT_TRACE_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();
static LAST_LOADING_STATES: Mutex<Vec<TermSurfMessage>> = Mutex::new(Vec::new());
static LAST_NAVIGATION_STATES: Mutex<Vec<TermSurfMessage>> = Mutex::new(Vec::new());

struct DeferredHttpAuthCancel {
    wc: usize,
    request_id: u64,
}

unsafe extern "C" fn deferred_http_auth_cancel_task(data: *mut c_void) {
    let task = unsafe { Box::from_raw(data as *mut DeferredHttpAuthCancel) };
    let empty = CString::new("").unwrap();
    let ok = unsafe {
        ffi::ts_reply_http_auth(
            task.wc as TsWebContents,
            task.request_id,
            false,
            empty.as_ptr(),
            empty.as_ptr(),
        )
    };
    eprintln!(
        "[termsurf-http-auth] deferred-cancel request_id={} ok={}",
        task.request_id, ok
    );
}

fn defer_http_auth_cancel(wc: TsWebContents, request_id: u64) {
    let task = Box::new(DeferredHttpAuthCancel {
        wc: wc as usize,
        request_id,
    });
    unsafe {
        ffi::ts_post_task(
            Some(deferred_http_auth_cancel_task),
            Box::into_raw(task) as *mut c_void,
        );
    }
}

pub fn init_pdf_input_trace() {
    trace_pdf_input(format!("trace-init pid={}", std::process::id()));
}

fn trace_pdf_input(line: impl AsRef<str>) {
    let path = PDF_INPUT_TRACE_PATH.get_or_init(|| {
        if std::env::var_os("TERMSURF_PDF_INPUT_TRACE").is_none() {
            return None;
        }
        let path = std::env::var_os("TERMSURF_PDF_INPUT_TRACE_FILE")
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::temp_dir().join("termsurf").join("pdf-input.log"));
        if let Some(parent) = path.parent() {
            let _ = create_dir_all(parent);
        }
        Some(path)
    });

    let Some(path) = path else {
        return;
    };
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "chromium {}", line.as_ref());
    }
}

fn remember_loading_state(msg: &TermSurfMessage) {
    let Some(Msg::LoadingState(ref loading)) = msg.msg else {
        return;
    };
    let mut states = LAST_LOADING_STATES.lock().unwrap();
    match loading.state.as_str() {
        "loading" | "done" | "error" => {
            states.retain(|existing| {
                !matches!(
                    existing.msg,
                    Some(Msg::LoadingState(ref existing_loading)) if existing_loading.tab_id == loading.tab_id
                )
            });
            states.push(msg.clone());
        }
        "progress" => {
            states.retain(|existing| {
                !matches!(
                    existing.msg,
                    Some(Msg::LoadingState(ref existing_loading))
                        if existing_loading.tab_id == loading.tab_id
                            && existing_loading.state == "progress"
                )
            });
            states.push(msg.clone());
        }
        _ => states.push(msg.clone()),
    }
}

pub fn replay_state_to_client(stream: &mut UnixStream) {
    let states = LAST_LOADING_STATES.lock().unwrap().clone();
    for msg in states {
        if let Some(Msg::LoadingState(ref loading)) = msg.msg {
            trace_pdf_input(format!(
                "loading-state-replay tab={} state={} progress={}",
                loading.tab_id, loading.state, loading.progress
            ));
        }
        let _ = crate::ipc::write_message(stream, &msg);
    }
    let states = LAST_NAVIGATION_STATES.lock().unwrap().clone();
    for msg in states {
        let _ = crate::ipc::write_message(stream, &msg);
    }
}

/// Global tab registry. Only accessed from the UI thread (via ts_post_task
/// and callbacks), so no synchronization needed — same pattern as Plusium's
/// `static std::vector<TabEntry>* g_tabs`.
fn tabs() -> &'static mut Vec<TabEntry> {
    static mut TABS: Vec<TabEntry> = Vec::new();
    unsafe { &mut *std::ptr::addr_of_mut!(TABS) }
}

fn find_by_handle(wc: TsWebContents) -> Option<&'static mut TabEntry> {
    tabs()
        .iter_mut()
        .find(|t| !t.handle.is_null() && t.handle == wc)
}

fn find_by_tab_id(tab_id: i64) -> Option<&'static mut TabEntry> {
    tabs().iter_mut().find(|t| tab_id_matches(tab_id, t.tab_id))
}

fn tab_id_matches(requested: i64, owned: i64) -> bool {
    requested > 0 && owned > 0 && requested == owned
}

fn navigation_action_contract(m: &proto::termsurf::NavigationAction) -> bool {
    m.tab_id > 0
        && m.pane_id.is_empty()
        && matches!(m.action.as_str(), "back" | "forward" | "refresh")
}

// --- String-to-int mappings ---

const POINTER_MODIFIER_MASK: u64 = 1 | 2 | 4 | 8 | 64 | 128 | 256;
const SCROLL_MODIFIER_MASK: u64 = 1 | 2 | 4 | 8;
const SCROLL_PHASE_MASK: u64 = 1 | 2 | 4 | 8 | 16 | 32;

fn mouse_type(s: &str) -> Option<i32> {
    match s {
        "down" => Some(0),
        "up" => Some(1),
        _ => None,
    }
}

fn mouse_button(s: &str) -> Option<i32> {
    match s {
        "left" => Some(0),
        "right" => Some(1),
        "middle" => Some(2),
        _ => None,
    }
}

fn mouse_event_contract(m: &proto::termsurf::MouseEvent) -> bool {
    m.tab_id > 0
        && mouse_type(&m.r#type).is_some()
        && mouse_button(&m.button).is_some()
        && m.click_count > 0
        && m.click_count <= i32::MAX as i64
        && m.x.is_finite()
        && m.y.is_finite()
        && m.modifiers & !POINTER_MODIFIER_MASK == 0
}

fn mouse_move_contract(m: &proto::termsurf::MouseMove) -> bool {
    m.tab_id > 0 && m.x.is_finite() && m.y.is_finite() && m.modifiers & !POINTER_MODIFIER_MASK == 0
}

fn scroll_event_contract(m: &proto::termsurf::ScrollEvent) -> bool {
    m.tab_id > 0
        && m.x.is_finite()
        && m.y.is_finite()
        && m.delta_x.is_finite()
        && m.delta_y.is_finite()
        && m.phase & !SCROLL_PHASE_MASK == 0
        && m.momentum_phase & !SCROLL_PHASE_MASK == 0
        && m.modifiers & !SCROLL_MODIFIER_MASK == 0
        && (m.delta_x != 0.0 || m.delta_y != 0.0 || m.phase != 0 || m.momentum_phase != 0)
}

fn key_type(s: &str) -> Option<i32> {
    match s {
        "down" => Some(0),
        "up" => Some(1),
        "repeat" => Some(2),
        _ => None,
    }
}

fn key_event_contract(m: &proto::termsurf::KeyEvent) -> bool {
    m.tab_id > 0
        && key_type(&m.r#type).is_some()
        && m.windows_key_code > 0
        && m.windows_key_code <= i32::MAX as i64
        && m.utf8.is_empty()
        && m.modifiers & !0x0f == 0
}

fn text_input_type(s: &str) -> bool {
    matches!(
        s,
        "commit" | "ime_start" | "ime_update" | "ime_commit" | "ime_cancel"
    )
}

fn wire_range(start: i64, length: i64) -> bool {
    (start == -1 && length == -1) || (start >= 0 && length >= 0)
}

// --- Message dispatch ---

pub fn handle_message(msg: &TermSurfMessage) {
    let Some(ref inner) = msg.msg else { return };
    match inner {
        Msg::CreateTab(m) => {
            let url = CString::new(m.url.as_str()).unwrap();
            trace_pdf_input(format!(
                "create-tab pane={} pixel_width={} pixel_height={} url={} dark={}",
                m.pane_id, m.pixel_width, m.pixel_height, m.url, m.dark
            ));
            tabs().push(TabEntry {
                handle: std::ptr::null_mut(),
                tab_id: 0,
                pane_id: m.pane_id.clone(),
                inspected_tab_id: 0,
                last_url: String::new(),
            });
            let entry = tabs().last_mut().unwrap();
            entry.handle = unsafe {
                ffi::ts_create_web_contents(
                    crate::browser_context(),
                    url.as_ptr(),
                    m.pixel_width as i32,
                    m.pixel_height as i32,
                    m.dark,
                )
            };
        }
        Msg::CreateDevtoolsTab(m) => {
            trace_pdf_input(format!(
                "create-devtools-tab pane={} inspected_tab_id={} pixel_width={} pixel_height={} ffi=ts_create_devtools_web_contents",
                m.pane_id, m.inspected_tab_id, m.pixel_width, m.pixel_height
            ));
            tabs().push(TabEntry {
                handle: std::ptr::null_mut(),
                tab_id: 0,
                pane_id: m.pane_id.clone(),
                inspected_tab_id: m.inspected_tab_id,
                last_url: String::new(),
            });
            let entry = tabs().last_mut().unwrap();
            entry.handle = unsafe {
                ffi::ts_create_devtools_web_contents(
                    crate::browser_context(),
                    m.inspected_tab_id as i32,
                    m.pixel_width as i32,
                    m.pixel_height as i32,
                    m.dark,
                )
            };
        }
        Msg::Resize(m) => {
            if let Some(t) = find_by_tab_id(m.tab_id) {
                trace_pdf_input(format!(
                    "resize tab_id={} pane_id={} pixel_width={} pixel_height={} screen_x={} screen_y={} screen_width={} screen_height={} screen_scale={} ffi=ts_set_view_size",
                    m.tab_id,
                    t.pane_id,
                    m.pixel_width,
                    m.pixel_height,
                    m.screen_x,
                    m.screen_y,
                    m.screen_width,
                    m.screen_height,
                    m.screen_scale
                ));
                unsafe {
                    ffi::ts_set_view_size(
                        t.handle,
                        m.pixel_width as i32,
                        m.pixel_height as i32,
                        m.screen_x,
                        m.screen_y,
                        m.screen_width,
                        m.screen_height,
                        m.screen_scale,
                    );
                }
            } else {
                trace_pdf_input(format!(
                    "resize tab_id={} result=no-tab pixel_width={} pixel_height={}",
                    m.tab_id, m.pixel_width, m.pixel_height
                ));
            }
        }
        Msg::CloseTab(m) => {
            let tab_id = m.tab_id;
            if let Some(t) = find_by_tab_id(tab_id) {
                trace_pdf_input(format!(
                    "close-tab tab_id={} pane_id={} result=destroying ffi=ts_destroy_web_contents",
                    tab_id, t.pane_id
                ));
                unsafe { ffi::ts_destroy_web_contents(t.handle) };
            } else {
                trace_pdf_input(format!("close-tab tab_id={} result=no-tab", tab_id));
            }
            tabs().retain(|t| t.tab_id != tab_id);
            trace_pdf_input(format!("close-tab tab_id={} result=removed", tab_id));
            if tabs().is_empty() {
                trace_pdf_input(
                    "close-tab result=no-tabs-remaining ffi=ts_destroy_browser_context".to_string(),
                );
                unsafe { ffi::ts_destroy_browser_context(crate::browser_context()) };
                trace_pdf_input("close-tab result=no-tabs-remaining ffi=ts_quit".to_string());
                unsafe { ffi::ts_quit() };
                trace_pdf_input("close-tab result=no-tabs-remaining process-exit".to_string());
                std::process::exit(0);
            }
        }
        Msg::Navigate(m) => {
            if let Some(t) = find_by_tab_id(m.tab_id) {
                let url = CString::new(m.url.as_str()).unwrap();
                trace_pdf_input(format!(
                    "navigate tab={} pane={} url={} ffi=ts_load_url",
                    m.tab_id, t.pane_id, m.url
                ));
                unsafe { ffi::ts_load_url(t.handle, url.as_ptr()) };
            }
        }
        Msg::NavigationAction(m) => {
            if !navigation_action_contract(m) {
                trace_pdf_input(format!(
                    "navigation-action tab={} pane={} action={} result=invalid-contract",
                    m.tab_id, m.pane_id, m.action
                ));
                return;
            }
            if let Some(t) = find_by_tab_id(m.tab_id) {
                let action = CString::new(m.action.as_str()).unwrap();
                let accepted = unsafe { ffi::ts_navigation_action(t.handle, action.as_ptr()) };
                trace_pdf_input(format!(
                    "navigation-action tab={} pane={} action={} accepted={}",
                    m.tab_id, t.pane_id, m.action, accepted
                ));
            } else {
                trace_pdf_input(format!(
                    "navigation-action tab={} action={} result=no-tab",
                    m.tab_id, m.action
                ));
            }
        }
        Msg::MouseEvent(m) => {
            if !mouse_event_contract(&m) {
                trace_pdf_input(format!(
                    "mouse-event tab={} result=invalid-contract type={} button={} coords=({:?}, {:?}) click_count={} modifiers={}",
                    m.tab_id, m.r#type, m.button, m.x, m.y, m.click_count, m.modifiers
                ));
                return;
            }
            if let Some(t) = find_by_tab_id(m.tab_id) {
                trace_pdf_input(format!(
                    "mouse-event tab={} pane={} ffi=ts_forward_mouse_event type={} button={} coords=({:.2}, {:.2}) click_count={} modifiers={}",
                    m.tab_id,
                    t.pane_id,
                    m.r#type,
                    m.button,
                    m.x,
                    m.y,
                    m.click_count,
                    m.modifiers
                ));
                unsafe {
                    ffi::ts_forward_mouse_event(
                        t.handle,
                        mouse_type(&m.r#type).expect("validated mouse type"),
                        mouse_button(&m.button).expect("validated mouse button"),
                        m.x as i32,
                        m.y as i32,
                        m.click_count as i32,
                        m.modifiers as i32,
                    );
                }
            } else {
                trace_pdf_input(format!(
                    "mouse-event tab={} result=no-tab type={} button={} coords=({:.2}, {:.2}) click_count={} modifiers={}",
                    m.tab_id,
                    m.r#type,
                    m.button,
                    m.x,
                    m.y,
                    m.click_count,
                    m.modifiers
                ));
            }
        }
        Msg::MouseMove(m) => {
            if !mouse_move_contract(&m) {
                trace_pdf_input(format!(
                    "mouse-move tab={} result=invalid-contract coords=({:?}, {:?}) modifiers={}",
                    m.tab_id, m.x, m.y, m.modifiers
                ));
                return;
            }
            if let Some(t) = find_by_tab_id(m.tab_id) {
                trace_pdf_input(format!(
                    "mouse-move tab={} pane={} ffi=ts_forward_mouse_move coords=({:.2}, {:.2}) modifiers={}",
                    m.tab_id, t.pane_id, m.x, m.y, m.modifiers
                ));
                unsafe {
                    ffi::ts_forward_mouse_move(
                        t.handle,
                        m.x as i32,
                        m.y as i32,
                        m.modifiers as i32,
                    );
                }
            } else {
                trace_pdf_input(format!(
                    "mouse-move tab={} result=no-tab coords=({:.2}, {:.2}) modifiers={}",
                    m.tab_id, m.x, m.y, m.modifiers
                ));
            }
        }
        Msg::ScrollEvent(m) => {
            if !scroll_event_contract(&m) {
                trace_pdf_input(format!(
                    "scroll-event tab={} result=invalid-contract coords=({:?}, {:?}) delta=({:?}, {:?}) phase={} momentum_phase={} precise={} modifiers={}",
                    m.tab_id, m.x, m.y, m.delta_x, m.delta_y, m.phase, m.momentum_phase, m.precise, m.modifiers
                ));
                return;
            }
            if let Some(t) = find_by_tab_id(m.tab_id) {
                trace_pdf_input(format!(
                    "scroll-event tab={} pane={} ffi=ts_forward_scroll_event coords=({:.2}, {:.2}) delta=({:.2}, {:.2}) phase={} momentum_phase={} precise={} modifiers={}",
                    m.tab_id,
                    t.pane_id,
                    m.x,
                    m.y,
                    m.delta_x,
                    m.delta_y,
                    m.phase,
                    m.momentum_phase,
                    m.precise,
                    m.modifiers
                ));
                unsafe {
                    ffi::ts_forward_scroll_event(
                        t.handle,
                        m.x as i32,
                        m.y as i32,
                        m.delta_x as f32,
                        m.delta_y as f32,
                        m.phase as i32,
                        m.momentum_phase as i32,
                        m.precise,
                        m.modifiers as i32,
                    );
                }
            } else {
                trace_pdf_input(format!(
                    "scroll-event tab={} result=no-tab coords=({:.2}, {:.2}) delta=({:.2}, {:.2})",
                    m.tab_id, m.x, m.y, m.delta_x, m.delta_y
                ));
            }
        }
        Msg::KeyEvent(m) => {
            if !key_event_contract(m) {
                trace_pdf_input(format!(
                    "key-event tab={} result=invalid-contract type={} windows_key_code={} utf8_len={} modifiers={}",
                    m.tab_id,
                    m.r#type,
                    m.windows_key_code,
                    m.utf8.len(),
                    m.modifiers
                ));
                return;
            }
            if let Some(t) = find_by_tab_id(m.tab_id) {
                let key_type = key_type(&m.r#type).expect("validated key type");
                trace_pdf_input(format!(
                    "key-event tab={} pane={} ffi=ts_forward_key_event type={} windows_key_code={} utf8_len={} modifiers={}",
                    m.tab_id,
                    t.pane_id,
                    m.r#type,
                    m.windows_key_code,
                    m.utf8.len(),
                    m.modifiers
                ));
                let utf8 = CString::new(m.utf8.as_str()).unwrap();
                unsafe {
                    ffi::ts_forward_key_event(
                        t.handle,
                        key_type,
                        m.windows_key_code as i32,
                        utf8.as_ptr(),
                        m.modifiers as i32,
                    );
                }
            } else {
                trace_pdf_input(format!(
                    "key-event tab={} result=no-tab type={} windows_key_code={} utf8_len={} modifiers={}",
                    m.tab_id,
                    m.r#type,
                    m.windows_key_code,
                    m.utf8.len(),
                    m.modifiers
                ));
            }
        }
        Msg::TextInput(m) => {
            if !text_input_type(&m.r#type)
                || !wire_range(m.selected_start, m.selected_length)
                || !wire_range(m.replacement_start, m.replacement_length)
            {
                trace_pdf_input(format!(
                    "text-input tab={} type={} result=invalid-contract",
                    m.tab_id, m.r#type
                ));
                return;
            }
            trace_pdf_input(format!(
                "text-input tab={} type={} utf8_len={} result=unsupported-explicit",
                m.tab_id,
                m.r#type,
                m.text.len()
            ));
        }
        Msg::FocusChanged(m) => {
            if let Some(t) = find_by_tab_id(m.tab_id) {
                trace_pdf_input(format!(
                    "focus-changed tab={} pane={} ffi=ts_set_focus focused={}",
                    m.tab_id, t.pane_id, m.focused
                ));
                unsafe { ffi::ts_set_focus(t.handle, m.focused) };
            } else {
                trace_pdf_input(format!(
                    "focus-changed tab={} result=no-tab focused={}",
                    m.tab_id, m.focused
                ));
            }
        }
        Msg::SetGuiActive(m) => {
            let reason =
                CString::new(m.reason.as_str()).unwrap_or_else(|_| CString::new("").unwrap());
            if m.tab_id == 0 {
                let mut target_count = 0;
                for t in tabs().iter() {
                    if !t.handle.is_null() {
                        target_count += 1;
                        unsafe { ffi::ts_set_gui_active(t.handle, m.active, reason.as_ptr()) };
                    }
                }
                trace_pdf_input(format!(
                    "set-gui-active tab=0 active={} reason={} target_count={}",
                    m.active, m.reason, target_count
                ));
            } else if let Some(t) = find_by_tab_id(m.tab_id) {
                if !t.handle.is_null() {
                    trace_pdf_input(format!(
                        "set-gui-active tab={} pane={} active={} reason={} target_count=1",
                        m.tab_id, t.pane_id, m.active, m.reason
                    ));
                    unsafe { ffi::ts_set_gui_active(t.handle, m.active, reason.as_ptr()) };
                }
            } else {
                trace_pdf_input(format!(
                    "set-gui-active tab={} active={} reason={} result=no-tab",
                    m.tab_id, m.active, m.reason
                ));
            }
        }
        Msg::JavascriptDialogReply(m) => {
            if let Some(t) = find_by_tab_id(m.tab_id) {
                let prompt_text = CString::new(m.prompt_text.as_str())
                    .unwrap_or_else(|_| CString::new("").unwrap());
                let ok = unsafe {
                    ffi::ts_reply_javascript_dialog(
                        t.handle,
                        m.request_id,
                        m.accepted,
                        prompt_text.as_ptr(),
                    )
                };
                eprintln!(
                    "[termsurf-js-dialog] reply tab_id={} request_id={} accepted={} ok={}",
                    m.tab_id, m.request_id, m.accepted, ok
                );
                trace_pdf_input(format!(
                    "javascript-dialog-reply tab={} pane={} request_id={} accepted={} ok={}",
                    m.tab_id, t.pane_id, m.request_id, m.accepted, ok
                ));
            } else {
                eprintln!(
                    "[termsurf-js-dialog] reply-missing-tab tab_id={} request_id={}",
                    m.tab_id, m.request_id
                );
            }
        }
        Msg::HttpAuthReply(m) => {
            if let Some(t) = find_by_tab_id(m.tab_id) {
                let username =
                    CString::new(m.username.as_str()).unwrap_or_else(|_| CString::new("").unwrap());
                let password =
                    CString::new(m.password.as_str()).unwrap_or_else(|_| CString::new("").unwrap());
                let ok = unsafe {
                    ffi::ts_reply_http_auth(
                        t.handle,
                        m.request_id,
                        m.accepted,
                        username.as_ptr(),
                        password.as_ptr(),
                    )
                };
                eprintln!(
                    "[termsurf-http-auth] reply tab_id={} request_id={} accepted={} ok={}",
                    m.tab_id, m.request_id, m.accepted, ok
                );
                trace_pdf_input(format!(
                    "http-auth-reply tab={} pane={} request_id={} accepted={} username={} password_len={} ok={}",
                    m.tab_id,
                    t.pane_id,
                    m.request_id,
                    m.accepted,
                    m.username,
                    m.password.chars().count(),
                    ok
                ));
            } else {
                eprintln!(
                    "[termsurf-http-auth] reply-missing-tab tab_id={} request_id={}",
                    m.tab_id, m.request_id
                );
            }
        }
        Msg::SetColorScheme(m) => {
            if let Some(t) = find_by_tab_id(m.tab_id) {
                trace_pdf_input(format!(
                    "set-color-scheme tab={} pane={} dark={} ffi=ts_set_color_scheme",
                    m.tab_id, t.pane_id, m.dark
                ));
                unsafe { ffi::ts_set_color_scheme(t.handle, m.dark) };
            } else {
                trace_pdf_input(format!(
                    "set-color-scheme tab={} result=missing-tab dark={}",
                    m.tab_id, m.dark
                ));
            }
        }
        Msg::QueryTabsRequest(_) => {
            let mut browser_count: i64 = 0;
            let mut devtools_count: i64 = 0;
            let mut tab_infos = Vec::new();
            for t in tabs().iter() {
                if t.inspected_tab_id > 0 {
                    devtools_count += 1;
                } else {
                    browser_count += 1;
                }
                tab_infos.push(proto::termsurf::TabInfo {
                    id: t.tab_id,
                    inspected_tab_id: t.inspected_tab_id,
                    pane_id: t.pane_id.clone(),
                    url: t.last_url.clone(),
                });
            }
            let reply = TermSurfMessage {
                msg: Some(Msg::QueryTabsReply(proto::termsurf::QueryTabsReply {
                    chromium_tabs: tabs().len() as i64,
                    chromium_browser: browser_count,
                    chromium_devtools: devtools_count,
                    tabs: tab_infos,
                    gui_panes: 0,
                    error: String::new(),
                })),
            };
            crate::ipc::send(&reply);
        }
        _ => {}
    }
}

#[cfg(test)]
mod text_input_contract_tests {
    use super::{
        key_event_contract, key_type, mouse_button, mouse_event_contract, mouse_move_contract,
        mouse_type, navigation_action_contract, scroll_event_contract, tab_id_matches,
        text_input_type, wire_range,
    };
    use crate::proto::termsurf;

    include!("../../key_contract_tests.rs");

    #[test]
    fn pointer_contract_is_exhaustive_and_finite() {
        assert_eq!(mouse_type("down"), Some(0));
        assert_eq!(mouse_type("up"), Some(1));
        for invalid in ["", "press", "click", "Down"] {
            assert_eq!(mouse_type(invalid), None);
        }
        assert_eq!(mouse_button("left"), Some(0));
        assert_eq!(mouse_button("right"), Some(1));
        assert_eq!(mouse_button("middle"), Some(2));
        for invalid in ["", "primary", "unknown", "Left"] {
            assert_eq!(mouse_button(invalid), None);
        }
        assert!(tab_id_matches(41, 41));
        assert!(tab_id_matches(73, 73));
        for (requested, owned) in [(0, 0), (0, 41), (-1, -1), (41, 73), (99, 41)] {
            assert!(!tab_id_matches(requested, owned));
        }

        let base_button = termsurf::MouseEvent {
            tab_id: 1,
            r#type: "down".into(),
            button: "left".into(),
            x: 10.5,
            y: 20.5,
            click_count: 1,
            modifiers: 0,
        };
        for event_type in ["down", "up"] {
            for button in ["left", "middle", "right"] {
                for modifiers in [0, 1, 2, 4, 8, 64, 128, 256, 1 | 2 | 4 | 8 | 64 | 128 | 256] {
                    let mut event = base_button.clone();
                    event.r#type = event_type.into();
                    event.button = button.into();
                    event.modifiers = modifiers;
                    assert!(mouse_event_contract(&event));
                }
            }
        }
        let mut event = base_button.clone();
        event.click_count = i32::MAX as i64;
        assert!(mouse_event_contract(&event));
        for invalid in ["", "press", "click", "Down"] {
            let mut event = base_button.clone();
            event.r#type = invalid.into();
            assert!(!mouse_event_contract(&event));
        }
        for invalid in ["", "primary", "unknown", "Left"] {
            let mut event = base_button.clone();
            event.button = invalid.into();
            assert!(!mouse_event_contract(&event));
        }
        for invalid in [i64::MIN, -1, 0, i32::MAX as i64 + 1, i64::MAX] {
            let mut event = base_button.clone();
            event.click_count = invalid;
            assert!(!mouse_event_contract(&event));
        }
        for invalid in [0, -1] {
            let mut event = base_button.clone();
            event.tab_id = invalid;
            assert!(!mouse_event_contract(&event));
        }
        for invalid in [16, 32, 512, u64::MAX] {
            let mut event = base_button.clone();
            event.modifiers = invalid;
            assert!(!mouse_event_contract(&event));
        }
        for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut event = base_button.clone();
            event.x = invalid;
            assert!(!mouse_event_contract(&event));
            let mut event = base_button.clone();
            event.y = invalid;
            assert!(!mouse_event_contract(&event));
        }

        let base_movement = termsurf::MouseMove {
            tab_id: 1,
            x: -12.0,
            y: 20.0,
            modifiers: 0,
        };
        for modifiers in [0, 1, 2, 4, 8, 64, 128, 256, 1 | 2 | 4 | 8 | 64 | 128 | 256] {
            let mut movement = base_movement.clone();
            movement.modifiers = modifiers;
            assert!(mouse_move_contract(&movement));
        }
        for invalid in [16, 32, 512, u64::MAX] {
            let mut movement = base_movement.clone();
            movement.modifiers = invalid;
            assert!(!mouse_move_contract(&movement));
        }
        for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut movement = base_movement.clone();
            movement.x = invalid;
            assert!(!mouse_move_contract(&movement));
            let mut movement = base_movement.clone();
            movement.y = invalid;
            assert!(!mouse_move_contract(&movement));
        }
        let mut movement = base_movement.clone();
        movement.tab_id = 0;
        assert!(!mouse_move_contract(&movement));

        let base_scroll = termsurf::ScrollEvent {
            tab_id: 1,
            x: 300.0,
            y: 170.0,
            delta_x: 0.0,
            delta_y: -12.5,
            phase: 0,
            momentum_phase: 0,
            precise: true,
            modifiers: 0,
        };
        for precise in [false, true] {
            for modifiers in [0, 1, 2, 4, 8, 1 | 2 | 4 | 8] {
                let mut scroll = base_scroll.clone();
                scroll.precise = precise;
                scroll.modifiers = modifiers;
                assert!(scroll_event_contract(&scroll));
            }
        }
        for phase in [1, 2, 4, 8, 16, 32, 1 | 2 | 4 | 8 | 16 | 32] {
            let mut scroll = base_scroll.clone();
            scroll.delta_y = 0.0;
            scroll.phase = phase;
            assert!(scroll_event_contract(&scroll));
            scroll.phase = 0;
            scroll.momentum_phase = phase;
            assert!(scroll_event_contract(&scroll));
        }
        let mut scroll = base_scroll.clone();
        scroll.delta_y = 0.0;
        scroll.delta_x = 0.5;
        assert!(scroll_event_contract(&scroll));
        scroll.delta_x = 0.0;
        assert!(!scroll_event_contract(&scroll));
        for invalid in [64, u64::MAX] {
            let mut scroll = base_scroll.clone();
            scroll.phase = invalid;
            assert!(!scroll_event_contract(&scroll));
            let mut scroll = base_scroll.clone();
            scroll.momentum_phase = invalid;
            assert!(!scroll_event_contract(&scroll));
        }
        for invalid in [16, 32, 64, 128, 256, 512, u64::MAX] {
            let mut scroll = base_scroll.clone();
            scroll.modifiers = invalid;
            assert!(!scroll_event_contract(&scroll));
        }
        for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for field in 0..4 {
                let mut scroll = base_scroll.clone();
                match field {
                    0 => scroll.x = invalid,
                    1 => scroll.y = invalid,
                    2 => scroll.delta_x = invalid,
                    _ => scroll.delta_y = invalid,
                }
                assert!(!scroll_event_contract(&scroll));
            }
        }
        let mut scroll = base_scroll;
        scroll.tab_id = -1;
        assert!(!scroll_event_contract(&scroll));
    }

    #[test]
    fn key_types_are_exhaustive() {
        assert_eq!(key_type("down"), Some(0));
        assert_eq!(key_type("up"), Some(1));
        assert_eq!(key_type("repeat"), Some(2));
        assert_eq!(key_type("insert"), None);
        assert_eq!(key_type("nav-back"), None);
    }

    #[test]
    fn text_types_and_ranges_are_exhaustive() {
        for kind in [
            "commit",
            "ime_start",
            "ime_update",
            "ime_commit",
            "ime_cancel",
        ] {
            assert!(text_input_type(kind));
        }
        assert!(!text_input_type("compositionupdate"));
        assert!(wire_range(-1, -1));
        assert!(wire_range(0, 0));
        assert!(wire_range(7, 3));
        assert!(!wire_range(-1, 0));
        assert!(!wire_range(0, -1));
    }

    #[test]
    fn navigation_actions_require_engine_identity_and_closed_values() {
        for action in ["back", "forward", "refresh"] {
            assert!(navigation_action_contract(&termsurf::NavigationAction {
                tab_id: 7,
                pane_id: String::new(),
                action: action.into(),
            }));
        }
        for action in ["", "reload", "Back"] {
            assert!(!navigation_action_contract(&termsurf::NavigationAction {
                tab_id: 7,
                pane_id: String::new(),
                action: action.into(),
            }));
        }
        assert!(!navigation_action_contract(&termsurf::NavigationAction {
            tab_id: 0,
            pane_id: String::new(),
            action: "back".into(),
        }));
        assert!(!navigation_action_contract(&termsurf::NavigationAction {
            tab_id: 7,
            pane_id: "pane-a".into(),
            action: "back".into(),
        }));
    }
}

// --- Callbacks (called on UI thread) ---

pub unsafe extern "C" fn on_tab_ready(wc: TsWebContents, tab_id: i32, _user_data: *mut c_void) {
    // Try by handle first, then by null handle (sync callback).
    let t = find_by_handle(wc).or_else(|| {
        tabs().iter_mut().find(|t| t.handle.is_null()).map(|t| {
            t.handle = wc;
            t
        })
    });
    let Some(t) = t else { return };
    t.tab_id = tab_id as i64;
    trace_pdf_input(format!(
        "tab-ready tab={} pane={} inspected_tab_id={}",
        t.tab_id, t.pane_id, t.inspected_tab_id
    ));

    let msg = TermSurfMessage {
        msg: Some(Msg::TabReady(proto::termsurf::TabReady {
            pane_id: t.pane_id.clone(),
            tab_id: tab_id as i64,
        })),
    };
    crate::ipc::send(&msg);
}

pub unsafe extern "C" fn on_ca_context_id(
    wc: TsWebContents,
    ca_context_id: u32,
    width: i32,
    height: i32,
    _user_data: *mut c_void,
) {
    let Some(t) = find_by_handle(wc) else { return };
    trace_pdf_input(format!(
        "ca-context tab={} pane={} inspected_tab_id={} context_id={} pixel_width={} pixel_height={}",
        t.tab_id, t.pane_id, t.inspected_tab_id, ca_context_id, width, height
    ));
    let msg = TermSurfMessage {
        msg: Some(Msg::CaContext(proto::termsurf::CaContext {
            tab_id: t.tab_id,
            ca_context_id: ca_context_id as u64,
            pixel_width: width as u64,
            pixel_height: height as u64,
        })),
    };
    crate::ipc::send(&msg);
}

pub unsafe extern "C" fn on_url_changed(
    wc: TsWebContents,
    url: *const std::os::raw::c_char,
    _user_data: *mut c_void,
) {
    let Some(t) = find_by_handle(wc) else { return };
    let url_str = unsafe { std::ffi::CStr::from_ptr(url) }
        .to_string_lossy()
        .into_owned();
    t.last_url = url_str.clone();
    trace_pdf_input(format!(
        "url-changed tab={} pane={} url={}",
        t.tab_id, t.pane_id, url_str
    ));
    let msg = TermSurfMessage {
        msg: Some(Msg::UrlChanged(proto::termsurf::UrlChanged {
            tab_id: t.tab_id,
            url: url_str,
        })),
    };
    crate::ipc::send(&msg);
}

pub unsafe extern "C" fn on_loading_state(
    wc: TsWebContents,
    state: *const std::os::raw::c_char,
    progress: i32,
    _user_data: *mut c_void,
) {
    let state_str = unsafe { std::ffi::CStr::from_ptr(state) }
        .to_string_lossy()
        .into_owned();
    let Some(t) = find_by_handle(wc) else {
        let pending_null_handle = tabs().iter().any(|t| t.handle.is_null());
        trace_pdf_input(format!(
            "loading-state-callback-missing-tab handle={:p} pending_null_handle={} state={} progress={}",
            wc, pending_null_handle, state_str, progress
        ));
        return;
    };
    trace_pdf_input(format!(
        "loading-state-callback tab={} pane={} state={} progress={}",
        t.tab_id, t.pane_id, state_str, progress
    ));
    let msg = TermSurfMessage {
        msg: Some(Msg::LoadingState(proto::termsurf::LoadingState {
            tab_id: t.tab_id,
            state: state_str,
            progress: progress as u64,
        })),
    };
    remember_loading_state(&msg);
    crate::ipc::send(&msg);
}

pub unsafe extern "C" fn on_navigation_state(
    wc: TsWebContents,
    can_go_back: bool,
    can_go_forward: bool,
    can_refresh: bool,
    _user_data: *mut c_void,
) {
    let Some(t) = find_by_handle(wc) else { return };
    let msg = TermSurfMessage {
        msg: Some(Msg::NavigationState(proto::termsurf::NavigationState {
            tab_id: t.tab_id,
            can_go_back,
            can_go_forward,
            can_refresh,
        })),
    };
    let mut states = LAST_NAVIGATION_STATES.lock().unwrap();
    states.retain(|existing| {
        !matches!(existing.msg, Some(Msg::NavigationState(ref state)) if state.tab_id == t.tab_id)
    });
    states.push(msg.clone());
    drop(states);
    crate::ipc::send(&msg);
}

pub unsafe extern "C" fn on_renderer_crashed(
    wc: TsWebContents,
    termination_status: *const std::os::raw::c_char,
    termination_status_code: i32,
    url: *const std::os::raw::c_char,
    can_reload: bool,
    _user_data: *mut c_void,
) {
    let Some(t) = find_by_handle(wc) else { return };
    let termination_status = unsafe { std::ffi::CStr::from_ptr(termination_status) }
        .to_string_lossy()
        .into_owned();
    let url = unsafe { std::ffi::CStr::from_ptr(url) }
        .to_string_lossy()
        .into_owned();
    eprintln!(
        "[termsurf-renderer-crash] tab_id={} status={} code={} url={} can_reload={}",
        t.tab_id, termination_status, termination_status_code, url, can_reload
    );
    trace_pdf_input(format!(
        "renderer-crashed tab={} pane={} status={} code={} url={} can_reload={}",
        t.tab_id, t.pane_id, termination_status, termination_status_code, url, can_reload
    ));
    let msg = TermSurfMessage {
        msg: Some(Msg::RendererCrashed(proto::termsurf::RendererCrashed {
            tab_id: t.tab_id,
            termination_status,
            termination_status_code,
            url,
            can_reload,
        })),
    };
    crate::ipc::send(&msg);
}

pub unsafe extern "C" fn on_title_changed(
    wc: TsWebContents,
    title: *const std::os::raw::c_char,
    _user_data: *mut c_void,
) {
    let Some(t) = find_by_handle(wc) else { return };
    let title_str = unsafe { std::ffi::CStr::from_ptr(title) }
        .to_string_lossy()
        .into_owned();
    trace_pdf_input(format!(
        "title-changed tab={} pane={} title={}",
        t.tab_id, t.pane_id, title_str
    ));
    let msg = TermSurfMessage {
        msg: Some(Msg::TitleChanged(proto::termsurf::TitleChanged {
            tab_id: t.tab_id,
            title: title_str,
        })),
    };
    crate::ipc::send(&msg);
}

pub unsafe extern "C" fn on_cursor_changed(
    wc: TsWebContents,
    cursor_type: i32,
    _user_data: *mut c_void,
) {
    let Some(t) = find_by_handle(wc) else { return };
    let msg = TermSurfMessage {
        msg: Some(Msg::CursorChanged(proto::termsurf::CursorChanged {
            tab_id: t.tab_id,
            cursor_type: cursor_type as i64,
        })),
    };
    crate::ipc::send(&msg);
}

pub unsafe extern "C" fn on_target_url_changed(
    wc: TsWebContents,
    url: *const std::os::raw::c_char,
    _user_data: *mut c_void,
) {
    let Some(t) = find_by_handle(wc) else { return };
    let url_str = unsafe { std::ffi::CStr::from_ptr(url) }
        .to_string_lossy()
        .into_owned();
    let msg = TermSurfMessage {
        msg: Some(Msg::TargetUrlChanged(proto::termsurf::TargetUrlChanged {
            tab_id: t.tab_id,
            url: url_str,
        })),
    };
    crate::ipc::send(&msg);
}

pub unsafe extern "C" fn on_javascript_dialog_request(
    wc: TsWebContents,
    request_id: u64,
    dialog_type: *const std::os::raw::c_char,
    origin_url: *const std::os::raw::c_char,
    message: *const std::os::raw::c_char,
    default_prompt_text: *const std::os::raw::c_char,
    _user_data: *mut c_void,
) {
    let Some(t) = find_by_handle(wc) else {
        eprintln!(
            "[termsurf-js-dialog] request-missing-tab request_id={}",
            request_id
        );
        return;
    };
    let dialog_type = unsafe { std::ffi::CStr::from_ptr(dialog_type) }
        .to_string_lossy()
        .into_owned();
    let origin_url = unsafe { std::ffi::CStr::from_ptr(origin_url) }
        .to_string_lossy()
        .into_owned();
    let message = unsafe { std::ffi::CStr::from_ptr(message) }
        .to_string_lossy()
        .into_owned();
    let default_prompt_text = unsafe { std::ffi::CStr::from_ptr(default_prompt_text) }
        .to_string_lossy()
        .into_owned();
    eprintln!(
        "[termsurf-js-dialog] request tab_id={} request_id={} type={} origin={}",
        t.tab_id, request_id, dialog_type, origin_url
    );
    trace_pdf_input(format!(
        "javascript-dialog-request tab={} pane={} request_id={} type={} origin={} message={}",
        t.tab_id, t.pane_id, request_id, dialog_type, origin_url, message
    ));
    let msg = TermSurfMessage {
        msg: Some(Msg::JavascriptDialogRequest(
            proto::termsurf::JavaScriptDialogRequest {
                tab_id: t.tab_id,
                request_id,
                dialog_type,
                origin_url,
                message,
                default_prompt_text,
            },
        )),
    };
    crate::ipc::send(&msg);
}

pub unsafe extern "C" fn on_console_message(
    wc: TsWebContents,
    level: *const std::os::raw::c_char,
    message: *const std::os::raw::c_char,
    line_no: i32,
    source_id: *const std::os::raw::c_char,
    _user_data: *mut c_void,
) {
    let Some(t) = find_by_handle(wc) else {
        eprintln!("[termsurf-console] message-missing-tab line_no={}", line_no);
        return;
    };
    let level = unsafe { std::ffi::CStr::from_ptr(level) }
        .to_string_lossy()
        .into_owned();
    let message = unsafe { std::ffi::CStr::from_ptr(message) }
        .to_string_lossy()
        .into_owned();
    let source_id = unsafe { std::ffi::CStr::from_ptr(source_id) }
        .to_string_lossy()
        .into_owned();
    eprintln!(
        "[termsurf-console] message tab_id={} level={} line_no={} source={}",
        t.tab_id, level, line_no, source_id
    );
    let msg = TermSurfMessage {
        msg: Some(Msg::ConsoleMessage(proto::termsurf::ConsoleMessage {
            tab_id: t.tab_id,
            level,
            message,
            line_no,
            source_id,
        })),
    };
    crate::ipc::send(&msg);
}

pub unsafe extern "C" fn on_http_auth_request(
    wc: TsWebContents,
    request_id: u64,
    url: *const std::os::raw::c_char,
    auth_scheme: *const std::os::raw::c_char,
    challenger: *const std::os::raw::c_char,
    realm: *const std::os::raw::c_char,
    is_proxy: bool,
    first_auth_attempt: bool,
    is_primary_main_frame_navigation: bool,
    is_navigation: bool,
    _user_data: *mut c_void,
) {
    let Some(t) = find_by_handle(wc) else {
        eprintln!(
            "[termsurf-http-auth] request-missing-tab request_id={}",
            request_id
        );
        defer_http_auth_cancel(wc, request_id);
        return;
    };
    let url = unsafe { std::ffi::CStr::from_ptr(url) }
        .to_string_lossy()
        .into_owned();
    let auth_scheme = unsafe { std::ffi::CStr::from_ptr(auth_scheme) }
        .to_string_lossy()
        .into_owned();
    let challenger = unsafe { std::ffi::CStr::from_ptr(challenger) }
        .to_string_lossy()
        .into_owned();
    let realm = unsafe { std::ffi::CStr::from_ptr(realm) }
        .to_string_lossy()
        .into_owned();
    eprintln!(
        "[termsurf-http-auth] request tab_id={} request_id={} scheme={} challenger={} realm={} proxy={} first_attempt={}",
        t.tab_id, request_id, auth_scheme, challenger, realm, is_proxy, first_auth_attempt
    );
    trace_pdf_input(format!(
        "http-auth-request tab={} pane={} request_id={} url={} scheme={} challenger={} realm={} proxy={} first_attempt={}",
        t.tab_id,
        t.pane_id,
        request_id,
        url,
        auth_scheme,
        challenger,
        realm,
        is_proxy,
        first_auth_attempt
    ));
    let msg = TermSurfMessage {
        msg: Some(Msg::HttpAuthRequest(proto::termsurf::HttpAuthRequest {
            tab_id: t.tab_id,
            request_id,
            url,
            auth_scheme,
            challenger,
            realm,
            is_proxy,
            first_auth_attempt,
            is_primary_main_frame_navigation,
            is_navigation,
        })),
    };
    if crate::ipc::send(&msg) == 0 {
        defer_http_auth_cancel(wc, request_id);
        eprintln!(
            "[termsurf-http-auth] request-no-client-cancel request_id={}",
            request_id
        );
    }
}
