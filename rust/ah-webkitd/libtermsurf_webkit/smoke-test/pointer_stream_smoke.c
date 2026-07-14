#include "libtermsurf_webkit.h"
#include "test_support.h"

#include <math.h>
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
    int loaded_a;
    int loaded_b;
    int started;
    int step;
    int action_step;
    int cursor_a[8];
    int cursor_a_count;
    int cursor_b[8];
    int cursor_b_count;
    char target_a[8][256];
    int target_a_count;
    char target_b[8][256];
    int target_b_count;
};

static void fail(const char *reason)
{
    fprintf(stderr, "POINTER_SMOKE_FAIL %s\n", reason);
    fflush(stderr);
    exit(1);
}

static void require_contains(const char *value, const char *needle)
{
    if (!value || !strstr(value, needle)) {
        fprintf(stderr, "title_missing needle=%s title=%s\n", needle, value ? value : "");
        fail("page_oracle");
    }
}

static void forward_mouse(
    ts_web_contents_t view,
    int type,
    int button,
    double x,
    double y,
    int click_count,
    int modifiers)
{
    if (!ts_forward_mouse_event(view, type, button, x, y, click_count, modifiers))
        fail("native_mouse_rejected");
}

static void forward_move(ts_web_contents_t view, double x, double y, int modifiers)
{
    if (!ts_forward_mouse_move(view, x, y, modifiers))
        fail("native_move_rejected");
}

static void forward_scroll(
    ts_web_contents_t view,
    double x,
    double y,
    double delta_x,
    double delta_y,
    int phase,
    int momentum_phase,
    bool precise,
    int modifiers)
{
    if (!ts_forward_scroll_event(
            view, x, y, delta_x, delta_y, phase, momentum_phase, precise, modifiers))
        fail("native_scroll_rejected");
}

static void require_malformed_rejection(struct State *state)
{
    if (ts_forward_mouse_event(NULL, 0, 0, 10, 10, 1, 0) ||
        ts_forward_mouse_event(state->view_a, 2, 0, 10, 10, 1, 0) ||
        ts_forward_mouse_event(state->view_a, 0, 3, 10, 10, 1, 0) ||
        ts_forward_mouse_event(state->view_a, 0, 0, NAN, 10, 1, 0) ||
        ts_forward_mouse_event(state->view_a, 0, 0, 10, 10, 0, 0) ||
        ts_forward_mouse_event(state->view_a, 0, 0, 10, 10, 1, 512) ||
        ts_forward_mouse_move(state->view_a, INFINITY, 10, 0) ||
        ts_forward_mouse_move(state->view_a, 10, 10, 512) ||
        ts_forward_scroll_event(state->view_a, 10, 10, NAN, 1, 0, 0, true, 0) ||
        ts_forward_scroll_event(state->view_a, 10, 10, 0, 0, 64, 0, true, 0) ||
        ts_forward_scroll_event(state->view_a, 10, 10, 0, 0, 0, 64, true, 0) ||
        ts_forward_scroll_event(state->view_a, 10, 10, 1, 0, 0, 0, true, 64) ||
        ts_forward_scroll_event(state->view_a, 10, 10, 0, 0, 0, 0, true, 0))
        fail("malformed_input_accepted");
}

static void on_cursor(ts_web_contents_t view, int cursor, void *user_data)
{
    struct State *state = user_data;
    int *count = view == state->view_a ? &state->cursor_a_count : &state->cursor_b_count;
    int *values = view == state->view_a ? state->cursor_a : state->cursor_b;
    if (*count >= 8)
        fail("cursor_overflow");
    values[(*count)++] = cursor;
}

static void on_target(ts_web_contents_t view, const char *url, void *user_data)
{
    struct State *state = user_data;
    int *count = view == state->view_a ? &state->target_a_count : &state->target_b_count;
    char (*values)[256] = view == state->view_a ? state->target_a : state->target_b;
    if (*count >= 8)
        fail("target_overflow");
    snprintf(values[(*count)++], 256, "%s", url ? url : "");
}

static void query_step(void *user_data);
static void dispatch_action(void *user_data);

static void advance_hover(const char *title, void *user_data)
{
    struct State *state = user_data;
    switch (state->step++) {
    case 0:
        require_contains(title, "[\"mousemove\",50,40,0,0,0,0]");
        if (state->cursor_a_count != 1 || state->cursor_a[0] != 0 || state->target_a_count)
            fail("default_feedback");
        forward_move(state->view_a, 140, 40, 0);
        break;
    case 1:
        require_contains(title, "[\"mousemove\",140,40,0,0,0,0]");
        if (state->cursor_a_count != 2 || state->cursor_a[1] != 2 ||
            state->target_a_count != 1 ||
            strcmp(state->target_a[0], "https://example.test/pointer-target"))
            fail("link_feedback");
        forward_move(state->view_a, 240, 40, 0);
        break;
    case 2:
        require_contains(title, "[\"mousemove\",240,40,0,0,0,0]");
        if (state->cursor_a_count != 3 || state->cursor_a[2] != 3 ||
            state->target_a_count != 2 || strcmp(state->target_a[1], ""))
            fail("text_feedback");
        forward_move(state->view_b, 140, 40, 0);
        break;
    case 3:
        require_contains(title, "[\"mousemove\",140,40,0,0,0,0]");
        if (state->cursor_b_count != 1 || state->cursor_b[0] != 2 ||
            state->target_b_count != 1 ||
            strcmp(state->target_b[0], "https://example.test/pointer-target") ||
            state->cursor_a_count != 3 || state->target_a_count != 2)
            fail("peer_feedback");

        dispatch_action(state);
        return;
    default:
        fail("unexpected_hover_step");
    }
    ts_webkit_test_post_delayed_task(0.3, query_step, state);
}

static void dispatch_action(void *user_data)
{
    struct State *state = user_data;
    switch (state->action_step++) {
    case 0: forward_mouse(state->view_a, 0, 0, 100, 105, 1, 1 | 8 | 64); break;
    case 1: forward_mouse(state->view_a, 1, 0, 100, 105, 1, 1 | 8); break;
    case 2: forward_mouse(state->view_a, 0, 2, 100, 105, 1, 128); break;
    case 3: forward_mouse(state->view_a, 1, 2, 100, 105, 1, 0); break;
    case 4: forward_mouse(state->view_a, 0, 1, 100, 105, 1, 256); break;
    case 5: forward_mouse(state->view_a, 1, 1, 100, 105, 1, 0); break;
    case 6: forward_mouse(state->view_a, 0, 0, 100, 105, 2, 64); break;
    case 7: forward_mouse(state->view_a, 1, 0, 100, 105, 2, 0); break;
    case 8: forward_mouse(state->view_a, 0, 0, 50, 175, 1, 64); break;
    case 9: forward_move(state->view_a, 120, 175, 64); break;
    case 10: forward_move(state->view_a, 220, 175, 64); break;
    case 11: forward_mouse(state->view_a, 1, 0, 220, 175, 1, 0); break;
    case 12: forward_scroll(state->view_a, 300, 170, 0, 0, 1, 0, true, 1); break;
    case 13: forward_scroll(state->view_a, 300, 170, 1.5, 24.25, 4, 0, true, 1); break;
    case 14: forward_scroll(state->view_a, 300, 170, 0, 0, 8, 0, true, 1); break;
    case 15: forward_scroll(state->view_a, 300, 170, 0, 5.75, 0, 1, true, 0); break;
    case 16: forward_scroll(state->view_a, 300, 170, 0, 3, 0, 4, true, 0); break;
    case 17: forward_scroll(state->view_a, 300, 170, 0, 0, 0, 8, true, 0); break;
    default:
        ts_webkit_test_post_delayed_task(0.5, query_step, state);
        return;
    }
    ts_webkit_test_post_delayed_task(0.08, dispatch_action, state);
}

static void finish(const char *title, void *user_data)
{
    struct State *state = user_data;
    require_contains(title, "[\"mousedown\",100,105,0,1,1,73]");
    require_contains(title, "[\"mousedown\",100,105,1,4,1,128]");
    require_contains(title, "[\"mousedown\",100,105,2,2,1,256]");
    require_contains(title, "[\"mouseup\",100,105,0,0,2,0]");
    require_contains(title, "[\"mousemove\",120,175,0,1,0,64]");
    require_contains(title, "[\"mousemove\",220,175,0,1,0,64]");
    require_contains(title, "[\"mouseup\",220,175,0,0,1,0]");
    require_contains(title, "[300,170,1.5,24.25,0,1]");
    require_contains(title, "\"scroll\":[0,33]");

    ts_destroy_web_contents(state->view_a);
    ts_destroy_web_contents(state->view_b);
    ts_destroy_browser_context(state->context);
    state->view_a = NULL;
    state->view_b = NULL;
    state->context = NULL;
    printf("POINTER_SMOKE_PASS engine=webkit tabs=2 coordinates=1 buttons=3 clicks=2 modifiers=1 drag=1 precise_scroll=1 phases=1 momentum=1 cursor=1 target_url=1 isolation=1 cleanup=1\n");
    ts_quit();
}

static void query_step(void *user_data)
{
    struct State *state = user_data;
    ts_web_contents_t view = state->step == 3 ? state->view_b : state->view_a;
    if (state->step >= 4)
        ts_webkit_test_evaluate_javascript(state->view_a, "document.title", finish, state);
    else
        ts_webkit_test_evaluate_javascript(view, "document.title", advance_hover, state);
}

static void start(void *user_data)
{
    struct State *state = user_data;
    ts_set_view_size(state->view_a, 800, 600, 0, 0, 400, 300, 2);
    ts_set_view_size(state->view_b, 800, 600, 400, 0, 400, 300, 2);
    ts_set_focus(state->view_a, true);
    ts_set_focus(state->view_b, true);
    require_malformed_rejection(state);
    forward_move(state->view_a, 50, 40, 0);
    ts_webkit_test_post_delayed_task(0.3, query_step, state);
}

static void on_loading(ts_web_contents_t view, const char *url, int loading, void *user_data)
{
    (void)url;
    struct State *state = user_data;
    if (loading)
        return;
    if (view == state->view_a)
        state->loaded_a = 1;
    if (view == state->view_b)
        state->loaded_b = 1;
    if (state->loaded_a && state->loaded_b && !state->started) {
        state->started = 1;
        ts_webkit_test_post_delayed_task(0.5, start, state);
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
    if (argc != 3) {
        fprintf(stderr, "usage: %s <peer-a-url> <peer-b-url>\n", argv[0]);
        return 2;
    }
    struct State state = { .url_a = argv[1], .url_b = argv[2] };
    ts_set_on_initialized(on_initialized, &state);
    ts_set_on_loading_state(on_loading, &state);
    ts_set_on_cursor_changed(on_cursor, &state);
    ts_set_on_target_url_changed(on_target, &state);
    return ts_content_main(argc, argv);
}
