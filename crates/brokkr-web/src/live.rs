//! The live pod-log tail of one stack (BROKKR-T-0348).
//!
//! The agent follows each pod's logs and sends every line to the broker,
//! which relays it at once to the subscribers of
//! `/api/v1/stacks/{id}/live`. This module opens that stream from the browser
//! and keeps the lines the Pod logs tab shows.
//!
//! - The frames are parsed with small local serde structs, not `brokkr-wire`:
//!   that crate depends on `brokkr-models`, which pulls diesel, and the
//!   console needs four fields. The host test `frames_match_the_wire_fixture`
//!   reads `crates/brokkr-wire/tests/fixtures/console_live_frames.json`, and a
//!   test in `brokkr-wire` keeps that fixture equal to what its serializer
//!   writes, so the two cannot drift silently.
//! - The browser authenticates with two subprotocols, `brokkr.v1` and
//!   `brokkr.pak.<token>`; the broker echoes only `brokkr.v1`. The token is
//!   never logged or shown.
//! - A refused upgrade (401/403) is not visible to the page: the socket only
//!   closes. So the socket never marks the session; REST stays the source of
//!   session truth (`api::note_status`).

use crate::models::PodLogDto;
use leptos::prelude::*;
use leptos::wasm_bindgen::closure::Closure;
use leptos::wasm_bindgen::JsCast;
use serde::Deserialize;
use std::collections::{HashSet, VecDeque};
use std::time::Duration;

/// The view keeps at most this many rows (lines and gaps). The broker's
/// history read gives 500 by default, so this holds the history plus some
/// minutes of a busy stack.
pub const MAX_LINES: usize = 2000;

/// A socket that stays open this long resets the backoff. A broker that
/// accepts and then drops the socket at once still backs off.
const STABLE_OPEN_MS: f64 = 5000.0;

/// The first retry delay and the cap.
const BACKOFF_BASE_MS: u64 = 1000;
const BACKOFF_CAP_MS: u64 = 30_000;

/// A `LogGap` frame: lines were dropped before they reached this view. The
/// agent sends one when it drops lines; the broker sends one (with a nil
/// agent) when this subscriber lags.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct GapDto {
    #[serde(default)]
    pub since_ts: Option<String>,
    #[serde(default)]
    pub dropped_count: u64,
    #[serde(default)]
    pub reason: String,
}

impl GapDto {
    /// The text of the gap row.
    pub fn text(&self) -> String {
        let reason = self.reason.replace('_', " ");
        let n = self.dropped_count;
        let lines = if n == 1 { "line is" } else { "lines are" };
        if reason.is_empty() {
            format!("Gap: {n} {lines} missing here.")
        } else {
            format!("Gap: {n} {lines} missing here ({reason}).")
        }
    }

    pub fn clock(&self) -> Option<String> {
        clock(self.since_ts.as_deref())
    }
}

/// `HH:MM:SSZ` of an RFC 3339 UTC time.
fn clock(ts: Option<&str>) -> Option<String> {
    ts.and_then(|t| t.get(11..19)).map(|t| format!("{t}Z"))
}

/// One frame of the live stream that the Pod logs tab uses.
#[derive(Debug, Clone, PartialEq)]
pub enum Frame {
    Line(PodLogDto),
    Gap(GapDto),
    /// Any other frame (`k8s_event`, and frames added later). The Kube events
    /// tab keeps its 5 s poll, so these are ignored.
    Other,
}

/// The envelope of `brokkr_wire::WsMessage`: adjacently tagged,
/// `{"type": "<snake_case>", "body": {...}}`.
#[derive(Deserialize)]
struct Envelope {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    body: serde_json::Value,
}

/// Parse one text frame. `None` when it is not a frame at all.
pub fn parse_frame(text: &str) -> Option<Frame> {
    let env: Envelope = serde_json::from_str(text).ok()?;
    Some(match env.kind.as_str() {
        "pod_log_line" => Frame::Line(serde_json::from_value(env.body).ok()?),
        "log_gap" => Frame::Gap(serde_json::from_value(env.body).ok()?),
        _ => Frame::Other,
    })
}

/// A time as `YYYY-MM-DDTHH:MM:SS.nnnnnnnnn`, so times sort as text.
///
/// The live frame carries the time from the pod (nanoseconds), and the
/// history carries the time the database kept (microseconds), and chrono
/// writes no fraction for a whole second: `12:00:00Z` sorts after
/// `12:00:00.5Z` as plain text.
fn ts_key(ts: &str) -> String {
    let ts = ts.trim_end_matches('Z').trim_end_matches("+00:00");
    let (secs, frac) = ts.split_once('.').unwrap_or((ts, ""));
    let digits: String = frac
        .chars()
        .take_while(char::is_ascii_digit)
        .take(9)
        .collect();
    format!("{secs}.{digits:0<9}")
}

/// The identity of a line: its container, its time to the microsecond (what
/// the database keeps) and its text. A line that arrives live and again in
/// the next history read has the same identity.
fn line_key(l: &PodLogDto) -> String {
    let ts = l.ts.as_deref().map(ts_key).unwrap_or_default();
    let micros = ts.get(..ts.len().saturating_sub(3)).unwrap_or("");
    format!(
        "{}/{}/{}\u{1f}{micros}\u{1f}{}",
        l.namespace, l.pod, l.container, l.line
    )
}

/// One row of the log view.
#[derive(Debug, Clone, PartialEq)]
pub enum Entry {
    Line(PodLogDto),
    Gap(GapDto),
}

impl Entry {
    fn sort_key(&self) -> String {
        match self {
            Entry::Line(l) => l.ts.as_deref().map(ts_key).unwrap_or_default(),
            Entry::Gap(g) => g.since_ts.as_deref().map(ts_key).unwrap_or_default(),
        }
    }
}

/// The rows of the Pod logs tab: the history first, then the live lines,
/// with no duplicates and at most `cap` rows (the oldest go first).
#[derive(Debug, Clone, PartialEq)]
pub struct LogBuffer {
    entries: VecDeque<Entry>,
    seen: HashSet<String>,
    cap: usize,
}

impl Default for LogBuffer {
    fn default() -> Self {
        Self::new(MAX_LINES)
    }
}

impl LogBuffer {
    pub fn new(cap: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            seen: HashSet::new(),
            cap: cap.max(1),
        }
    }

    pub fn entries(&self) -> impl Iterator<Item = &Entry> {
        self.entries.iter()
    }

    /// Append a live line. False when the view has it already.
    pub fn push_line(&mut self, line: PodLogDto) -> bool {
        if !self.seen.insert(line_key(&line)) {
            return false;
        }
        self.entries.push_back(Entry::Line(line));
        self.trim();
        true
    }

    /// Append a gap row.
    pub fn push_gap(&mut self, gap: GapDto) {
        self.entries.push_back(Entry::Gap(gap));
        self.trim();
    }

    /// Merge a history read. The broker answers newest first; the view shows
    /// oldest first. Lines the view has are skipped; when lines are added,
    /// the rows are put in time order again (a stable sort, so rows with the
    /// same time keep their order).
    pub fn merge_history(&mut self, newest_first: &[PodLogDto]) {
        let mut added = false;
        for l in newest_first.iter().rev() {
            if self.seen.insert(line_key(l)) {
                self.entries.push_back(Entry::Line(l.clone()));
                added = true;
            }
        }
        if added {
            self.entries
                .make_contiguous()
                .sort_by_cached_key(Entry::sort_key);
            self.trim();
        }
    }

    fn trim(&mut self) {
        while self.entries.len() > self.cap {
            if let Some(Entry::Line(l)) = self.entries.pop_front() {
                self.seen.remove(&line_key(&l));
            }
        }
    }
}

/// The delay before retry number `failures` (0 is the first retry): 1 s,
/// 2 s, 4 s, ... up to 30 s.
pub fn backoff(failures: u32) -> Duration {
    let ms = BACKOFF_BASE_MS.saturating_mul(1u64 << failures.min(16));
    Duration::from_millis(ms.min(BACKOFF_CAP_MS))
}

/// The failure count after a close: a socket that stayed open for
/// `STABLE_OPEN_MS` starts again at 0; a socket that did not open, or closed
/// soon after, counts one more failure.
pub fn failures_after_close(failures: u32, open_for_ms: Option<f64>) -> u32 {
    match open_for_ms {
        Some(ms) if ms >= STABLE_OPEN_MS => 0,
        _ => failures.saturating_add(1),
    }
}

/// The URL of the live stream on the same origin as the page.
pub fn live_url(page_protocol: &str, host: &str, stack_id: &str) -> String {
    let scheme = if page_protocol == "https:" {
        "wss:"
    } else {
        "ws:"
    };
    format!("{scheme}//{host}/api/v1/stacks/{stack_id}/live")
}

/// The state shown next to the Pod logs title.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveState {
    /// No stack is shown.
    Idle,
    Connecting,
    /// The stream is open: lines show when the pod writes them.
    Live,
    /// The stream closed; the console tries again after `in_secs` and reads
    /// the history every 5 s until then.
    Reconnecting {
        in_secs: u64,
    },
    /// The browser cannot open the stream; the 5 s history read is all.
    Polling,
}

impl LiveState {
    pub fn label(self) -> &'static str {
        match self {
            LiveState::Idle => "",
            LiveState::Connecting => "connecting",
            LiveState::Live => "live",
            LiveState::Reconnecting { .. } => "reconnecting",
            LiveState::Polling => "polling",
        }
    }

    pub fn hint(self) -> String {
        match self {
            LiveState::Idle => String::new(),
            LiveState::Connecting => "The console opens the live stream of this stack.".into(),
            LiveState::Live => "New lines show when the pod writes them.".into(),
            LiveState::Reconnecting { in_secs } => format!(
                "The live stream stopped. The console tries again in {in_secs} s and reads the logs every 5 s until then."
            ),
            LiveState::Polling => {
                "The live stream is not available. The console reads the logs every 5 s.".into()
            }
        }
    }

    /// Whether the stream delivers the lines, so the 5 s history read can
    /// stop.
    pub fn is_live(self) -> bool {
        self == LiveState::Live
    }
}

/// An open socket and the handlers it calls. Dropping it removes the
/// handlers before it closes the socket, so the browser never calls a
/// dropped closure.
struct Conn {
    ws: web_sys::WebSocket,
    _on_open: Closure<dyn FnMut()>,
    _on_message: Closure<dyn FnMut(web_sys::MessageEvent)>,
    _on_close: Closure<dyn FnMut(web_sys::CloseEvent)>,
}

impl Drop for Conn {
    fn drop(&mut self) {
        self.ws.set_onopen(None);
        self.ws.set_onmessage(None);
        self.ws.set_onclose(None);
        let _ = self.ws.close();
    }
}

#[derive(Default)]
struct Inner {
    stack: Option<String>,
    conn: Option<Conn>,
    retry: Option<TimeoutHandle>,
    failures: u32,
    opened_at: Option<f64>,
    /// Bumped on each connect and stop: a handler of an older socket does
    /// nothing.
    generation: u64,
}

/// The live tail of the stack the Pod logs tab shows.
#[derive(Clone, Copy)]
pub struct LiveTail {
    inner: StoredValue<Inner, LocalStorage>,
    /// The rows of the view.
    pub lines: RwSignal<LogBuffer>,
    pub state: RwSignal<LiveState>,
    /// Called each time the stream opens: the caller reads the history once,
    /// so the lines written while the stream was down are not lost.
    on_open: Callback<()>,
}

impl LiveTail {
    /// A tail that stops when the calling component is cleaned up.
    pub fn new(on_open: Callback<()>) -> Self {
        let tail = Self {
            inner: StoredValue::new_local(Inner::default()),
            lines: RwSignal::new(LogBuffer::default()),
            state: RwSignal::new(LiveState::Idle),
            on_open,
        };
        on_cleanup(move || tail.stop());
        tail
    }

    /// Tail `stack`, or nothing. A new stack starts with an empty view.
    pub fn follow(self, stack: Option<String>) {
        let same = self
            .inner
            .try_with_value(|i| i.stack == stack)
            .unwrap_or(false);
        if same {
            return;
        }
        self.stop();
        self.lines.set(LogBuffer::default());
        let open = stack.is_some();
        self.inner.update_value(|i| {
            i.stack = stack;
            i.failures = 0;
        });
        if open {
            self.state.set(LiveState::Connecting);
            self.connect();
        }
    }

    /// Close the stream and cancel a retry.
    pub fn stop(self) {
        let _ = self.inner.try_update_value(|i| {
            i.generation += 1;
            i.stack = None;
            i.conn = None;
            i.opened_at = None;
            if let Some(h) = i.retry.take() {
                h.clear();
            }
        });
        let _ = self.state.try_set(LiveState::Idle);
    }

    fn current(self, generation: u64) -> bool {
        self.inner
            .try_with_value(|i| i.generation == generation)
            .unwrap_or(false)
    }

    fn connect(self) {
        let Some((stack, generation)) = self
            .inner
            .try_update_value(|i| {
                i.generation += 1;
                i.conn = None;
                i.retry = None;
                i.stack.clone().map(|s| (s, i.generation))
            })
            .flatten()
        else {
            return;
        };
        let Some(ws) = open_socket(&stack) else {
            self.state.set(LiveState::Polling);
            return;
        };

        let on_open = Closure::<dyn FnMut()>::new(move || {
            if !self.current(generation) {
                return;
            }
            self.inner
                .update_value(|i| i.opened_at = Some(js_sys::Date::now()));
            self.state.set(LiveState::Live);
            self.on_open.run(());
        });
        let on_message =
            Closure::<dyn FnMut(web_sys::MessageEvent)>::new(move |ev: web_sys::MessageEvent| {
                if !self.current(generation) {
                    return;
                }
                let Some(text) = ev.data().as_string() else {
                    return;
                };
                match parse_frame(&text) {
                    Some(Frame::Line(l)) => self.lines.update(|b| {
                        b.push_line(l);
                    }),
                    Some(Frame::Gap(g)) => self.lines.update(|b| b.push_gap(g)),
                    _ => {}
                }
            });
        let on_close =
            Closure::<dyn FnMut(web_sys::CloseEvent)>::new(move |_ev: web_sys::CloseEvent| {
                if !self.current(generation) {
                    return;
                }
                // The socket that calls this handler stays in `conn` until the
                // next connect replaces it: dropping a closure while it runs
                // is not allowed.
                let failures = self.inner.with_value(|i| {
                    let open_for = i.opened_at.map(|t| js_sys::Date::now() - t);
                    failures_after_close(i.failures, open_for)
                });
                let wait = backoff(failures.saturating_sub(1));
                let retry = set_timeout_with_handle(move || self.connect(), wait).ok();
                self.inner.update_value(|i| {
                    i.failures = failures;
                    i.opened_at = None;
                    i.retry = retry;
                });
                self.state.set(LiveState::Reconnecting {
                    in_secs: wait.as_secs(),
                });
            });
        ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));
        self.inner.update_value(|i| {
            i.conn = Some(Conn {
                ws,
                _on_open: on_open,
                _on_message: on_message,
                _on_close: on_close,
            })
        });
    }
}

/// Open the stream of `stack` on the page's origin, with the injected token
/// in the PAK subprotocol. `None` when the browser refuses to make the socket.
fn open_socket(stack: &str) -> Option<web_sys::WebSocket> {
    let location = web_sys::window()?.location();
    let url = live_url(&location.protocol().ok()?, &location.host().ok()?, stack);
    let protocols = js_sys::Array::new();
    protocols.push(&"brokkr.v1".into());
    if let Some(token) = crate::api::token() {
        protocols.push(&format!("brokkr.pak.{token}").into());
    }
    web_sys::WebSocket::new_with_str_sequence(&url, &protocols).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(ts: &str, text: &str) -> PodLogDto {
        PodLogDto {
            ts: Some(ts.into()),
            namespace: "payments".into(),
            pod: "api-1".into(),
            container: "api".into(),
            line: text.into(),
        }
    }

    fn texts(b: &LogBuffer) -> Vec<String> {
        b.entries()
            .map(|e| match e {
                Entry::Line(l) => l.line.clone(),
                Entry::Gap(g) => g.text(),
            })
            .collect()
    }

    #[test]
    fn frames_match_the_wire_fixture() {
        let fixture: Vec<serde_json::Value> = serde_json::from_str(include_str!(
            "../../brokkr-wire/tests/fixtures/console_live_frames.json"
        ))
        .expect("fixture");
        let frames: Vec<Frame> = fixture
            .iter()
            .map(|v| parse_frame(&v.to_string()).expect("frame"))
            .collect();
        let Frame::Line(l) = &frames[0] else {
            panic!("want a line, got {:?}", frames[0]);
        };
        assert_eq!(l.ts.as_deref(), Some("2026-10-07T12:04:53.123456789Z"));
        assert_eq!(l.source(), "payments/payments-api-7d9f4-x2k1/api");
        assert_eq!(l.line, "POST /charge 201 48ms");
        assert_eq!(l.clock().as_deref(), Some("12:04:53Z"));
        let Frame::Gap(g) = &frames[1] else {
            panic!("want a gap, got {:?}", frames[1]);
        };
        assert_eq!(g.dropped_count, 17);
        assert_eq!(g.reason, "buffer_full");
        assert_eq!(g.clock().as_deref(), Some("12:04:54Z"));
        assert_eq!(g.text(), "Gap: 17 lines are missing here (buffer full).");
    }

    #[test]
    fn other_frames_are_ignored_and_junk_is_none() {
        let ev = r#"{"type":"k8s_event","body":{"reason":"BackOff"}}"#;
        assert_eq!(parse_frame(ev), Some(Frame::Other));
        assert_eq!(parse_frame("not json"), None);
        assert_eq!(parse_frame(r#"{"type":"pod_log_line","body":7}"#), None);
    }

    #[test]
    fn a_line_seen_live_is_not_added_again_by_the_history() {
        let mut b = LogBuffer::new(10);
        // The history comes newest first; the view shows oldest first.
        b.merge_history(&[
            line("2026-10-07T12:00:02.000001Z", "two"),
            line("2026-10-07T12:00:01Z", "one"),
        ]);
        assert_eq!(texts(&b), ["one", "two"]);
        // Live: nanoseconds from the pod.
        assert!(b.push_line(line("2026-10-07T12:00:03.123456789Z", "three")));
        assert!(!b.push_line(line("2026-10-07T12:00:02.000001000Z", "two")));
        // The next history read has the database's microseconds.
        b.merge_history(&[
            line("2026-10-07T12:00:03.123456Z", "three"),
            line("2026-10-07T12:00:02.000001Z", "two"),
        ]);
        assert_eq!(texts(&b), ["one", "two", "three"]);
    }

    #[test]
    fn history_lines_missed_while_down_go_in_time_order() {
        let mut b = LogBuffer::new(10);
        b.push_line(line("2026-10-07T12:00:01Z", "one"));
        b.push_line(line("2026-10-07T12:00:04Z", "four"));
        b.merge_history(&[
            line("2026-10-07T12:00:04Z", "four"),
            line("2026-10-07T12:00:02.5Z", "two and a half"),
            line("2026-10-07T12:00:02Z", "two"),
        ]);
        assert_eq!(texts(&b), ["one", "two", "two and a half", "four"]);
    }

    #[test]
    fn the_buffer_keeps_the_newest_rows() {
        let mut b = LogBuffer::new(3);
        for n in 0..5 {
            b.push_line(line(&format!("2026-10-07T12:00:0{n}Z"), &n.to_string()));
        }
        assert_eq!(b.entries().count(), 3);
        assert_eq!(texts(&b), ["2", "3", "4"]);
        // A line that left the buffer is not remembered as seen.
        assert!(b.push_line(line("2026-10-07T12:00:00Z", "0")));
    }

    #[test]
    fn a_gap_is_a_row() {
        let mut b = LogBuffer::new(10);
        b.push_line(line("2026-10-07T12:00:01Z", "one"));
        b.push_gap(GapDto {
            since_ts: Some("2026-10-07T12:00:02Z".into()),
            dropped_count: 1,
            reason: "rate_limit".into(),
        });
        assert_eq!(
            texts(&b),
            ["one", "Gap: 1 line is missing here (rate limit)."]
        );
    }

    #[test]
    fn backoff_doubles_up_to_a_cap() {
        let secs: Vec<u64> = (0..8).map(|n| backoff(n).as_secs()).collect();
        assert_eq!(secs, [1, 2, 4, 8, 16, 30, 30, 30]);
        assert_eq!(backoff(u32::MAX).as_secs(), 30);
    }

    #[test]
    fn a_stable_socket_resets_the_backoff() {
        assert_eq!(failures_after_close(0, None), 1);
        assert_eq!(failures_after_close(3, Some(200.0)), 4);
        assert_eq!(failures_after_close(3, Some(60_000.0)), 0);
    }

    #[test]
    fn the_url_follows_the_page() {
        assert_eq!(
            live_url("http:", "localhost:3000", "s1"),
            "ws://localhost:3000/api/v1/stacks/s1/live"
        );
        assert_eq!(
            live_url("https:", "brokkr.example", "s1"),
            "wss://brokkr.example/api/v1/stacks/s1/live"
        );
    }

    #[test]
    fn times_sort_as_text_after_the_key() {
        assert!(ts_key("2026-10-07T12:00:00.5Z") > ts_key("2026-10-07T12:00:00Z"));
        assert_eq!(
            ts_key("2026-10-07T12:00:00+00:00"),
            "2026-10-07T12:00:00.000000000"
        );
    }
}
