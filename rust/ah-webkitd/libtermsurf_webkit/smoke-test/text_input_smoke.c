#include "libtermsurf_webkit.h"
#include "test_support.h"

#include <stdbool.h>
#include <stdint.h>
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
};

static void fail(const char *reason)
{
    fprintf(stderr, "TEXT_SMOKE_FAIL %s\n", reason);
    fflush(stderr);
    exit(1);
}

static void require_contains(const char *value, const char *first, const char *second)
{
    if (!value || !strstr(value, first) || (second && !strstr(value, second))) {
        fprintf(stderr, "title_mismatch title=%s first=%s second=%s\n",
            value ? value : "", first, second ? second : "");
        fail("page_oracle_mismatch");
    }
}

static void require_text(ts_web_contents_t view, const char *type, const char *text,
    int64_t selected_start, int64_t selected_length,
    int64_t replacement_start, int64_t replacement_length)
{
    if (!ts_forward_text_input(view, type, text, selected_start, selected_length,
            replacement_start, replacement_length))
        fail("text_input_rejected");
}

static void finish_peer_check(const char *title, void *user_data)
{
    struct State *state = user_data;
    require_contains(title, "\"peer\":\"B\"", "\"value\":\"A😀Z\"");

    if (ts_forward_text_input(state->view_a, "unknown", "x", -1, -1, -1, -1))
        fail("unknown_type_accepted");
    if (ts_forward_text_input(state->view_a, "commit", "x", -1, 0, -1, -1))
        fail("invalid_selected_range_accepted");
    if (ts_forward_text_input(state->view_a, "commit", "x", -1, -1, 0, -1))
        fail("invalid_replacement_range_accepted");

    ts_destroy_web_contents(state->view_a);
    ts_destroy_web_contents(state->view_b);
    ts_destroy_browser_context(state->context);
    state->view_a = NULL;
    state->view_b = NULL;
    state->context = NULL;
    printf("TEXT_SMOKE_PASS engine=webkit tabs=2 unicode=1 utf16_ranges=1 composition=1 cancellation=1 focus=1 peer_isolation=1 cleanup=1\n");
    ts_quit();
}

static void check_final_a(const char *title, void *user_data)
{
    struct State *state = user_data;
    require_contains(title, "AéZ ascii é é 日本語日本語", "\"focused\":true");
    ts_webkit_test_evaluate_javascript(
        state->view_b, "document.title", finish_peer_check, state);
}

static void query_final_a(void *user_data)
{
    struct State *state = user_data;
    ts_webkit_test_evaluate_javascript(
        state->view_a, "document.title", check_final_a, state);
}

static void check_focus_lost(const char *title, void *user_data)
{
    struct State *state = user_data;
    require_contains(title, "AéZ ascii é é 日本語日本語", "\"focused\":false");
    if (strstr(title, "insertFromComposition:focus-cancel"))
        fail("focus_cancel_committed");
    ts_set_focus(state->view_a, true);
    if (!ts_web_contents_is_focused(state->view_a))
        fail("focus_restore_not_applied");
    ts_webkit_test_post_delayed_task(0.2, query_final_a, state);
}

static void query_focus_lost(void *user_data)
{
    struct State *state = user_data;
    ts_webkit_test_evaluate_javascript(
        state->view_a, "document.title", check_focus_lost, state);
}

static void run_focus_cancel(void *user_data)
{
    struct State *state = user_data;
    require_text(state->view_a, "ime_start", "", 0, 0, -1, -1);
    require_text(state->view_a, "ime_update", "focus-cancel", 4, 2, -1, -1);
    require_text(state->view_a, "ime_cancel", "", -1, -1, -1, -1);
    ts_set_focus(state->view_a, false);
    if (ts_web_contents_is_focused(state->view_a))
        fail("focus_loss_not_applied");
    ts_webkit_test_post_delayed_task(0.2, query_focus_lost, state);
}

static void check_cancel(const char *title, void *user_data)
{
    struct State *state = user_data;
    require_contains(title, "AéZ ascii é é 日本語日本語", "\"composing\":false");
    if (strstr(title, "insertFromComposition:cancel-me"))
        fail("explicit_cancel_committed");
    ts_webkit_test_post_delayed_task(0.2, run_focus_cancel, state);
}

static void query_cancel(void *user_data)
{
    struct State *state = user_data;
    ts_webkit_test_evaluate_javascript(
        state->view_a, "document.title", check_cancel, state);
}

static void run_cancel(void *user_data)
{
    struct State *state = user_data;
    require_text(state->view_a, "ime_start", "", 0, 0, -1, -1);
    require_text(state->view_a, "ime_update", "cancel-me", 3, 2, -1, -1);
    require_text(state->view_a, "ime_cancel", "", -1, -1, -1, -1);
    ts_webkit_test_post_delayed_task(0.2, query_cancel, state);
}

static void check_composition_update(const char *title, void *user_data)
{
    struct State *state = user_data;
    require_contains(title, "compositionupdate::日本", "\"composing\":true");
    require_text(state->view_a, "ime_update", "日本語", 2, 1, -1, -1);
    require_text(state->view_a, "ime_commit", "日本語", 2, 1, -1, -1);
    ts_webkit_test_post_delayed_task(0.3, run_cancel, state);
}

static void start_composition(void *user_data)
{
    struct State *state = user_data;
    require_text(state->view_a, "ime_start", "", 0, 0, 18, 0);
    require_text(state->view_a, "ime_update", "日本", 1, 1, -1, -1);
    ts_webkit_test_evaluate_javascript(
        state->view_a, "document.title", check_composition_update, state);
}

static void check_commits(const char *title, void *user_data)
{
    struct State *state = user_data;
    require_contains(title, "AéZ ascii é é 日本語", "insertText");
    ts_webkit_test_post_delayed_task(0.2, start_composition, state);
}

static void run_commits(void *user_data)
{
    struct State *state = user_data;
    require_text(state->view_a, "commit", " ascii é é 日本語", -1, -1, -1, -1);
    require_text(state->view_a, "commit", "é", -1, -1, 1, 2);
    ts_webkit_test_evaluate_javascript(
        state->view_a, "document.title", check_commits, state);
}

static void check_initial_b(const char *title, void *user_data)
{
    struct State *state = user_data;
    require_contains(title, "\"peer\":\"B\"", "\"value\":\"A😀Z\"");
    ts_webkit_test_post_delayed_task(0.2, run_commits, state);
}

static void check_initial_a(const char *title, void *user_data)
{
    struct State *state = user_data;
    require_contains(title, "\"peer\":\"A\"", "\"value\":\"A😀Z\"");
    ts_webkit_test_evaluate_javascript(
        state->view_b, "document.title", check_initial_b, state);
}

static void start_sequence(void *user_data)
{
    struct State *state = user_data;
    ts_set_focus(state->view_a, true);
    ts_set_focus(state->view_b, true);
    if (!ts_web_contents_is_focused(state->view_a)
        || !ts_web_contents_is_focused(state->view_b))
        fail("logical_focus_missing");
    ts_webkit_test_evaluate_javascript(
        state->view_a, "document.title", check_initial_a, state);
}

static void on_loading_state(
    ts_web_contents_t view, const char *url, int loading, void *user_data)
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
        ts_webkit_test_post_delayed_task(0.5, start_sequence, state);
    }
}

static void on_initialized(void *user_data)
{
    struct State *state = user_data;
    state->context = ts_create_incognito_browser_context();
    state->view_a = ts_create_web_contents(state->context, state->url_a, 800, 600, false);
    state->view_b = ts_create_web_contents(state->context, state->url_b, 800, 600, false);
    if (!state->context || !state->view_a || !state->view_b
        || state->view_a == state->view_b)
        fail("distinct_views");
}

int main(int argc, const char **argv)
{
    if (argc != 3) {
        fprintf(stderr, "usage: %s <peer-a-url> <peer-b-url>\n", argv[0]);
        return 2;
    }
    struct State state = {
        .url_a = argv[1],
        .url_b = argv[2],
    };
    ts_set_on_initialized(on_initialized, &state);
    ts_set_on_loading_state(on_loading_state, &state);
    return ts_content_main(argc, argv);
}
