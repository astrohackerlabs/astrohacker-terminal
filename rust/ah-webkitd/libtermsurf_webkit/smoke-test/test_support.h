#ifndef TERMSURF_LIBTERMSURF_WEBKIT_TEST_SUPPORT_H
#define TERMSURF_LIBTERMSURF_WEBKIT_TEST_SUPPORT_H

#include "libtermsurf_webkit.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef void (*ts_webkit_test_eval_cb)(const char *result, void *user_data);
typedef void (*ts_webkit_test_task_cb)(void *user_data);

void ts_webkit_test_evaluate_javascript(
    ts_web_contents_t wc,
    const char *script,
    ts_webkit_test_eval_cb callback,
    void *user_data);

void ts_webkit_test_post_delayed_task(double seconds, ts_webkit_test_task_cb callback, void *user_data);

void ts_webkit_test_kill_web_content_process(ts_web_contents_t wc);
int ts_webkit_test_renderer_crash_delegate_count(void);
int ts_webkit_test_editing_action_for_key(
    int type, int keycode, const char *utf8, int modifiers, int is_pdf);
int ts_webkit_test_navigation_action_for_key(
    int type, int keycode, int modifiers, int is_loading, int escape_owned);
int ts_webkit_test_editing_responder_execution_count(void);
int ts_webkit_test_editing_pending_count(void);
/* 0 if pointer identity correctly misses, 1 if incorrectly consumed, -1 if unavailable. */
int ts_webkit_test_unrelated_editing_event_consumed(void);

/* 1 if host window ignoresMouseEvents, 0 if not, -1 if unavailable */
int ts_webkit_test_host_ignores_mouse_events(ts_web_contents_t wc);

#ifdef __cplusplus
}
#endif

#endif
