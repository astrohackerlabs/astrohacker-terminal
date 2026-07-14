#import <AppKit/AppKit.h>

#include "libtermsurf_webkit.h"
#include "test_support.h"

#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

struct State {
    ts_browser_context_t context;
    ts_web_contents_t view_a;
    ts_web_contents_t view_b;
    const char *url_a;
    const char *url_b;
    const char *url_prevent;
    int loaded_a;
    int loaded_b;
    int started;
    int peer;
    int editor;
    int phase;
    int post_phase;
    int survivor_phase;
    int loading_prevent;
    int prevention_phase;
    int prevention_execution_count;
};

static struct State *global_state;
static NSArray<NSDictionary<NSString *, NSData *> *> *saved_pasteboard;

static void snapshot_pasteboard(void)
{
    NSMutableArray *snapshot = [NSMutableArray array];
    for (NSPasteboardItem *item in NSPasteboard.generalPasteboard.pasteboardItems ?: @[]) {
        NSMutableDictionary *values = [NSMutableDictionary dictionary];
        for (NSPasteboardType type in item.types) {
            NSData *data = [item dataForType:type];
            if (data)
                values[type] = data;
        }
        [snapshot addObject:values];
    }
    saved_pasteboard = [snapshot copy];
}

static void restore_pasteboard(void)
{
    if (!saved_pasteboard)
        return;
    NSPasteboard *pasteboard = NSPasteboard.generalPasteboard;
    [pasteboard clearContents];
    NSMutableArray *items = [NSMutableArray array];
    for (NSDictionary<NSString *, NSData *> *values in saved_pasteboard) {
        NSPasteboardItem *item = [[NSPasteboardItem alloc] init];
        for (NSPasteboardType type in values)
            [item setData:values[type] forType:type];
        [items addObject:item];
    }
    if (items.count)
        [pasteboard writeObjects:items];
}

static void cleanup(void)
{
    restore_pasteboard();
    if (!global_state)
        return;
    if (global_state->view_a)
        ts_destroy_web_contents(global_state->view_a);
    if (global_state->view_b)
        ts_destroy_web_contents(global_state->view_b);
    if (global_state->context)
        ts_destroy_browser_context(global_state->context);
}

static void fail(const char *reason)
{
    fprintf(stderr, "EDITING_SMOKE_FAIL engine=webkit reason=%s\n", reason);
    fflush(stderr);
    exit(1);
}

static void require_contains(const char *value, const char *needle)
{
    if (!value || !strstr(value, needle)) {
        fprintf(stderr, "title_missing needle=%s title=%s\n", needle, value ?: "");
        fail("page_oracle");
    }
}

static void require_pair(const char *value, const char *one, const char *two)
{
    if (!value || !strstr(value, one) || !strstr(value, two)) {
        fprintf(stderr, "title_pair_missing one=%s two=%s title=%s\n",
            one, two, value ?: "");
        fail("page_oracle_pair");
    }
}

static ts_web_contents_t active_view(struct State *state)
{
    return state->peer ? state->view_b : state->view_a;
}

static ts_web_contents_t peer_view(struct State *state)
{
    return state->peer ? state->view_a : state->view_b;
}

static const char *editor_name(struct State *state)
{
    return state->editor ? "rich" : "form";
}

static const char *editor_seed(struct State *state)
{
    return state->editor ? "EDIT-BETA" : "EDIT-ALPHA";
}

static double editor_y(struct State *state)
{
    return state->editor ? 165 : 65;
}

static void seed_pasteboard(NSString *value)
{
    NSPasteboard *pasteboard = NSPasteboard.generalPasteboard;
    [pasteboard clearContents];
    if (![pasteboard setString:value forType:NSPasteboardTypeString])
        fail("pasteboard_seed");
}

static void require_pasteboard(NSString *expected)
{
    NSString *actual = [NSPasteboard.generalPasteboard stringForType:NSPasteboardTypeString] ?: @"";
    if (![actual isEqualToString:expected]) {
        fprintf(stderr, "pasteboard_mismatch expected=%s actual=%s\n",
            expected.UTF8String, actual.UTF8String);
        fail("pasteboard_oracle");
    }
}

static void move(ts_web_contents_t view, double x, double y, int modifiers)
{
    if (!ts_forward_mouse_move(view, x, y, modifiers))
        fail("native_move_rejected");
}

static void mouse(ts_web_contents_t view, int type, double x, double y, int modifiers)
{
    if (!ts_forward_mouse_event(view, type, 0, x, y, 1, modifiers))
        fail("native_mouse_rejected");
}

static void drag_all(ts_web_contents_t view, double y)
{
    move(view, 350, y, 0);
    mouse(view, 0, 350, y, 64);
    move(view, 220, y, 64);
    move(view, 100, y, 64);
    move(view, 30, y, 64);
    mouse(view, 1, 30, y, 0);
}

static void click_editor(ts_web_contents_t view, double y)
{
    move(view, 350, y, 0);
    mouse(view, 0, 350, y, 64);
    mouse(view, 1, 350, y, 0);
}

static void key(ts_web_contents_t view, int keycode, int modifiers)
{
    if (!ts_forward_key_event(view, 0, keycode, "", modifiers)
        || !ts_forward_key_event(view, 1, keycode, "", modifiers))
        fail("native_key_rejected");
}

static void commit(ts_web_contents_t view, const char *text)
{
    if (!ts_forward_text_input(view, "commit", text, -1, -1, -1, -1))
        fail("native_text_rejected");
}

static void dispatch_matrix(void *user_data);
static void begin_post_matrix(struct State *state);
static void start_post(void *user_data);
static void query_post(void *user_data);
static void run_survivor(void *user_data);
static void start_prevention(void *user_data);

static int expected_editing_action(int type, int keycode, int modifiers)
{
    if (type != 0 && type != 2)
        return 0;
    if (modifiers == 9 && keycode == 0x5A)
        return 6;
    if (modifiers != 8)
        return 0;
    switch (keycode) {
    case 0x41: return 1;
    case 0x43: return 2;
    case 0x58: return 3;
    case 0x56: return 4;
    case 0x5A: return 5;
    default: return 0;
    }
}

static void verify_editing_mapping_contract(void)
{
    for (int type = -1; type <= 3; type++) {
        for (int keycode = 0; keycode <= 0xFF; keycode++) {
            for (int modifiers = 0; modifiers <= 31; modifiers++) {
                int expected = expected_editing_action(type, keycode, modifiers);
                int actual = ts_webkit_test_editing_action_for_key(
                    type, keycode, "", modifiers, 0);
                if (actual != expected) {
                    fprintf(stderr,
                        "editing_mapping_mismatch type=%d keycode=%d modifiers=%d expected=%d actual=%d\n",
                        type, keycode, modifiers, expected, actual);
                    fail("editing_mapping_contract");
                }
                if (ts_webkit_test_editing_action_for_key(
                        type, keycode, "x", modifiers, 0) != 0
                    || ts_webkit_test_editing_action_for_key(
                        type, keycode, "", modifiers, 1) != 0) {
                    fail("editing_mapping_sideband");
                }
            }
        }
    }
    if (ts_webkit_test_editing_action_for_key(0, 0x43, NULL, 8, 0) != 0)
        fail("editing_mapping_null_text");
}

static void verify_matrix(const char *title, void *user_data)
{
    struct State *state = user_data;
    const char *editor = editor_name(state);
    const char *seed = editor_seed(state);
    char needle[512];

    switch (state->phase) {
    case 0:
        if (state->editor)
            require_contains(title, "\"selectedText\":\"EDIT-BETA\"");
        else
            require_contains(title, "\"formStart\":0,\"formEnd\":10");
        break;
    case 1:
        snprintf(needle, sizeof(needle), "[\"copy\",\"%s\",\"\",\"\",\"%s\"]", editor, seed);
        require_contains(title, needle);
        require_pasteboard([NSString stringWithUTF8String:seed]);
        snprintf(needle, sizeof(needle), state->editor ? "\"richText\":\"%s\"" : "\"formValue\":\"%s\"", seed);
        require_contains(title, needle);
        break;
    case 2:
        snprintf(needle, sizeof(needle), "[\"cut\",\"%s\",\"\",\"\",\"%s\"]", editor, seed);
        require_contains(title, needle);
        require_pair(title, "deleteByCut", state->editor ? "\"richText\":\"\"" : "\"formValue\":\"\"");
        require_pasteboard([NSString stringWithUTF8String:seed]);
        break;
    case 3:
        require_pair(title, "[\"paste\"", "insertFromPaste");
        require_contains(title, state->editor ? "\"richText\":\"PASTE-日本語\"" : "\"formValue\":\"PASTE-日本語\"");
        break;
    case 4:
        require_pair(title, "historyUndo", state->editor ? "\"richText\":\"\"" : "\"formValue\":\"\"");
        break;
    case 5:
        require_pair(title, "historyRedo", state->editor ? "\"richText\":\"PASTE-日本語\"" : "\"formValue\":\"PASTE-日本語\"");
        break;
    case 6:
        if (state->editor)
            require_contains(title, "\"selectedText\":\"PASTE-日本語\"");
        else
            require_contains(title, "\"formStart\":0,\"formEnd\":9");
        break;
    case 7:
        require_pair(title, "insertText", state->editor ? "\"richText\":\"é😀\"" : "\"formValue\":\"é😀\"");
        break;
    default:
        fail("unexpected_matrix_phase");
    }

    state->phase++;
    if (state->phase <= 7)
        ts_webkit_test_post_delayed_task(0.08, dispatch_matrix, state);
    else
        begin_post_matrix(state);
}

static void query_matrix(void *user_data)
{
    struct State *state = user_data;
    ts_webkit_test_evaluate_javascript(active_view(state), "document.title", verify_matrix, state);
}

static void dispatch_matrix(void *user_data)
{
    struct State *state = user_data;
    ts_web_contents_t view = active_view(state);
    switch (state->phase) {
    case 0:
        drag_all(view, editor_y(state));
        break;
    case 1:
        seed_pasteboard(@"COPY-SENTINEL");
        key(view, 0x43, 8);
        break;
    case 2:
        key(view, 0x58, 8);
        break;
    case 3:
        seed_pasteboard(@"PASTE-日本語");
        key(view, 0x56, 8);
        break;
    case 4:
        key(view, 0x5A, 8);
        break;
    case 5:
        key(view, 0x5A, 9);
        break;
    case 6:
        key(view, 0x41, 8);
        break;
    case 7:
        commit(view, "é😀");
        break;
    default:
        fail("unexpected_matrix_dispatch");
    }
    ts_webkit_test_post_delayed_task(0.2, query_matrix, state);
}

static void verify_isolation(const char *title, void *user_data)
{
    struct State *state = user_data;
    if (state->peer)
        require_pair(title, "\"formValue\":\"é😀\"", "\"richText\":\"é😀\"");
    else
        require_pair(title, "\"formValue\":\"EDIT-ALPHA\"", "\"richText\":\"EDIT-BETA\"");

    if (!state->editor) {
        state->editor = 1;
    } else if (!state->peer) {
        ts_set_focus(state->view_a, false);
        ts_set_focus(state->view_b, true);
        state->peer = 1;
        state->editor = 0;
    } else {
        ts_webkit_test_post_delayed_task(0.1, start_post, state);
        return;
    }
    state->phase = 0;
    ts_webkit_test_post_delayed_task(0.1, dispatch_matrix, state);
}

static void begin_post_matrix(struct State *state)
{
    ts_webkit_test_evaluate_javascript(peer_view(state), "document.title", verify_isolation, state);
}

static void finish_success(struct State *state)
{
    ts_destroy_web_contents(state->view_b);
    state->view_b = NULL;
    ts_destroy_browser_context(state->context);
    state->context = NULL;
    restore_pasteboard();
    printf("EDITING_SMOKE_PASS engine=webkit tabs=2 surfaces=2 selection=1 copy=1 cut=1 paste=1 undo=1 redo=1 replace=1 system_pasteboard=1 isolation=1 survivor=1 prevention=1 identity=1 supersession=1 pending_cleanup=1 disabled_action=1 cleanup=1\n");
    fflush(stdout);
    ts_quit();
}

static void verify_survivor(const char *title, void *user_data)
{
    struct State *state = user_data;
    switch (state->survivor_phase) {
    case 1:
        require_pair(title, "\"active\":\"form\"", "\"selectedText\":\"é😀\"");
        break;
    case 2:
        require_contains(title, "\"formStart\":0,\"formEnd\":3");
        break;
    case 3:
        require_contains(title, "\"formValue\":\"SURVIVOR\"");
        finish_success(state);
        return;
    default:
        fail("unexpected_survivor_phase");
    }
    ts_webkit_test_post_delayed_task(0.08, run_survivor, state);
}

static void query_survivor(void *user_data)
{
    struct State *state = user_data;
    ts_webkit_test_evaluate_javascript(state->view_b, "document.title", verify_survivor, state);
}

static void run_survivor(void *user_data)
{
    struct State *state = user_data;
    switch (state->survivor_phase) {
    case 0:
        ts_set_focus(state->view_b, true);
        drag_all(state->view_b, 65);
        break;
    case 1:
        key(state->view_b, 0x41, 8);
        break;
    case 2:
        commit(state->view_b, "SURVIVOR");
        break;
    default:
        fail("unexpected_survivor_dispatch");
    }
    state->survivor_phase++;
    ts_webkit_test_post_delayed_task(0.2, query_survivor, state);
}

static void verify_peer_after_history(const char *title, void *user_data)
{
    struct State *state = user_data;
    require_contains(title, "\"richText\":\"é😀\"");
    key(state->view_a, 0x5A, 9);
    state->post_phase = 3;
    ts_webkit_test_post_delayed_task(0.2, query_post, state);
}

static void verify_post(const char *title, void *user_data)
{
    struct State *state = user_data;
    switch (state->post_phase) {
    case 0:
        require_contains(title, "\"active\":\"rich\"");
        key(state->view_a, 0x5A, 8);
        state->post_phase = 1;
        break;
    case 1:
        require_pair(title, "historyUndo", "\"richText\":\"PASTE-日本語\"");
        ts_webkit_test_evaluate_javascript(state->view_b, "document.title", verify_peer_after_history, state);
        return;
    case 3:
        require_pair(title, "historyRedo", "\"richText\":\"é😀\"");
        if (ts_forward_key_event(NULL, 0, 0x43, "", 8)
            || ts_forward_key_event(state->view_a, 3, 0x43, "", 8)
            || ts_forward_key_event(state->view_a, 0, 0x43, "", 16)
            || ts_forward_key_event(state->view_b, 0, 0x43, "", 8))
            fail("malformed_key_accepted");
        state->loading_prevent = 1;
        ts_load_url(state->view_a, state->url_prevent);
        return;
    default:
        fail("unexpected_post_phase");
    }
    ts_webkit_test_post_delayed_task(0.2, query_post, state);
}

static void query_post(void *user_data)
{
    struct State *state = user_data;
    ts_webkit_test_evaluate_javascript(state->view_a, "document.title", verify_post, state);
}

static void start_post(void *user_data)
{
    struct State *state = user_data;
    ts_set_focus(state->view_b, false);
    ts_set_focus(state->view_a, true);
    click_editor(state->view_a, 165);
    state->post_phase = 0;
    ts_webkit_test_post_delayed_task(0.2, query_post, state);
}

static void finish_prevention(void *user_data)
{
    struct State *state = user_data;
    if (ts_webkit_test_editing_pending_count() != 1)
        fail("close_pending_missing");
    ts_destroy_web_contents(state->view_a);
    state->view_a = NULL;
    if (ts_webkit_test_editing_pending_count() != 0)
        fail("close_pending_cleanup");
    ts_webkit_test_post_delayed_task(0.2, run_survivor, state);
}

static void verify_prevention(const char *title, void *user_data);

static void query_prevention(void *user_data)
{
    struct State *state = user_data;
    ts_webkit_test_evaluate_javascript(
        state->view_a, "document.title", verify_prevention, state);
}

static void verify_prevention_expired(void *user_data)
{
    struct State *state = user_data;
    if (ts_webkit_test_editing_pending_count() != 0)
        fail("prevent_pending_expiry");
    if (ts_webkit_test_editing_responder_execution_count()
        != state->prevention_execution_count)
        fail("prevent_late_execution");

    key(state->view_a, 0x43, 8);
    if (ts_webkit_test_editing_pending_count() != 1)
        fail("focus_cleanup_pending_missing");
    ts_set_focus(state->view_a, false);
    if (ts_webkit_test_editing_pending_count() != 0)
        fail("focus_cleanup_pending_retained");
    ts_set_focus(state->view_a, true);

    key(state->view_a, 0x5A, 8);
    state->prevention_phase = 3;
    ts_webkit_test_post_delayed_task(0.25, query_prevention, state);
}

static void verify_prevention(const char *title, void *user_data)
{
    struct State *state = user_data;
    switch (state->prevention_phase) {
    case 0:
        require_pair(title, "\"preventCopy\":true", "\"selectedText\":\"EDIT-ALPHA\"");
        seed_pasteboard(@"PREVENT-SENTINEL");
        state->prevention_execution_count =
            ts_webkit_test_editing_responder_execution_count();
        key(state->view_a, 0x43, 8);
        if (ts_webkit_test_editing_pending_count() != 1)
            fail("prevent_pending_missing");
        if (ts_webkit_test_unrelated_editing_event_consumed() != 0)
            fail("event_identity_soft_match");
        if (ts_webkit_test_editing_pending_count() != 1)
            fail("identity_probe_consumed_pending");
        state->prevention_phase = 1;
        ts_webkit_test_post_delayed_task(0.25, query_prevention, state);
        return;
    case 1:
        require_pair(title, "\"preventedCopyCount\":1", "[\"keydown\",\"form\",\"\",\"c\",\"EDIT-ALPHA\"]");
        if (strstr(title, "[\"copy\"") != NULL)
            fail("prevent_copy_event");
        require_pasteboard(@"PREVENT-SENTINEL");
        if (ts_webkit_test_editing_responder_execution_count()
            != state->prevention_execution_count)
            fail("prevent_native_execution");
        key(state->view_a, 0x43, 8);
        if (ts_webkit_test_editing_pending_count() != 1)
            fail("supersession_pending_count");
        state->prevention_phase = 2;
        ts_webkit_test_post_delayed_task(0.25, query_prevention, state);
        return;
    case 2:
        require_contains(title, "\"preventedCopyCount\":2");
        if (strstr(title, "[\"copy\"") != NULL)
            fail("superseded_copy_event");
        require_pasteboard(@"PREVENT-SENTINEL");
        if (ts_webkit_test_editing_responder_execution_count()
            != state->prevention_execution_count)
            fail("superseded_native_execution");
        ts_webkit_test_post_delayed_task(2.2, verify_prevention_expired, state);
        return;
    case 3:
        require_contains(title, "[\"keydown\",\"form\",\"\",\"z\"");
        if (strstr(title, "historyUndo") != NULL)
            fail("disabled_undo_page_action");
        if (ts_webkit_test_editing_responder_execution_count()
            != state->prevention_execution_count)
            fail("disabled_undo_execution");
        if (ts_webkit_test_editing_pending_count() != 0)
            fail("disabled_undo_pending");
        key(state->view_a, 0x43, 8);
        ts_webkit_test_post_delayed_task(0.05, finish_prevention, state);
        return;
    default:
        fail("unexpected_prevention_phase");
    }
}

static void start_prevention(void *user_data)
{
    struct State *state = user_data;
    ts_set_focus(state->view_b, false);
    ts_set_focus(state->view_a, true);
    drag_all(state->view_a, 65);
    state->prevention_phase = 0;
    ts_webkit_test_post_delayed_task(0.25, query_prevention, state);
}

static void verify_initial_b(const char *title, void *user_data)
{
    struct State *state = user_data;
    require_pair(title, "\"peer\":\"B\"", "\"formValue\":\"EDIT-ALPHA\"");
    ts_set_focus(state->view_a, true);
    ts_set_focus(state->view_b, false);
    state->peer = 0;
    state->editor = 0;
    state->phase = 0;
    ts_webkit_test_post_delayed_task(0.1, dispatch_matrix, state);
}

static void verify_initial_a(const char *title, void *user_data)
{
    struct State *state = user_data;
    require_pair(title, "\"peer\":\"A\"", "\"richText\":\"EDIT-BETA\"");
    ts_webkit_test_evaluate_javascript(state->view_b, "document.title", verify_initial_b, state);
}

static void start_sequence(void *user_data)
{
    struct State *state = user_data;
    ts_set_view_size(state->view_a, 800, 600, 0, 0, 400, 300, 2);
    ts_set_view_size(state->view_b, 800, 600, 400, 0, 400, 300, 2);
    ts_webkit_test_evaluate_javascript(state->view_a, "document.title", verify_initial_a, state);
}

static void on_loading_state(ts_web_contents_t view, const char *url, int loading, void *user_data)
{
    (void)url;
    struct State *state = user_data;
    if (state->loading_prevent && view == state->view_a) {
        if (!loading) {
            state->loading_prevent = 0;
            ts_webkit_test_post_delayed_task(0.5, start_prevention, state);
        }
        return;
    }
    if (loading)
        return;
    if (view == state->view_a)
        state->loaded_a = 1;
    if (view == state->view_b)
        state->loaded_b = 1;
    if (state->loaded_a && state->loaded_b && !state->started) {
        state->started = 1;
        ts_webkit_test_post_delayed_task(0.5, start_sequence, state);
    }
}

static void on_initialized(void *user_data)
{
    struct State *state = user_data;
    state->context = ts_create_incognito_browser_context();
    state->view_a = ts_create_web_contents(state->context, state->url_a, 800, 600, false);
    state->view_b = ts_create_web_contents(state->context, state->url_b, 800, 600, false);
    if (!state->context || !state->view_a || !state->view_b || state->view_a == state->view_b)
        fail("distinct_views");
}

int main(int argc, const char **argv)
{
    @autoreleasepool {
        if (argc != 4) {
            fprintf(stderr, "usage: %s <peer-a-url> <peer-b-url> <prevent-copy-url>\n", argv[0]);
            return 2;
        }
        verify_editing_mapping_contract();
        snapshot_pasteboard();
        atexit(cleanup);
        struct State *state = calloc(1, sizeof(*state));
        if (!state)
            fail("state_allocation");
        state->url_a = argv[1];
        state->url_b = argv[2];
        state->url_prevent = argv[3];
        global_state = state;
        ts_set_on_initialized(on_initialized, state);
        ts_set_on_loading_state(on_loading_state, state);
        return ts_content_main(argc, argv);
    }
}
