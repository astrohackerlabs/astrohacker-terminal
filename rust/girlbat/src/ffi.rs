use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::{c_char, c_double, c_int, c_uint, c_ulonglong, c_void};
use std::ptr;
use std::time::{Duration, Instant};

type Runtime = c_void;
type View = c_void;

extern "C" {
    fn ts_ladybird_runtime_name() -> *const c_char;
    fn ts_ladybird_runtime_version() -> *const c_char;
    fn ts_ladybird_runtime_resource_root() -> *const c_char;
    fn ts_ladybird_runtime_create() -> *mut Runtime;
    fn ts_ladybird_runtime_destroy(runtime: *mut Runtime);
    fn ts_ladybird_runtime_pump(runtime: *mut Runtime) -> bool;
    fn ts_ladybird_runtime_last_error(runtime: *const Runtime) -> *const c_char;
    fn ts_ladybird_view_create(runtime: *mut Runtime, width: c_int, height: c_int) -> *mut View;
    fn ts_ladybird_view_destroy(view: *mut View);
    fn ts_ladybird_view_load_url(view: *mut View, url: *const c_char) -> bool;
    fn ts_ladybird_view_resize(view: *mut View, width: c_int, height: c_int) -> bool;
    fn ts_ladybird_view_set_color_scheme(view: *mut View, dark: bool) -> bool;
    fn ts_ladybird_view_set_gui_active(view: *mut View, active: bool) -> bool;
    fn ts_ladybird_view_mouse_event(
        view: *mut View,
        event_type: *const c_char,
        button: *const c_char,
        x: c_double,
        y: c_double,
        click_count: c_int,
        modifiers: c_ulonglong,
    ) -> bool;
    fn ts_ladybird_view_mouse_move(
        view: *mut View,
        x: c_double,
        y: c_double,
        modifiers: c_ulonglong,
    ) -> bool;
    fn ts_ladybird_view_scroll_event(
        view: *mut View,
        x: c_double,
        y: c_double,
        delta_x: c_double,
        delta_y: c_double,
        phase: c_ulonglong,
        momentum_phase: c_ulonglong,
        precise: bool,
        modifiers: c_ulonglong,
    ) -> bool;
    fn ts_ladybird_view_key_event(
        view: *mut View,
        event_type: *const c_char,
        windows_key_code: c_int,
        utf8: *const c_char,
        modifiers: c_ulonglong,
    ) -> bool;
    fn ts_ladybird_view_take_title_changed(
        view: *mut View,
        out_title: *mut c_char,
        out_title_len: usize,
    ) -> bool;
    fn ts_ladybird_view_take_console_message(
        view: *mut View,
        out_message: *mut ConsoleMessageRecord,
    ) -> bool;
    fn ts_ladybird_view_take_cursor_changed(view: *mut View, out_cursor_type: *mut c_int) -> bool;
    fn ts_ladybird_view_take_target_url_changed(
        view: *mut View,
        out_url: *mut c_char,
        out_url_len: usize,
    ) -> bool;
    fn ts_ladybird_view_take_javascript_dialog_request(
        view: *mut View,
        out_request: *mut JavaScriptDialogRequestRecord,
    ) -> bool;
    fn ts_ladybird_view_reply_javascript_dialog(
        view: *mut View,
        request_id: c_ulonglong,
        accepted: bool,
        prompt_text: *const c_char,
    ) -> bool;
    fn ts_ladybird_view_take_renderer_crashed(
        view: *mut View,
        out_crash: *mut RendererCrashRecord,
    ) -> bool;
    fn ts_ladybird_view_crash_current_page_for_testing(view: *mut View) -> bool;
    fn ts_ladybird_view_last_url(view: *const View) -> *const c_char;
    fn ts_ladybird_view_did_finish_load(view: *const View) -> bool;
    fn ts_ladybird_view_did_crash(view: *const View) -> bool;
    fn ts_ladybird_view_render_surface_probe(
        view: *mut View,
        out_probe: *mut RenderSurfaceProbe,
    ) -> bool;
    fn ts_ladybird_view_export_render_surface(
        view: *mut View,
        out_export: *mut RenderSurfaceExport,
    ) -> bool;
}

const DATA_URL: &str = "data:text/html,%3Ctitle%3EGirlbat%20ABI%3C/title%3E%3Cp%3Eok%3C/p%3E";

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RenderSurfaceProbe {
    pub has_surface: bool,
    pub can_export_shared_image: bool,
    pub pixel_width: c_int,
    pub pixel_height: c_int,
    pub generation: u64,
    pub ready_to_paint_seen: bool,
    pub has_usable_bitmap: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RenderSurfaceExport {
    pub has_surface: bool,
    pub surface_port: c_uint,
    pub pixel_width: c_int,
    pub pixel_height: c_int,
    pub bytes_per_row: c_ulonglong,
    pub pixel_format: c_uint,
    pub generation: c_ulonglong,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ConsoleMessageRecord {
    pub level: [c_char; 32],
    pub message: [c_char; 1024],
    pub line_no: c_int,
    pub source_id: [c_char; 512],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct JavaScriptDialogRequestRecord {
    pub request_id: c_ulonglong,
    pub dialog_type: [c_char; 32],
    pub origin_url: [c_char; 1024],
    pub message: [c_char; 1024],
    pub default_prompt_text: [c_char; 1024],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RendererCrashRecord {
    pub termination_status: [c_char; 64],
    pub termination_status_code: c_int,
    pub url: [c_char; 1024],
    pub can_reload: bool,
}

impl Default for RendererCrashRecord {
    fn default() -> Self {
        Self {
            termination_status: [0; 64],
            termination_status_code: 0,
            url: [0; 1024],
            can_reload: false,
        }
    }
}

impl Default for JavaScriptDialogRequestRecord {
    fn default() -> Self {
        Self {
            request_id: 0,
            dialog_type: [0; 32],
            origin_url: [0; 1024],
            message: [0; 1024],
            default_prompt_text: [0; 1024],
        }
    }
}

impl Default for ConsoleMessageRecord {
    fn default() -> Self {
        Self {
            level: [0; 32],
            message: [0; 1024],
            line_no: 0,
            source_id: [0; 512],
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsoleMessage {
    pub level: String,
    pub message: String,
    pub line_no: i32,
    pub source_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JavaScriptDialogRequest {
    pub request_id: u64,
    pub dialog_type: String,
    pub origin_url: String,
    pub message: String,
    pub default_prompt_text: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RendererCrash {
    pub termination_status: String,
    pub termination_status_code: i32,
    pub url: String,
    pub can_reload: bool,
}

pub struct AbiRuntime {
    raw: *mut Runtime,
}

pub struct AbiView {
    raw: *mut View,
}

impl AbiRuntime {
    pub fn create() -> Result<Self, String> {
        let raw = unsafe { ts_ladybird_runtime_create() };
        if raw.is_null() {
            return Err(last_error(ptr::null()));
        }
        Ok(Self { raw })
    }

    pub fn create_view(&self, width: i32, height: i32) -> Result<AbiView, String> {
        let raw = unsafe { ts_ladybird_view_create(self.raw, width, height) };
        if raw.is_null() {
            return Err(last_error(self.raw));
        }
        Ok(AbiView { raw })
    }

    pub fn pump(&self) -> Result<(), String> {
        if unsafe { ts_ladybird_runtime_pump(self.raw) } {
            Ok(())
        } else {
            Err(last_error(self.raw))
        }
    }
}

impl Drop for AbiRuntime {
    fn drop(&mut self) {
        unsafe { ts_ladybird_runtime_destroy(self.raw) };
    }
}

impl AbiView {
    pub fn load_url(&self, url: &str) -> Result<(), String> {
        let url = CString::new(url).map_err(|_| "url contains nul byte".to_string())?;
        if unsafe { ts_ladybird_view_load_url(self.raw, url.as_ptr()) } {
            Ok(())
        } else {
            Err(last_error(ptr::null()))
        }
    }

    pub fn resize(&self, width: i32, height: i32) -> Result<(), String> {
        if width <= 0 || height <= 0 {
            return Err("view size must be positive".to_string());
        }
        if unsafe { ts_ladybird_view_resize(self.raw, width, height) } {
            Ok(())
        } else {
            Err(last_error(ptr::null()))
        }
    }

    pub fn set_color_scheme(&self, dark: bool) -> Result<(), String> {
        if unsafe { ts_ladybird_view_set_color_scheme(self.raw, dark) } {
            Ok(())
        } else {
            Err(last_error(ptr::null()))
        }
    }

    pub fn set_gui_active(&self, active: bool) -> Result<(), String> {
        if unsafe { ts_ladybird_view_set_gui_active(self.raw, active) } {
            Ok(())
        } else {
            Err(last_error(ptr::null()))
        }
    }

    pub fn mouse_event(
        &self,
        event_type: &str,
        button: &str,
        x: f64,
        y: f64,
        click_count: i32,
        modifiers: u64,
    ) -> Result<(), String> {
        let event_type = CString::new(event_type)
            .map_err(|_| "mouse event type contains nul byte".to_string())?;
        let button =
            CString::new(button).map_err(|_| "mouse button contains nul byte".to_string())?;
        if unsafe {
            ts_ladybird_view_mouse_event(
                self.raw,
                event_type.as_ptr(),
                button.as_ptr(),
                x,
                y,
                click_count,
                modifiers,
            )
        } {
            Ok(())
        } else {
            Err(last_error(ptr::null()))
        }
    }

    pub fn mouse_move(&self, x: f64, y: f64, modifiers: u64) -> Result<(), String> {
        if unsafe { ts_ladybird_view_mouse_move(self.raw, x, y, modifiers) } {
            Ok(())
        } else {
            Err(last_error(ptr::null()))
        }
    }

    pub fn scroll_event(
        &self,
        x: f64,
        y: f64,
        delta_x: f64,
        delta_y: f64,
        phase: u64,
        momentum_phase: u64,
        precise: bool,
        modifiers: u64,
    ) -> Result<(), String> {
        if unsafe {
            ts_ladybird_view_scroll_event(
                self.raw,
                x,
                y,
                delta_x,
                delta_y,
                phase,
                momentum_phase,
                precise,
                modifiers,
            )
        } {
            Ok(())
        } else {
            Err(last_error(ptr::null()))
        }
    }

    pub fn key_event(
        &self,
        event_type: &str,
        windows_key_code: i32,
        utf8: &str,
        modifiers: u64,
    ) -> Result<(), String> {
        let event_type =
            CString::new(event_type).map_err(|_| "key event type contains nul byte".to_string())?;
        let utf8 = CString::new(utf8).map_err(|_| "key utf8 contains nul byte".to_string())?;
        if unsafe {
            ts_ladybird_view_key_event(
                self.raw,
                event_type.as_ptr(),
                windows_key_code,
                utf8.as_ptr(),
                modifiers,
            )
        } {
            Ok(())
        } else {
            Err(last_error(ptr::null()))
        }
    }

    pub fn take_title_changed(&self) -> Result<Option<String>, String> {
        let mut title = [0_i8; 1024];
        let changed = unsafe {
            ts_ladybird_view_take_title_changed(self.raw, title.as_mut_ptr(), title.len())
        };
        if !changed {
            return Ok(None);
        }
        let title = unsafe { CStr::from_ptr(title.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        Ok(Some(title))
    }

    pub fn take_console_message(&self) -> Result<Option<ConsoleMessage>, String> {
        let mut record = ConsoleMessageRecord::default();
        let changed =
            unsafe { ts_ladybird_view_take_console_message(self.raw, &mut record as *mut _) };
        if !changed {
            return Ok(None);
        }
        Ok(Some(ConsoleMessage {
            level: c_array_string(&record.level),
            message: c_array_string(&record.message),
            line_no: record.line_no,
            source_id: c_array_string(&record.source_id),
        }))
    }

    pub fn take_cursor_changed(&self) -> Result<Option<i32>, String> {
        let mut cursor_type = 0_i32;
        let changed =
            unsafe { ts_ladybird_view_take_cursor_changed(self.raw, &mut cursor_type as *mut _) };
        if !changed {
            return Ok(None);
        }
        Ok(Some(cursor_type))
    }

    pub fn take_target_url_changed(&self) -> Result<Option<String>, String> {
        let mut target_url = [0_i8; 1024];
        let changed = unsafe {
            ts_ladybird_view_take_target_url_changed(
                self.raw,
                target_url.as_mut_ptr(),
                target_url.len(),
            )
        };
        if !changed {
            return Ok(None);
        }
        Ok(Some(c_array_string(&target_url)))
    }

    pub fn take_javascript_dialog_request(
        &self,
    ) -> Result<Option<JavaScriptDialogRequest>, String> {
        let mut record = JavaScriptDialogRequestRecord::default();
        let changed = unsafe {
            ts_ladybird_view_take_javascript_dialog_request(self.raw, &mut record as *mut _)
        };
        if !changed {
            return Ok(None);
        }
        Ok(Some(JavaScriptDialogRequest {
            request_id: record.request_id as u64,
            dialog_type: c_array_string(&record.dialog_type),
            origin_url: c_array_string(&record.origin_url),
            message: c_array_string(&record.message),
            default_prompt_text: c_array_string(&record.default_prompt_text),
        }))
    }

    pub fn reply_javascript_dialog(
        &self,
        request_id: u64,
        accepted: bool,
        prompt_text: &str,
    ) -> Result<(), String> {
        let prompt_text =
            CString::new(prompt_text).map_err(|_| "dialog prompt text contains NUL".to_string())?;
        if unsafe {
            ts_ladybird_view_reply_javascript_dialog(
                self.raw,
                request_id as c_ulonglong,
                accepted,
                prompt_text.as_ptr(),
            )
        } {
            Ok(())
        } else {
            Err(last_error(ptr::null()))
        }
    }

    pub fn take_renderer_crashed(&self) -> Result<Option<RendererCrash>, String> {
        let mut record = RendererCrashRecord::default();
        let changed =
            unsafe { ts_ladybird_view_take_renderer_crashed(self.raw, &mut record as *mut _) };
        if !changed {
            return Ok(None);
        }
        Ok(Some(RendererCrash {
            termination_status: c_array_string(&record.termination_status),
            termination_status_code: record.termination_status_code,
            url: c_array_string(&record.url),
            can_reload: record.can_reload,
        }))
    }

    pub fn crash_current_page_for_testing(&self) -> Result<(), String> {
        if unsafe { ts_ladybird_view_crash_current_page_for_testing(self.raw) } {
            Ok(())
        } else {
            Err(last_error(ptr::null()))
        }
    }

    pub fn last_url(&self) -> String {
        unsafe_string(unsafe { ts_ladybird_view_last_url(self.raw) })
    }

    pub fn did_finish_load(&self) -> bool {
        unsafe { ts_ladybird_view_did_finish_load(self.raw) }
    }

    pub fn did_crash(&self) -> bool {
        unsafe { ts_ladybird_view_did_crash(self.raw) }
    }

    pub fn render_surface_probe(&self) -> Result<RenderSurfaceProbe, String> {
        let mut probe = RenderSurfaceProbe::default();
        if unsafe { ts_ladybird_view_render_surface_probe(self.raw, &mut probe) } {
            Ok(probe)
        } else {
            Err(last_error(ptr::null()))
        }
    }

    pub fn export_render_surface(&self) -> Result<RenderSurfaceExport, String> {
        let mut exported = RenderSurfaceExport::default();
        if unsafe { ts_ladybird_view_export_render_surface(self.raw, &mut exported) } {
            Ok(exported)
        } else {
            Err(last_error(ptr::null()))
        }
    }
}

impl Drop for AbiView {
    fn drop(&mut self) {
        unsafe { ts_ladybird_view_destroy(self.raw) };
    }
}

pub fn runtime_name() -> String {
    unsafe_string(unsafe { ts_ladybird_runtime_name() })
}

pub fn runtime_version() -> String {
    unsafe_string(unsafe { ts_ladybird_runtime_version() })
}

pub fn runtime_resource_root() -> String {
    unsafe_string(unsafe { ts_ladybird_runtime_resource_root() })
}

pub fn warmup() -> bool {
    handle_smoke()
}

pub fn negative_smoke() -> bool {
    let runtime = unsafe { ts_ladybird_runtime_create() };
    if runtime.is_null() {
        eprintln!(
            "[Girlbat] abi-negative-smoke failed to create first runtime: {}",
            last_error(ptr::null())
        );
        return false;
    }

    let duplicate = unsafe { ts_ladybird_runtime_create() };
    let duplicate_failed = duplicate.is_null();
    let error = last_error(ptr::null());
    unsafe { ts_ladybird_runtime_destroy(runtime) };

    if !duplicate_failed {
        unsafe { ts_ladybird_runtime_destroy(duplicate) };
        eprintln!("[Girlbat] abi-negative-smoke duplicate runtime unexpectedly succeeded");
        return false;
    }
    if error.is_empty() {
        eprintln!("[Girlbat] abi-negative-smoke duplicate runtime error was empty");
        return false;
    }

    eprintln!("[Girlbat] abi-negative-smoke duplicate runtime failed as expected: {error}");
    true
}

pub fn render_surface_smoke() -> bool {
    let runtime_name = runtime_name();
    let is_stub = runtime_name.contains("stub");
    let runtime = match AbiRuntime::create() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("[Girlbat] render-surface-smoke failed to create runtime: {error}");
            return false;
        }
    };

    let view = match runtime.create_view(800, 600) {
        Ok(view) => view,
        Err(error) => {
            eprintln!("[Girlbat] render-surface-smoke failed to create view: {error}");
            return false;
        }
    };

    if let Err(error) = view.load_url(DATA_URL) {
        eprintln!("[Girlbat] render-surface-smoke failed to load URL: {error}");
        return false;
    }

    let load_deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < load_deadline {
        if view.did_finish_load() || view.did_crash() {
            break;
        }
        if let Err(error) = runtime.pump() {
            eprintln!("[Girlbat] render-surface-smoke pump failed: {error}");
            return false;
        }
        std::thread::sleep(Duration::from_millis(5));
    }

    if view.did_crash() {
        eprintln!("[Girlbat] render-surface-smoke failed: view crashed");
        return false;
    }
    if !view.did_finish_load() {
        eprintln!("[Girlbat] render-surface-smoke failed: load timed out");
        return false;
    }

    let mut last_probe = match view.render_surface_probe() {
        Ok(probe) => probe,
        Err(error) => {
            eprintln!("[Girlbat] render-surface-smoke probe failed: {error}");
            return false;
        }
    };

    if is_stub {
        let ok = !last_probe.has_surface
            && !last_probe.can_export_shared_image
            && last_probe.pixel_width == 0
            && last_probe.pixel_height == 0
            && last_probe.generation == 0;
        eprintln!("[Girlbat] render-surface-smoke stub probe={last_probe:?}");
        if ok {
            eprintln!("[Girlbat] render-surface-smoke PASS-stub-unsupported");
        } else {
            eprintln!(
                "[Girlbat] render-surface-smoke failed: stub reported impossible surface state"
            );
        }
        return ok;
    }

    let probe_deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < probe_deadline {
        if let Err(error) = runtime.pump() {
            eprintln!("[Girlbat] render-surface-smoke post-load pump failed: {error}");
            return false;
        }
        last_probe = match view.render_surface_probe() {
            Ok(probe) => probe,
            Err(error) => {
                eprintln!("[Girlbat] render-surface-smoke probe failed: {error}");
                return false;
            }
        };
        if positive_surface(&last_probe) {
            eprintln!("[Girlbat] render-surface-smoke real probe={last_probe:?}");
            eprintln!("[Girlbat] render-surface-smoke PASS-surface");
            return true;
        }
        if impossible_surface_state(&last_probe) {
            eprintln!("[Girlbat] render-surface-smoke real probe={last_probe:?}");
            eprintln!("[Girlbat] render-surface-smoke failed: impossible surface state");
            return false;
        }
        std::thread::sleep(Duration::from_millis(5));
    }

    eprintln!("[Girlbat] render-surface-smoke real probe={last_probe:?}");
    if clean_negative_surface(&last_probe) {
        eprintln!(
            "[Girlbat] render-surface-smoke PASS-negative-finding ready_to_paint_seen={} has_usable_bitmap={}",
            last_probe.ready_to_paint_seen, last_probe.has_usable_bitmap
        );
        return true;
    }

    eprintln!("[Girlbat] render-surface-smoke failed: indeterminate surface state");
    false
}

pub fn real_frame_attachment_smoke() -> bool {
    let runtime_name = runtime_name();
    if runtime_name.contains("stub") {
        eprintln!(
            "[Girlbat] real-frame-attachment-smoke failed: stub backend cannot export a real frame"
        );
        return false;
    }

    let runtime = match AbiRuntime::create() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("[Girlbat] real-frame-attachment-smoke failed to create runtime: {error}");
            return false;
        }
    };

    let view = match runtime.create_view(800, 600) {
        Ok(view) => view,
        Err(error) => {
            eprintln!("[Girlbat] real-frame-attachment-smoke failed to create view: {error}");
            return false;
        }
    };

    if let Err(error) = view.load_url(DATA_URL) {
        eprintln!("[Girlbat] real-frame-attachment-smoke failed to load URL: {error}");
        return false;
    }

    let load_deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < load_deadline {
        if view.did_finish_load() || view.did_crash() {
            break;
        }
        if let Err(error) = runtime.pump() {
            eprintln!("[Girlbat] real-frame-attachment-smoke pump failed: {error}");
            return false;
        }
        std::thread::sleep(Duration::from_millis(5));
    }

    if view.did_crash() {
        eprintln!("[Girlbat] real-frame-attachment-smoke failed: view crashed");
        return false;
    }
    if !view.did_finish_load() {
        eprintln!("[Girlbat] real-frame-attachment-smoke failed: load timed out");
        return false;
    }

    let mut last_export = RenderSurfaceExport::default();
    let export_deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < export_deadline {
        if let Err(error) = runtime.pump() {
            eprintln!("[Girlbat] real-frame-attachment-smoke post-load pump failed: {error}");
            return false;
        }
        last_export = match view.export_render_surface() {
            Ok(exported) => exported,
            Err(error) => {
                eprintln!("[Girlbat] real-frame-attachment-smoke export failed: {error}");
                return false;
            }
        };
        if usable_export(&last_export) {
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }

    if !usable_export(&last_export) {
        eprintln!("[Girlbat] real-frame-attachment-smoke failed: no transferable surface export={last_export:?}");
        return false;
    }

    eprintln!(
        "[Girlbat] real-frame-attachment-smoke exported width={} height={} bytes_per_row={} pixel_format=0x{:x} generation={} surface_port={}",
        last_export.pixel_width,
        last_export.pixel_height,
        last_export.bytes_per_row,
        last_export.pixel_format,
        last_export.generation,
        last_export.surface_port
    );

    crate::render_channel::real_frame_attachment_smoke(last_export)
}

pub fn renderer_crash_smoke() -> bool {
    let runtime_name = runtime_name();
    if runtime_name.contains("stub") {
        eprintln!(
            "[Girlbat] renderer-crash-smoke failed: stub backend cannot prove a real Ladybird crash callback"
        );
        return false;
    }

    let runtime = match AbiRuntime::create() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("[Girlbat] renderer-crash-smoke failed to create runtime: {error}");
            return false;
        }
    };

    let view = match runtime.create_view(800, 600) {
        Ok(view) => view,
        Err(error) => {
            eprintln!("[Girlbat] renderer-crash-smoke failed to create view: {error}");
            return false;
        }
    };

    if let Err(error) = view.load_url(DATA_URL) {
        eprintln!("[Girlbat] renderer-crash-smoke failed to load URL: {error}");
        return false;
    }

    let load_deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < load_deadline {
        if view.did_finish_load() || view.did_crash() {
            break;
        }
        if let Err(error) = runtime.pump() {
            eprintln!("[Girlbat] renderer-crash-smoke pump failed: {error}");
            return false;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    if view.did_crash() {
        eprintln!("[Girlbat] renderer-crash-smoke failed: view crashed before trigger");
        return false;
    }
    if !view.did_finish_load() {
        eprintln!("[Girlbat] renderer-crash-smoke failed: load timed out");
        return false;
    }

    if let Err(error) = view.crash_current_page_for_testing() {
        eprintln!("[Girlbat] renderer-crash-smoke failed to trigger crash: {error}");
        return false;
    }

    let crash_deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < crash_deadline {
        if let Err(error) = runtime.pump() {
            eprintln!("[Girlbat] renderer-crash-smoke post-trigger pump failed: {error}");
            return false;
        }
        match view.take_renderer_crashed() {
            Ok(Some(crash)) => {
                let ok = crash.termination_status == "crashed"
                    && crash.termination_status_code == 0
                    && crash.url.starts_with("data:")
                    && crash.can_reload;
                eprintln!("[Girlbat] renderer-crash-smoke crash={crash:?} ok={ok}");
                return ok;
            }
            Ok(None) => {}
            Err(error) => {
                eprintln!("[Girlbat] renderer-crash-smoke crash poll failed: {error}");
                return false;
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }

    eprintln!("[Girlbat] renderer-crash-smoke failed: crash callback timed out");
    false
}

fn handle_smoke() -> bool {
    let runtime = unsafe { ts_ladybird_runtime_create() };
    if runtime.is_null() {
        eprintln!(
            "[Girlbat] abi handle smoke failed to create runtime: {}",
            last_error(ptr::null())
        );
        return false;
    }
    eprintln!("[Girlbat] abi handle smoke runtime created");

    let view = unsafe { ts_ladybird_view_create(runtime, 800, 600) };
    if view.is_null() {
        eprintln!(
            "[Girlbat] abi handle smoke failed to create view: {}",
            last_error(runtime)
        );
        unsafe { ts_ladybird_runtime_destroy(runtime) };
        return false;
    }
    eprintln!("[Girlbat] abi handle smoke view created");

    let url = CString::new(DATA_URL).expect("static data URL contains no nul");
    if !unsafe { ts_ladybird_view_load_url(view, url.as_ptr()) } {
        eprintln!(
            "[Girlbat] abi handle smoke failed to load URL: {}",
            last_error(runtime)
        );
        unsafe {
            ts_ladybird_view_destroy(view);
            ts_ladybird_runtime_destroy(runtime);
        }
        return false;
    }
    eprintln!("[Girlbat] abi handle smoke navigation requested url={DATA_URL}");

    let deadline = Instant::now() + Duration::from_secs(30);
    let mut completed = false;
    while Instant::now() < deadline {
        if unsafe { ts_ladybird_view_did_finish_load(view) || ts_ladybird_view_did_crash(view) } {
            completed = true;
            break;
        }
        if !unsafe { ts_ladybird_runtime_pump(runtime) } {
            eprintln!(
                "[Girlbat] abi handle smoke pump failed: {}",
                last_error(runtime)
            );
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }

    let final_url = unsafe_string(unsafe { ts_ladybird_view_last_url(view) });
    let ok = completed
        && unsafe { ts_ladybird_view_did_finish_load(view) }
        && !unsafe { ts_ladybird_view_did_crash(view) }
        && final_url.starts_with("data:");

    if ok {
        eprintln!("[Girlbat] abi handle smoke bounded pump completed");
        eprintln!("[Girlbat] abi handle smoke final url={final_url}");
    } else {
        eprintln!(
            "[Girlbat] abi handle smoke did not finish successfully final_url={final_url} error={}",
            last_error(runtime)
        );
    }

    unsafe {
        ts_ladybird_view_destroy(view);
        eprintln!("[Girlbat] abi handle smoke view destroyed");
        ts_ladybird_runtime_destroy(runtime);
        eprintln!("[Girlbat] abi handle smoke runtime destroyed");
    }

    ok
}

fn last_error(runtime: *const Runtime) -> String {
    unsafe_string(unsafe { ts_ladybird_runtime_last_error(runtime) })
}

fn positive_surface(probe: &RenderSurfaceProbe) -> bool {
    probe.has_surface
        && probe.can_export_shared_image
        && probe.pixel_width > 0
        && probe.pixel_height > 0
        && probe.generation > 0
}

fn usable_export(exported: &RenderSurfaceExport) -> bool {
    exported.has_surface
        && exported.surface_port != 0
        && exported.pixel_width > 0
        && exported.pixel_height > 0
        && exported.bytes_per_row > 0
        && exported.pixel_format != 0
        && exported.generation > 0
}

fn clean_negative_surface(probe: &RenderSurfaceProbe) -> bool {
    !probe.has_surface
        && !probe.can_export_shared_image
        && probe.pixel_width == 0
        && probe.pixel_height == 0
}

fn impossible_surface_state(probe: &RenderSurfaceProbe) -> bool {
    if probe.has_surface {
        !probe.can_export_shared_image || probe.pixel_width <= 0 || probe.pixel_height <= 0
    } else {
        probe.can_export_shared_image || probe.pixel_width != 0 || probe.pixel_height != 0
    }
}

fn unsafe_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned()
}

fn c_array_string(value: &[c_char]) -> String {
    let ptr = value.as_ptr();
    if ptr.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned()
}
