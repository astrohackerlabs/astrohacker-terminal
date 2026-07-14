use std::ffi::c_void;
use std::os::raw::{c_char, c_double, c_int, c_ulonglong};
use std::path::PathBuf;
use std::sync::OnceLock;

use libloading::{Library, Symbol};

pub type TsBrowserContext = *mut c_void;
pub type TsWebContents = *mut c_void;

fn library() -> &'static Library {
    static LIBRARY: OnceLock<Library> = OnceLock::new();
    LIBRARY.get_or_init(|| {
        let mut candidates = Vec::new();
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                candidates.push(parent.join("libtermsurf_webkit.dylib"));
            }
        }
        candidates.push(PathBuf::from(env!("ASTROHACKER_WEBKIT_ABI_DYLIB")));

        let mut errors = Vec::new();
        for candidate in candidates {
            match unsafe { Library::new(&candidate) } {
                Ok(library) => return library,
                Err(error) => errors.push(format!("{}: {error}", candidate.display())),
            }
        }
        panic!(
            "failed to load libtermsurf_webkit.dylib; tried {}",
            errors.join("; ")
        );
    })
}

pub fn abi_load_smoke() -> Result<(), String> {
    unsafe {
        library()
            .get::<unsafe extern "C" fn(c_int, *const *const c_char) -> c_int>(b"ts_content_main\0")
            .map(|_| ())
            .map_err(|error| error.to_string())
    }
}

unsafe fn symbol<T>(name: &[u8]) -> Symbol<'static, T> {
    library()
        .get(name)
        .unwrap_or_else(|error| panic!("failed to resolve WebKit ABI symbol {name:?}: {error}"))
}

pub unsafe fn ts_content_main(argc: c_int, argv: *const *const c_char) -> c_int {
    type Fn = unsafe extern "C" fn(c_int, *const *const c_char) -> c_int;
    symbol::<Fn>(b"ts_content_main\0")(argc, argv)
}

pub unsafe fn ts_set_on_initialized(
    callback: Option<unsafe extern "C" fn(*mut c_void)>,
    user_data: *mut c_void,
) {
    type Fn = unsafe extern "C" fn(Option<unsafe extern "C" fn(*mut c_void)>, *mut c_void);
    symbol::<Fn>(b"ts_set_on_initialized\0")(callback, user_data)
}

pub unsafe fn ts_post_task(
    task: Option<unsafe extern "C" fn(*mut c_void)>,
    user_data: *mut c_void,
) {
    type Fn = unsafe extern "C" fn(Option<unsafe extern "C" fn(*mut c_void)>, *mut c_void);
    symbol::<Fn>(b"ts_post_task\0")(task, user_data)
}

pub unsafe fn ts_quit() {
    type Fn = unsafe extern "C" fn();
    symbol::<Fn>(b"ts_quit\0")()
}

pub unsafe fn ts_create_browser_context(path: *const c_char) -> TsBrowserContext {
    type Fn = unsafe extern "C" fn(*const c_char) -> TsBrowserContext;
    symbol::<Fn>(b"ts_create_browser_context\0")(path)
}

pub unsafe fn ts_create_incognito_browser_context() -> TsBrowserContext {
    type Fn = unsafe extern "C" fn() -> TsBrowserContext;
    symbol::<Fn>(b"ts_create_incognito_browser_context\0")()
}

pub unsafe fn ts_destroy_browser_context(ctx: TsBrowserContext) {
    type Fn = unsafe extern "C" fn(TsBrowserContext);
    symbol::<Fn>(b"ts_destroy_browser_context\0")(ctx)
}

pub unsafe fn ts_create_web_contents(
    ctx: TsBrowserContext,
    url: *const c_char,
    width: c_int,
    height: c_int,
    dark: bool,
) -> TsWebContents {
    type Fn =
        unsafe extern "C" fn(TsBrowserContext, *const c_char, c_int, c_int, bool) -> TsWebContents;
    symbol::<Fn>(b"ts_create_web_contents\0")(ctx, url, width, height, dark)
}

pub unsafe fn ts_create_devtools_web_contents(
    ctx: TsBrowserContext,
    inspected_tab_id: c_int,
    width: c_int,
    height: c_int,
    dark: bool,
) -> TsWebContents {
    type Fn = unsafe extern "C" fn(TsBrowserContext, c_int, c_int, c_int, bool) -> TsWebContents;
    symbol::<Fn>(b"ts_create_devtools_web_contents\0")(ctx, inspected_tab_id, width, height, dark)
}

pub unsafe fn ts_destroy_web_contents(wc: TsWebContents) {
    type Fn = unsafe extern "C" fn(TsWebContents);
    symbol::<Fn>(b"ts_destroy_web_contents\0")(wc)
}

pub unsafe fn ts_load_url(wc: TsWebContents, url: *const c_char) {
    type Fn = unsafe extern "C" fn(TsWebContents, *const c_char);
    symbol::<Fn>(b"ts_load_url\0")(wc, url)
}

pub unsafe fn ts_navigation_action(wc: TsWebContents, action: *const c_char) -> bool {
    type Fn = unsafe extern "C" fn(TsWebContents, *const c_char) -> bool;
    symbol::<Fn>(b"ts_navigation_action\0")(wc, action)
}

pub unsafe fn ts_webkit_test_kill_web_content_process(wc: TsWebContents) {
    type Fn = unsafe extern "C" fn(TsWebContents);
    symbol::<Fn>(b"ts_webkit_test_kill_web_content_process\0")(wc)
}

pub unsafe fn ts_forward_mouse_event(
    wc: TsWebContents,
    r#type: c_int,
    button: c_int,
    x: c_double,
    y: c_double,
    click_count: c_int,
    modifiers: c_int,
) -> bool {
    type Fn =
        unsafe extern "C" fn(TsWebContents, c_int, c_int, c_double, c_double, c_int, c_int) -> bool;
    symbol::<Fn>(b"ts_forward_mouse_event\0")(wc, r#type, button, x, y, click_count, modifiers)
}

pub unsafe fn ts_forward_mouse_move(
    wc: TsWebContents,
    x: c_double,
    y: c_double,
    modifiers: c_int,
) -> bool {
    type Fn = unsafe extern "C" fn(TsWebContents, c_double, c_double, c_int) -> bool;
    symbol::<Fn>(b"ts_forward_mouse_move\0")(wc, x, y, modifiers)
}

pub unsafe fn ts_forward_scroll_event(
    wc: TsWebContents,
    x: c_double,
    y: c_double,
    delta_x: c_double,
    delta_y: c_double,
    phase: c_int,
    momentum_phase: c_int,
    precise: bool,
    modifiers: c_int,
) -> bool {
    type Fn = unsafe extern "C" fn(
        TsWebContents,
        c_double,
        c_double,
        c_double,
        c_double,
        c_int,
        c_int,
        bool,
        c_int,
    ) -> bool;
    symbol::<Fn>(b"ts_forward_scroll_event\0")(
        wc,
        x,
        y,
        delta_x,
        delta_y,
        phase,
        momentum_phase,
        precise,
        modifiers,
    )
}

pub unsafe fn ts_forward_key_event(
    wc: TsWebContents,
    r#type: c_int,
    keycode: c_int,
    utf8: *const c_char,
    modifiers: c_int,
) -> bool {
    type Fn = unsafe extern "C" fn(TsWebContents, c_int, c_int, *const c_char, c_int) -> bool;
    symbol::<Fn>(b"ts_forward_key_event\0")(wc, r#type, keycode, utf8, modifiers)
}

pub unsafe fn ts_forward_text_input(
    wc: TsWebContents,
    type_: *const c_char,
    text: *const c_char,
    selected_start: i64,
    selected_length: i64,
    replacement_start: i64,
    replacement_length: i64,
) -> bool {
    type Fn = unsafe extern "C" fn(
        TsWebContents,
        *const c_char,
        *const c_char,
        i64,
        i64,
        i64,
        i64,
    ) -> bool;
    symbol::<Fn>(b"ts_forward_text_input\0")(
        wc,
        type_,
        text,
        selected_start,
        selected_length,
        replacement_start,
        replacement_length,
    )
}

pub unsafe fn ts_set_focus(wc: TsWebContents, focused: bool) {
    type Fn = unsafe extern "C" fn(TsWebContents, bool);
    symbol::<Fn>(b"ts_set_focus\0")(wc, focused)
}

pub unsafe fn ts_set_gui_active(wc: TsWebContents, active: bool, reason: *const c_char) {
    type Fn = unsafe extern "C" fn(TsWebContents, bool, *const c_char);
    symbol::<Fn>(b"ts_set_gui_active\0")(wc, active, reason)
}

pub unsafe fn ts_set_color_scheme(wc: TsWebContents, dark: bool) {
    type Fn = unsafe extern "C" fn(TsWebContents, bool);
    symbol::<Fn>(b"ts_set_color_scheme\0")(wc, dark)
}

pub unsafe fn ts_set_view_size(
    wc: TsWebContents,
    width: c_int,
    height: c_int,
    screen_x: c_double,
    screen_y: c_double,
    screen_width: c_double,
    screen_height: c_double,
    screen_scale: c_double,
) {
    type Fn = unsafe extern "C" fn(
        TsWebContents,
        c_int,
        c_int,
        c_double,
        c_double,
        c_double,
        c_double,
        c_double,
    );
    symbol::<Fn>(b"ts_set_view_size\0")(
        wc,
        width,
        height,
        screen_x,
        screen_y,
        screen_width,
        screen_height,
        screen_scale,
    )
}

pub unsafe fn ts_reply_javascript_dialog(
    wc: TsWebContents,
    request_id: c_ulonglong,
    accepted: bool,
    prompt_text: *const c_char,
) -> bool {
    type Fn = unsafe extern "C" fn(TsWebContents, c_ulonglong, bool, *const c_char) -> bool;
    symbol::<Fn>(b"ts_reply_javascript_dialog\0")(wc, request_id, accepted, prompt_text)
}

pub unsafe fn ts_reply_http_auth(
    wc: TsWebContents,
    request_id: c_ulonglong,
    accepted: bool,
    username: *const c_char,
    password: *const c_char,
) -> bool {
    type Fn = unsafe extern "C" fn(
        TsWebContents,
        c_ulonglong,
        bool,
        *const c_char,
        *const c_char,
    ) -> bool;
    symbol::<Fn>(b"ts_reply_http_auth\0")(wc, request_id, accepted, username, password)
}

pub unsafe fn ts_set_on_tab_ready(
    cb: Option<unsafe extern "C" fn(TsWebContents, c_int, *mut c_void)>,
    user_data: *mut c_void,
) {
    type Fn = unsafe extern "C" fn(
        Option<unsafe extern "C" fn(TsWebContents, c_int, *mut c_void)>,
        *mut c_void,
    );
    symbol::<Fn>(b"ts_set_on_tab_ready\0")(cb, user_data)
}

pub unsafe fn ts_set_on_ca_context_id(
    cb: Option<unsafe extern "C" fn(TsWebContents, u32, c_int, c_int, *mut c_void)>,
    user_data: *mut c_void,
) {
    type Fn = unsafe extern "C" fn(
        Option<unsafe extern "C" fn(TsWebContents, u32, c_int, c_int, *mut c_void)>,
        *mut c_void,
    );
    symbol::<Fn>(b"ts_set_on_ca_context_id\0")(cb, user_data)
}

pub unsafe fn ts_set_on_url_changed(
    cb: Option<unsafe extern "C" fn(TsWebContents, *const c_char, *mut c_void)>,
    user_data: *mut c_void,
) {
    type Fn = unsafe extern "C" fn(
        Option<unsafe extern "C" fn(TsWebContents, *const c_char, *mut c_void)>,
        *mut c_void,
    );
    symbol::<Fn>(b"ts_set_on_url_changed\0")(cb, user_data)
}

pub unsafe fn ts_set_on_loading_state(
    cb: Option<unsafe extern "C" fn(TsWebContents, *const c_char, c_int, *mut c_void)>,
    user_data: *mut c_void,
) {
    type Fn = unsafe extern "C" fn(
        Option<unsafe extern "C" fn(TsWebContents, *const c_char, c_int, *mut c_void)>,
        *mut c_void,
    );
    symbol::<Fn>(b"ts_set_on_loading_state\0")(cb, user_data)
}

pub unsafe fn ts_set_on_navigation_state(
    cb: Option<unsafe extern "C" fn(TsWebContents, bool, bool, bool, *mut c_void)>,
    user_data: *mut c_void,
) {
    type Fn = unsafe extern "C" fn(
        Option<unsafe extern "C" fn(TsWebContents, bool, bool, bool, *mut c_void)>,
        *mut c_void,
    );
    symbol::<Fn>(b"ts_set_on_navigation_state\0")(cb, user_data)
}

pub unsafe fn ts_set_on_title_changed(
    cb: Option<unsafe extern "C" fn(TsWebContents, *const c_char, *mut c_void)>,
    user_data: *mut c_void,
) {
    type Fn = unsafe extern "C" fn(
        Option<unsafe extern "C" fn(TsWebContents, *const c_char, *mut c_void)>,
        *mut c_void,
    );
    symbol::<Fn>(b"ts_set_on_title_changed\0")(cb, user_data)
}

pub unsafe fn ts_set_on_cursor_changed(
    cb: Option<unsafe extern "C" fn(TsWebContents, c_int, *mut c_void)>,
    user_data: *mut c_void,
) {
    type Fn = unsafe extern "C" fn(
        Option<unsafe extern "C" fn(TsWebContents, c_int, *mut c_void)>,
        *mut c_void,
    );
    symbol::<Fn>(b"ts_set_on_cursor_changed\0")(cb, user_data)
}

pub unsafe fn ts_set_on_target_url_changed(
    cb: Option<unsafe extern "C" fn(TsWebContents, *const c_char, *mut c_void)>,
    user_data: *mut c_void,
) {
    type Fn = unsafe extern "C" fn(
        Option<unsafe extern "C" fn(TsWebContents, *const c_char, *mut c_void)>,
        *mut c_void,
    );
    symbol::<Fn>(b"ts_set_on_target_url_changed\0")(cb, user_data)
}

pub unsafe fn ts_set_on_javascript_dialog_request(
    cb: Option<
        unsafe extern "C" fn(
            TsWebContents,
            c_ulonglong,
            *const c_char,
            *const c_char,
            *const c_char,
            *const c_char,
            *mut c_void,
        ),
    >,
    user_data: *mut c_void,
) {
    type Callback = unsafe extern "C" fn(
        TsWebContents,
        c_ulonglong,
        *const c_char,
        *const c_char,
        *const c_char,
        *const c_char,
        *mut c_void,
    );
    type Fn = unsafe extern "C" fn(Option<Callback>, *mut c_void);
    symbol::<Fn>(b"ts_set_on_javascript_dialog_request\0")(cb, user_data)
}

pub unsafe fn ts_set_on_console_message(
    cb: Option<
        unsafe extern "C" fn(
            TsWebContents,
            *const c_char,
            *const c_char,
            c_int,
            *const c_char,
            *mut c_void,
        ),
    >,
    user_data: *mut c_void,
) {
    type Callback = unsafe extern "C" fn(
        TsWebContents,
        *const c_char,
        *const c_char,
        c_int,
        *const c_char,
        *mut c_void,
    );
    type Fn = unsafe extern "C" fn(Option<Callback>, *mut c_void);
    symbol::<Fn>(b"ts_set_on_console_message\0")(cb, user_data)
}

pub unsafe fn ts_set_on_http_auth_request(
    cb: Option<
        unsafe extern "C" fn(
            TsWebContents,
            c_ulonglong,
            *const c_char,
            *const c_char,
            *const c_char,
            *const c_char,
            bool,
            bool,
            bool,
            bool,
            *mut c_void,
        ),
    >,
    user_data: *mut c_void,
) {
    type Callback = unsafe extern "C" fn(
        TsWebContents,
        c_ulonglong,
        *const c_char,
        *const c_char,
        *const c_char,
        *const c_char,
        bool,
        bool,
        bool,
        bool,
        *mut c_void,
    );
    type Fn = unsafe extern "C" fn(Option<Callback>, *mut c_void);
    symbol::<Fn>(b"ts_set_on_http_auth_request\0")(cb, user_data)
}

pub unsafe fn ts_set_on_renderer_crashed(
    cb: Option<
        unsafe extern "C" fn(TsWebContents, *const c_char, c_int, *const c_char, bool, *mut c_void),
    >,
    user_data: *mut c_void,
) {
    type Callback =
        unsafe extern "C" fn(TsWebContents, *const c_char, c_int, *const c_char, bool, *mut c_void);
    type Fn = unsafe extern "C" fn(Option<Callback>, *mut c_void);
    symbol::<Fn>(b"ts_set_on_renderer_crashed\0")(cb, user_data)
}

pub unsafe fn ts_set_on_render_probe(
    cb: Option<
        unsafe extern "C" fn(
            TsWebContents,
            *const c_char,
            *const c_char,
            c_int,
            c_int,
            c_int,
            c_int,
            c_int,
            c_int,
            *const c_char,
            *mut c_void,
        ),
    >,
    user_data: *mut c_void,
) {
    type Callback = unsafe extern "C" fn(
        TsWebContents,
        *const c_char,
        *const c_char,
        c_int,
        c_int,
        c_int,
        c_int,
        c_int,
        c_int,
        *const c_char,
        *mut c_void,
    );
    type Fn = unsafe extern "C" fn(Option<Callback>, *mut c_void);
    symbol::<Fn>(b"ts_set_on_render_probe\0")(cb, user_data)
}
