#import <AppKit/AppKit.h>

#include "libtermsurf_webkit.h"
#include "test_support.h"

#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

struct ViewLife {
    int active;
    int starts;
    int terminals;
    bool saw_url;
    bool saw_title;
    bool completed;
    bool expect_stop;
    char expected_url[256];
    char expected_title[256];
};

struct State {
    ts_browser_context_t context;
    ts_web_contents_t view_a;
    ts_web_contents_t view_b;
    const char *base_url;
    struct ViewLife life[2];
    int phase;
    int initial_query;
    int stop_attempts;
    int forward_reload;
    bool initial_started;
    bool stop_requested;
    bool finished;
};

static struct State *global_state;
static void fail(const char *reason);

static int expected_navigation_action(
    int type, int keycode, int modifiers, bool loading, bool escape_owned)
{
    if (type < 0 || type > 2 || (modifiers & ~15) != 0)
        return 0;
    if (modifiers == 8 && (keycode == 0xDB || keycode == 0xDD || keycode == 0x52)) {
        if (type != 0)
            return 1;
        return keycode == 0xDB ? 2 : keycode == 0xDD ? 3 : 4;
    }
    if (modifiers == 0 && keycode == 0x1B) {
        if (type == 0 && loading)
            return 5;
        if (escape_owned)
            return 1;
    }
    return 0;
}

static void verify_navigation_mapping_contract(void)
{
    for (int type = -1; type <= 3; type++) {
        for (int keycode = 0; keycode <= 0xFF; keycode++) {
            for (int modifiers = 0; modifiers <= 31; modifiers++) {
                for (int loading = 0; loading <= 1; loading++) {
                    for (int escape_owned = 0; escape_owned <= 1; escape_owned++) {
                        int expected = expected_navigation_action(
                            type, keycode, modifiers, loading != 0, escape_owned != 0);
                        int actual = ts_webkit_test_navigation_action_for_key(
                            type, keycode, modifiers, loading, escape_owned);
                        if (actual != expected) {
                            fprintf(stderr,
                                "navigation_mapping_mismatch type=%d keycode=%d modifiers=%d loading=%d escape_owned=%d expected=%d actual=%d\n",
                                type, keycode, modifiers, loading, escape_owned,
                                expected, actual);
                            fail("navigation_mapping_contract");
                        }
                    }
                }
            }
        }
    }
}

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
    fprintf(stderr, "NAVIGATION_LIFECYCLE_SMOKE_FAIL engine=webkit reason=%s\n", reason);
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

static void set_expectation(struct State *state, int index, const char *url,
    const char *title, bool expect_stop)
{
    struct ViewLife *life = &state->life[index];
    if (life->active)
        fail("expectation_while_active");
    life->saw_url = false;
    life->saw_title = false;
    life->completed = false;
    life->expect_stop = expect_stop;
    snprintf(life->expected_url, sizeof(life->expected_url), "%s", url);
    snprintf(life->expected_title, sizeof(life->expected_title), "%s", title);
}

static void make_url(char *out, size_t size, struct State *state, const char *path)
{
    snprintf(out, size, "%s%s", state->base_url, path);
}

static void key(ts_web_contents_t view, int keycode, int modifiers)
{
    if (!ts_forward_key_event(view, 0, keycode, "", modifiers)
        || !ts_forward_key_event(view, 1, keycode, "", modifiers))
        fail("physical_key_rejected");
}

static void query_page(void *user_data);
static void attempt_stop(void *user_data);

static void start_phase(struct State *state, int phase)
{
    char url[1024];
    state->phase = phase;
    switch (phase) {
    case 1:
        set_expectation(state, 0, "/b?peer=A", "NAV_B peer=A reload=1", false);
        make_url(url, sizeof(url), state, "/b?peer=A");
        ts_load_url(state->view_a, url);
        break;
    case 2:
        set_expectation(state, 0, "/a?peer=A", "NAV_A peer=A reload=2", false);
        make_url(url, sizeof(url), state, "/a?peer=A");
        ts_load_url(state->view_a, url);
        break;
    case 3:
        set_expectation(state, 0, "/b?peer=A", "NAV_B peer=A reload=2", false);
        ts_set_focus(state->view_b, false);
        ts_set_focus(state->view_a, true);
        if (!ts_forward_mouse_move(state->view_a, 100, 120, 0)
            || !ts_forward_mouse_event(state->view_a, 0, 0, 100, 120, 1, 64)
            || !ts_forward_mouse_event(state->view_a, 1, 0, 100, 120, 1, 0))
            fail("link_pointer");
        break;
    case 4:
        set_expectation(state, 0, "/a?peer=A", "NAV_A peer=A", false);
        key(state->view_a, 0xDB, 8);
        break;
    case 5:
        set_expectation(state, 0, "/b?peer=A", "NAV_B peer=A", false);
        key(state->view_a, 0xDD, 8);
        break;
    case 6:
        set_expectation(state, 0, "/b?peer=A", "NAV_B peer=A", false);
        key(state->view_a, 0x52, 8);
        break;
    case 7:
        set_expectation(state, 0, "/b?peer=A", "NAV_B peer=A", false);
        make_url(url, sizeof(url), state, "/redirect?peer=A");
        ts_load_url(state->view_a, url);
        break;
    case 8:
        set_expectation(state, 0, "/slow?peer=A", "NAV_SLOW_START peer=A", true);
        state->stop_requested = false;
        state->stop_attempts = 0;
        make_url(url, sizeof(url), state, "/slow?peer=A");
        ts_load_url(state->view_a, url);
        ts_webkit_test_post_delayed_task(0.05, attempt_stop, state);
        break;
    case 9:
        set_expectation(state, 1, "/b?peer=B", "NAV_B peer=B reload=1", false);
        make_url(url, sizeof(url), state, "/b?peer=B");
        ts_load_url(state->view_b, url);
        break;
    default:
        fail("unknown_phase");
    }
}

static void finish(struct State *state)
{
    if (state->life[0].active || state->life[1].active)
        fail("cleanup_active_navigation");
    if (state->life[0].starts != 9 || state->life[0].terminals != 9)
        fail("view_a_lifecycle_count");
    if (state->life[1].starts != 2 || state->life[1].terminals != 2)
        fail("view_b_lifecycle_count");
    ts_destroy_web_contents(state->view_b);
    state->view_b = NULL;
    ts_destroy_browser_context(state->context);
    state->context = NULL;
    state->finished = true;
    puts("NAVIGATION_LIFECYCLE_SMOKE_PASS engine=webkit tabs=2 navigate=1 link=1 redirect=1 back=1 forward=1 reload=1 stop=1 lifecycle=1 isolation=1 survivor=1 cleanup=1");
    fflush(stdout);
    ts_quit();
}

static void verified_page(const char *result, void *user_data)
{
    struct State *state = user_data;
    if (!result || !*result || strstr(result, "ERROR:"))
        fail("page_oracle_error");

    if (state->phase == 0) {
        const char *title = state->initial_query == 0
            ? "NAV_A peer=A reload=1|/a?peer=A|"
            : "NAV_A peer=B reload=1|/a?peer=B|";
        if (!strstr(result, title))
            fail("initial_page_oracle");
        if (state->initial_query == 0) {
            state->initial_query = 1;
            ts_webkit_test_evaluate_javascript(state->view_b,
                "document.title + '|' + location.pathname + location.search + '|' + (document.documentElement.dataset.terminal || '')",
                verified_page, state);
            return;
        }
        start_phase(state, 1);
        return;
    }

    if (state->phase == 8) {
        if (state->initial_query == 0) {
            if (!strstr(result, "NAV_SLOW_START peer=A|/slow?peer=A|")
                || strstr(result, "NAV_SLOW_TERMINAL")
                || strstr(result, "|1"))
                fail("stop_page_oracle");
            state->initial_query = 1;
            ts_webkit_test_evaluate_javascript(state->view_b,
                "document.title + '|' + location.pathname + location.search + '|' + (document.documentElement.dataset.terminal || '')",
                verified_page, state);
            return;
        }
        if (!strstr(result, "NAV_A peer=B reload=1|/a?peer=B|"))
            fail("peer_isolation");
        ts_destroy_web_contents(state->view_a);
        state->view_a = NULL;
        start_phase(state, 9);
        return;
    }

    struct ViewLife *life = &state->life[state->phase == 9 ? 1 : 0];
    if (!strstr(result, life->expected_title) || !strstr(result, life->expected_url))
        fail("page_oracle_mismatch");
    const char *reload = strstr(result, "reload=");
    if ((state->phase == 5 || state->phase == 6) && !reload)
        fail("reload_counter_missing");
    if (state->phase == 5)
        state->forward_reload = atoi(reload + strlen("reload="));
    if (state->phase == 6
        && atoi(reload + strlen("reload=")) != state->forward_reload + 1)
        fail("reload_counter_not_incremented");
    if (state->phase == 9) {
        finish(state);
        return;
    }
    start_phase(state, state->phase + 1);
}

static void query_page(void *user_data)
{
    struct State *state = user_data;
    ts_web_contents_t view = state->phase == 9 ? state->view_b : state->view_a;
    ts_webkit_test_evaluate_javascript(view,
        "document.title + '|' + location.pathname + location.search + '|' + (document.documentElement.dataset.terminal || '')",
        verified_page, state);
}

static void stop_title_checked(const char *result, void *user_data)
{
    struct State *state = user_data;
    if (state->finished || state->phase != 8 || state->stop_requested)
        return;
    if (result && strstr(result, "NAV_SLOW_START peer=A")) {
        if (!state->life[0].active)
            fail("stop_without_active_load");
        state->stop_requested = true;
        key(state->view_a, 0x1B, 0);
        return;
    }
    if (++state->stop_attempts >= 40)
        fail("slow_start_timeout");
    ts_webkit_test_post_delayed_task(0.1, attempt_stop, state);
}

static void attempt_stop(void *user_data)
{
    struct State *state = user_data;
    if (state->finished || state->phase != 8 || state->stop_requested)
        return;
    ts_webkit_test_evaluate_javascript(
        state->view_a, "document.title", stop_title_checked, state);
}

static void on_loading_state(ts_web_contents_t view, const char *url, int loading,
    void *user_data)
{
    (void)url;
    struct State *state = user_data;
    int index = view_index(state, view);
    struct ViewLife *life = &state->life[index];
    if (loading) {
        if (life->active)
            fail("duplicate_start");
        life->active = 1;
        life->starts++;
        life->saw_url = false;
        life->saw_title = false;
        return;
    }
    if (!life->active)
        fail("terminal_without_start");
    if (!life->saw_url || !life->saw_title) {
        fprintf(stderr,
            "lifecycle_observation_missing phase=%d view=%d saw_url=%d saw_title=%d expected_url=%s expected_title=%s callback_url=%s\n",
            state->phase, index, life->saw_url, life->saw_title,
            life->expected_url, life->expected_title, url ?: "");
        fail("missing_lifecycle_observation");
    }
    if (life->expect_stop && !state->stop_requested)
        fail("slow_completed_before_stop");
    life->active = 0;
    life->terminals++;
    life->completed = true;

    if (state->phase == 0) {
        if (state->life[0].completed && state->life[1].completed
            && !state->initial_started) {
            state->initial_started = true;
            state->initial_query = 0;
            ts_webkit_test_post_delayed_task(0.1, query_page, state);
        }
        return;
    }
    if ((state->phase == 9 && index != 1) || (state->phase != 9 && index != 0))
        fail("wrong_view_terminal");
    if (state->phase == 8)
        state->initial_query = 0;
    ts_webkit_test_post_delayed_task(0.1, query_page, state);
}

static void on_url_changed(ts_web_contents_t view, const char *url, void *user_data)
{
    struct State *state = user_data;
    struct ViewLife *life = &state->life[view_index(state, view)];
    if (life->active && url && strstr(url, life->expected_url))
        life->saw_url = true;
}

static void on_title_changed(ts_web_contents_t view, const char *title, void *user_data)
{
    struct State *state = user_data;
    struct ViewLife *life = &state->life[view_index(state, view)];
    if (life->active && title && strstr(title, life->expected_title))
        life->saw_title = true;
    if (title && strstr(title, "NAV_SLOW_TERMINAL"))
        fail("slow_terminal_title");
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
    set_expectation(state, 0, "/a?peer=A", "NAV_A peer=A reload=1", false);
    set_expectation(state, 1, "/a?peer=B", "NAV_A peer=B reload=1", false);
    state->context = ts_create_incognito_browser_context();
    state->view_a = ts_create_web_contents(state->context, url_a, 640, 480, false);
    state->view_b = ts_create_web_contents(state->context, url_b, 640, 480, false);
    if (!state->context || !state->view_a || !state->view_b
        || state->view_a == state->view_b)
        fail("distinct_views");
    ts_set_view_size(state->view_a, 640, 480, 0, 0, 640, 480, 1);
    ts_set_view_size(state->view_b, 640, 480, 640, 0, 640, 480, 1);
    ts_webkit_test_post_delayed_task(70, watchdog, state);
}

int main(int argc, const char **argv)
{
    @autoreleasepool {
        verify_navigation_mapping_contract();
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
        ts_set_on_url_changed(on_url_changed, state);
        ts_set_on_title_changed(on_title_changed, state);
        return ts_content_main(argc, argv);
    }
}
