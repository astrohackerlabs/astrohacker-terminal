#import <AppKit/AppKit.h>

#include "libtermsurf_webkit.h"
#include "test_support.h"

#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

struct NavState {
    bool can_go_back;
    bool can_go_forward;
    bool can_refresh;
    int events;
};

struct State {
    ts_browser_context_t context;
    ts_web_contents_t view_a;
    ts_web_contents_t view_b;
    const char *base_url;
    struct NavState nav[2];
    int phase;
    int initial_done;
    int reload_before;
    bool awaiting_load;
    bool crashed;
    bool finished;
};

static struct State *global_state;
static void fail(const char *reason);
static void query_a(void *user_data);
static void query_b(void *user_data);
static void query_after_same_document(void *user_data);
static void finish(struct State *state);
static void pushed_state_ready(const char *result, void *user_data);

static void cleanup(void)
{
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
    fprintf(stderr, "NAVIGATION_ACTIONS_SMOKE_FAIL engine=webkit reason=%s\n", reason);
    fflush(stderr);
    exit(1);
}

static int view_index(struct State *state, ts_web_contents_t view)
{
    if (view == state->view_a)
        return 0;
    if (view == state->view_b)
        return 1;
    fail("unknown_view");
    return -1;
}

static void make_url(char *out, size_t size, struct State *state, const char *path)
{
    snprintf(out, size, "%s%s", state->base_url, path);
}

static void expect_nav(struct State *state, int index, bool back, bool forward, bool refresh)
{
    struct NavState *nav = &state->nav[index];
    if (nav->can_go_back != back || nav->can_go_forward != forward || nav->can_refresh != refresh) {
        fprintf(stderr,
            "navigation_state_mismatch index=%d expected=(%d,%d,%d) actual=(%d,%d,%d) events=%d phase=%d\n",
            index, back, forward, refresh,
            nav->can_go_back, nav->can_go_forward, nav->can_refresh, nav->events,
            state->phase);
        fail("navigation_state_mismatch");
    }
}

static int reload_count(const char *result)
{
    const char *reload = strstr(result ?: "", "reload=");
    if (!reload)
        fail("reload_counter_missing");
    return atoi(reload + strlen("reload="));
}

static void expect_page(const char *result, const char *path, const char *title)
{
    if (!result || !*result || strstr(result, "ERROR:"))
        fail("page_oracle_error");
    if (!strstr(result, path) || !strstr(result, title)) {
        fprintf(stderr, "page_oracle_mismatch expected_path=%s expected_title=%s actual=%s\n",
            path, title, result ?: "");
        fail("page_oracle_mismatch");
    }
}

static void start_phase(struct State *state, int phase)
{
    char url[1024];
    state->phase = phase;
    state->awaiting_load = false;
    switch (phase) {
    case 1:
        state->awaiting_load = true;
        make_url(url, sizeof(url), state, "/b?peer=A");
        ts_load_url(state->view_a, url);
        break;
    case 2:
        state->awaiting_load = true;
        if (!ts_navigation_action(state->view_a, "back"))
            fail("semantic_back_rejected");
        break;
    case 3:
        if (ts_navigation_action(state->view_a, "back"))
            fail("disabled_back_accepted");
        ts_webkit_test_post_delayed_task(0.2, query_a, state);
        break;
    case 4:
        state->awaiting_load = true;
        if (!ts_navigation_action(state->view_a, "forward"))
            fail("semantic_forward_rejected");
        break;
    case 5:
        ts_webkit_test_evaluate_javascript(state->view_a,
            "history.pushState({smoke:true}, '', location.pathname + location.search + '#state');"
            "document.title='NAV_B peer=A pushed';"
            "document.title + '|' + location.pathname + location.search + location.hash + '|reload=' + document.documentElement.dataset.reload",
            pushed_state_ready,
            state);
        break;
    case 6:
        if (!ts_navigation_action(state->view_a, "back"))
            fail("same_document_back_rejected");
        ts_webkit_test_post_delayed_task(0.2, query_after_same_document, state);
        break;
    case 7:
        if (!ts_navigation_action(state->view_a, "forward"))
            fail("same_document_forward_rejected");
        ts_webkit_test_post_delayed_task(0.2, query_after_same_document, state);
        break;
    case 8:
        state->awaiting_load = true;
        if (!ts_navigation_action(state->view_a, "refresh"))
            fail("semantic_refresh_rejected");
        break;
    case 9:
        ts_webkit_test_kill_web_content_process(state->view_a);
        ts_webkit_test_post_delayed_task(0.5, query_after_same_document, state);
        break;
    case 10:
        state->awaiting_load = true;
        if (!ts_navigation_action(state->view_a, "refresh"))
            fail("crash_refresh_rejected");
        break;
    default:
        finish(state);
        break;
    }
}

static void pushed_state_ready(const char *result, void *user_data)
{
    struct State *state = user_data;
    expect_page(result, "/b?peer=A#state", "NAV_B peer=A pushed");
    expect_nav(state, 0, true, false, true);
    ts_webkit_test_post_delayed_task(0.2, query_after_same_document, state);
}

static void verified_b(const char *result, void *user_data)
{
    (void)user_data;
    expect_page(result, "/a?peer=B", "NAV_A peer=B reload=1");
}

static void verified_a(const char *result, void *user_data)
{
    struct State *state = user_data;
    switch (state->phase) {
    case 0:
        expect_page(result, "/a?peer=A", "NAV_A peer=A reload=1");
        ts_webkit_test_evaluate_javascript(state->view_b,
            "document.title + '|' + location.pathname + location.search + '|reload=' + document.documentElement.dataset.reload",
            verified_b, state);
        expect_nav(state, 0, false, false, true);
        expect_nav(state, 1, false, false, true);
        start_phase(state, 1);
        break;
    case 1:
        expect_page(result, "/b?peer=A", "NAV_B peer=A reload=1");
        expect_nav(state, 0, true, false, true);
        expect_nav(state, 1, false, false, true);
        start_phase(state, 2);
        break;
    case 2:
    case 3:
        expect_page(result, "/a?peer=A", "NAV_A peer=A reload=1");
        expect_nav(state, 0, false, true, true);
        expect_nav(state, 1, false, false, true);
        start_phase(state, state->phase == 2 ? 3 : 4);
        break;
    case 4:
        expect_page(result, "/b?peer=A", "NAV_B peer=A reload=1");
        state->reload_before = reload_count(result);
        expect_nav(state, 0, true, false, true);
        expect_nav(state, 1, false, false, true);
        start_phase(state, 5);
        break;
    case 8:
        expect_page(result, "/b?peer=A", "NAV_B peer=A");
        if (reload_count(result) != state->reload_before + 1)
            fail("reload_counter_not_incremented");
        expect_nav(state, 0, true, false, true);
        expect_nav(state, 1, false, false, true);
        start_phase(state, 9);
        break;
    case 10:
        expect_page(result, "/b?peer=A", "NAV_B peer=A");
        expect_nav(state, 0, true, false, true);
        expect_nav(state, 1, false, false, true);
        finish(state);
        break;
    default:
        fail("unexpected_verified_a_phase");
    }
}

static void verified_same_document(const char *result, void *user_data)
{
    struct State *state = user_data;
    if (state->phase == 5) {
        expect_page(result, "/b?peer=A#state", "NAV_B peer=A pushed");
        start_phase(state, 6);
    } else if (state->phase == 6) {
        expect_page(result, "/b?peer=A", "NAV_B peer=A");
        expect_nav(state, 0, true, true, true);
        start_phase(state, 7);
    } else if (state->phase == 7) {
        expect_page(result, "/b?peer=A#state", "NAV_B peer=A pushed");
        expect_nav(state, 0, true, false, true);
        start_phase(state, 8);
    } else if (state->phase == 9) {
        if (!state->crashed)
            fail("crash_callback_missing");
        expect_nav(state, 0, false, false, true);
        expect_nav(state, 1, false, false, true);
        start_phase(state, 10);
    } else {
        fail("unexpected_same_document_phase");
    }
}

static void query_a(void *user_data)
{
    struct State *state = user_data;
    ts_webkit_test_evaluate_javascript(state->view_a,
        "document.title + '|' + location.pathname + location.search + location.hash + '|reload=' + document.documentElement.dataset.reload",
        verified_a, state);
}

static void query_b(void *user_data)
{
    struct State *state = user_data;
    ts_webkit_test_evaluate_javascript(state->view_b,
        "document.title + '|' + location.pathname + location.search + location.hash + '|reload=' + document.documentElement.dataset.reload",
        verified_b, state);
}

static void query_after_same_document(void *user_data)
{
    struct State *state = user_data;
    ts_webkit_test_evaluate_javascript(state->view_a,
        "document.title + '|' + location.pathname + location.search + location.hash + '|reload=' + document.documentElement.dataset.reload",
        verified_same_document, state);
}

static void on_loading_state(ts_web_contents_t view, const char *url, int loading, void *user_data)
{
    (void)url;
    struct State *state = user_data;
    if (loading)
        return;
    if (state->phase == 0) {
        state->initial_done++;
        if (state->initial_done == 2)
            ts_webkit_test_post_delayed_task(0.2, query_a, state);
        return;
    }
    if (view == state->view_a && state->awaiting_load)
        ts_webkit_test_post_delayed_task(0.2, query_a, state);
}

static void on_navigation_state(ts_web_contents_t view, bool back, bool forward, bool refresh, void *user_data)
{
    struct State *state = user_data;
    if ((view != state->view_a && view != state->view_b) || !state->view_a || !state->view_b)
        return;
    int index = view_index(state, view);
    state->nav[index].can_go_back = back;
    state->nav[index].can_go_forward = forward;
    state->nav[index].can_refresh = refresh;
    state->nav[index].events++;
}

static void on_renderer_crashed(ts_web_contents_t view, const char *reason, int exit_code, const char *url, bool can_reload, void *user_data)
{
    (void)reason;
    (void)exit_code;
    (void)url;
    struct State *state = user_data;
    if (view != state->view_a)
        fail("wrong_view_crashed");
    state->crashed = true;
    if (!can_reload)
        fail("crash_not_reloadable");
}

static void finish(struct State *state)
{
    if (state->nav[0].events < 6 || state->nav[1].events < 1)
        fail("navigation_state_events_missing");
    query_b(state);
    state->finished = true;
    puts("NAVIGATION_ACTIONS_SMOKE_PASS engine=webkit tabs=2 back=1 forward=1 refresh=1 capabilities=1 disabled=1 isolation=1 same_document=1 crash_recovery=1 cleanup=1");
    fflush(stdout);
    ts_quit();
}

static void watchdog(void *user_data)
{
    struct State *state = user_data;
    if (!state->finished)
        fail("timeout");
}

static void on_initialized(void *user_data)
{
    struct State *state = user_data;
    char url_a[1024], url_b[1024];
    make_url(url_a, sizeof(url_a), state, "/a?peer=A");
    make_url(url_b, sizeof(url_b), state, "/a?peer=B");
    state->context = ts_create_incognito_browser_context();
    state->view_a = ts_create_web_contents(state->context, url_a, 640, 480, false);
    state->view_b = ts_create_web_contents(state->context, url_b, 640, 480, false);
    if (!state->context || !state->view_a || !state->view_b || state->view_a == state->view_b)
        fail("distinct_views");
    ts_set_view_size(state->view_a, 640, 480, 0, 0, 640, 480, 1);
    ts_set_view_size(state->view_b, 640, 480, 640, 0, 640, 480, 1);
    ts_webkit_test_post_delayed_task(70, watchdog, state);
}

int main(int argc, const char **argv)
{
    @autoreleasepool {
        if (argc != 2) {
            fprintf(stderr, "usage: %s http://127.0.0.1:PORT\n", argv[0]);
            return 2;
        }
        struct State *state = calloc(1, sizeof(*state));
        if (!state)
            fail("state_allocation");
        state->base_url = argv[1];
        global_state = state;
        atexit(cleanup);
        ts_set_on_initialized(on_initialized, state);
        ts_set_on_loading_state(on_loading_state, state);
        ts_set_on_navigation_state(on_navigation_state, state);
        ts_set_on_renderer_crashed(on_renderer_crashed, state);
        return ts_content_main(argc, argv);
    }
}
