use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AnyElement, Context, Div, FocusHandle, FontWeight, Hsla, InteractiveElement as _, IntoElement, KeyDownEvent,
    ParentElement, Render, Rems, SharedString, StatefulInteractiveElement as _, Styled, Window, div, px, rems,
};
use gpui_omarchy::{ActiveTheme, Theme, empty_state, keycap};
use vb_thread::{Body, Claim, Field, Handoff, Placement, Proposal, Source, Take, Thread};
use voice_brief_desktop::store::{Session, Store};

const SPACE_XS: Rems = Rems(0.25);
const SPACE_SM: Rems = Rems(0.5);
const SPACE_MD: Rems = Rems(0.75);
const SPACE_LG: Rems = Rems(1.0);
const SPACE_XL: Rems = Rems(1.5);
const TEXT_CAPTION: Rems = Rems(0.6875);
const TEXT_BODY: Rems = Rems(0.8125);
const TEXT_TITLE: Rems = Rems(0.9375);
const POLL: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, PartialEq, Eq)]
enum Follow {
    Newest,
    Pinned(PathBuf),
}

pub struct Desk {
    root: Option<PathBuf>,
    store: Store,
    sessions: Vec<Session>,
    follow: Follow,
    pub focus: FocusHandle,
}

impl Desk {
    pub fn new(root: Option<PathBuf>, cx: &mut Context<'_, Self>) -> Self {
        let mut desk: Self = Self { root, store: Store::default(), sessions: Vec::new(), follow: Follow::Newest, focus: cx.focus_handle() };
        desk.rescan();
        Self::start_polling(cx);
        desk
    }

    fn rescan(&mut self) -> bool {
        let fresh: Vec<Session> = self.root.as_deref().map(|r| self.store.scan(r)).unwrap_or_default();
        let changed: bool = fresh != self.sessions;
        self.sessions = fresh;
        changed
    }

    fn start_polling(cx: &Context<'_, Self>) {
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(POLL).await;
                let alive: bool = this
                    .update(cx, |this, cx| {
                        if this.rescan() {
                            cx.notify();
                        }
                    })
                    .is_ok();
                if !alive {
                    break;
                }
            }
        })
        .detach();
    }

    fn shown(&self) -> Option<usize> {
        match &self.follow {
            Follow::Newest => (!self.sessions.is_empty()).then_some(0),
            Follow::Pinned(path) => self.sessions.iter().position(|s| &s.path == path).or_else(|| (!self.sessions.is_empty()).then_some(0)),
        }
    }

    fn step_session(&mut self, older: bool) {
        let Some(at) = self.shown() else { return };
        let next: usize = if older { (at + 1).min(self.sessions.len() - 1) } else { at.saturating_sub(1) };
        self.follow = if next == 0 { Follow::Newest } else { Follow::Pinned(self.sessions[next].path.clone()) };
    }

    fn on_key(&mut self, event: &KeyDownEvent, cx: &mut Context<'_, Self>) {
        let key: &str = event.keystroke.key.as_str();
        match key {
            "[" | "left" | "h" => self.step_session(true),
            "]" | "right" | "l" => self.step_session(false),
            "home" | "0" => self.follow = Follow::Newest,
            _ => return,
        }
        cx.notify();
    }
}

fn ago(started: u64) -> String {
    let now: u64 = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let mins: u64 = now.saturating_sub(started) / 60;
    match mins {
        0 => "started just now".to_owned(),
        1..=59 => format!("started {mins} min ago"),
        60..=1439 => format!("started {} h ago", mins / 60),
        _ => format!("started {} d ago", mins / 1440),
    }
}

enum Stage {
    Ended,
    Listening,
    Placing,
    Ready,
    Failed,
}

fn stage(session: &Session) -> Stage {
    match (&session.thread.placement, session.is_alive()) {
        (Placement::Placing { .. }, _) => Stage::Placing,
        (Placement::Failed { .. }, _) => Stage::Failed,
        (Placement::Proposed(_), _) => Stage::Ready,
        (Placement::Idle, true) => Stage::Listening,
        (Placement::Idle, false) => Stage::Ended,
    }
}

fn stage_look(s: &Stage, t: &Theme) -> (Hsla, &'static str) {
    match s {
        Stage::Ended => (t.secondary, "ended"),
        Stage::Listening => (t.success, "listening"),
        Stage::Placing => (t.warning, "placing"),
        Stage::Ready => (t.accent, "proposal ready"),
        Stage::Failed => (t.danger, "placement failed"),
    }
}

fn dot(color: Hsla) -> Div {
    div().size(rems(0.5)).rounded_full().bg(color).flex_shrink_0()
}

fn caption(label: impl Into<SharedString>, t: &Theme) -> Div {
    div().text_size(TEXT_CAPTION).text_color(t.secondary).font_weight(FontWeight::MEDIUM).child(label.into())
}

fn header(session: &Session, at: usize, count: usize, t: &Theme) -> Div {
    let (color, label) = stage_look(&stage(session), t);
    div()
        .flex()
        .items_center()
        .gap(SPACE_MD)
        .px(SPACE_XL)
        .py(SPACE_LG)
        .child(dot(color))
        .child(div().text_size(TEXT_TITLE).text_color(t.bright).font_weight(FontWeight::SEMIBOLD).child(session.project_name().to_owned()))
        .child(div().text_color(color).child(label))
        .child(div().flex_1())
        .child(div().text_color(t.secondary).child(ago(session.started)))
        .child(div().text_color(t.secondary).font_family(t.mono_font.clone()).child(format!("{} / {count}", at + 1)))
}

fn take_row(take: &Take, t: &Theme) -> Div {
    let marks: Vec<(String, Hsla)> = [
        take.sent.then(|| ("sent".to_owned(), t.accent)),
        take.corrected.then(|| ("edited".to_owned(), t.secondary)),
        (!take.clean).then(|| ("cut off".to_owned(), t.warning)),
    ]
    .into_iter()
    .flatten()
    .collect();
    div()
        .flex()
        .gap(SPACE_MD)
        .py(SPACE_SM)
        .when(take.deleted, |d| d.opacity(0.4))
        .child(div().w(rems(1.75)).flex_shrink_0().font_family(t.mono_font.clone()).text_color(t.secondary).child(take.id.clone()))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(SPACE_XS)
                .flex_1()
                .min_w_0()
                .child(div().text_color(t.foreground).when(take.deleted, |d| d.line_through()).child(take.text.clone()))
                .when(!marks.is_empty(), |d| {
                    d.child(div().flex().gap(SPACE_SM).text_size(TEXT_CAPTION).children(marks.into_iter().map(|(m, c)| div().text_color(c).child(m))))
                }),
        )
}

fn takes_column(thread: &Thread, t: &Theme) -> impl IntoElement {
    let live: usize = thread.live_takes();
    let title: String = match live {
        1 => "1 take".to_owned(),
        n => format!("{n} takes"),
    };
    div()
        .id("takes")
        .flex()
        .flex_col()
        .w(rems(19.0))
        .flex_shrink_0()
        .h_full()
        .overflow_y_scroll()
        .px(SPACE_XL)
        .pb(SPACE_LG)
        .child(caption(title, t).pb(SPACE_SM))
        .when(thread.takes.is_empty(), |d| d.child(div().text_color(t.secondary).child("Dictate with /voice in this Claude Code session.")))
        .children(thread.takes.iter().map(|k| take_row(k, t)))
}

fn source_line(claim: &Claim, thread: &Thread, t: &Theme) -> Div {
    let line: Div = div().text_size(TEXT_CAPTION);
    match &claim.source {
        Source::Said(spans) => {
            let quotes: Vec<String> = spans.iter().map(|s| format!("{} “{s}”", thread.quoted_from(s).unwrap_or("?"))).collect();
            line.text_color(t.secondary).child(format!("said {}", quotes.join(" · ")))
        }
        Source::Added(reason) => line.text_color(t.warning).child(format!("added · {reason}")),
    }
}

fn field_block(field: &Field, thread: &Thread, t: &Theme) -> Div {
    let body: AnyElement = match &field.body {
        Body::Claims(claims) => div()
            .flex()
            .flex_col()
            .gap(SPACE_SM)
            .children(claims.iter().map(|c| div().flex().flex_col().gap(SPACE_XS).child(div().text_color(t.foreground).child(c.text.clone())).child(source_line(c, thread, t))))
            .into_any_element(),
        Body::Asked(question) => div().flex().gap(SPACE_SM).text_color(t.warning).child(div().font_weight(FontWeight::BOLD).child("?")).child(div().italic().child(question.clone())).into_any_element(),
        Body::Missing => div().text_color(t.danger).child("missing from the reply").into_any_element(),
    };
    div().flex().flex_col().gap(SPACE_XS).py(SPACE_MD).border_t_1().border_color(t.border).child(caption(field.title, t)).child(body)
}

fn proposal_view(lead: String, lead_color: Hsla, proposal: &Proposal, thread: &Thread, t: &Theme) -> Div {
    let from: String = match proposal.ctx.as_slice() {
        [] => String::new(),
        ctx => format!(" · from {}", ctx.join(" ")),
    };
    div()
        .flex()
        .flex_col()
        .child(div().flex().gap(SPACE_XS).pb(SPACE_SM).child(div().text_color(lead_color).font_weight(FontWeight::SEMIBOLD).child(lead)).child(div().text_color(t.secondary).child(from)))
        .children(proposal.fields.iter().map(|f| field_block(f, thread, t)))
}

fn placement_view(thread: &Thread, t: &Theme) -> Div {
    let notice = |text: String, color: Hsla| div().text_color(color).pb(SPACE_MD).child(text);
    let accepted: Option<Div> = thread.accepted.as_ref().map(|r| {
        let lead: String = match &r.handoff {
            Handoff::Pending => format!("Revision {} · accepted", r.number),
            Handoff::Done { .. } => format!("Revision {} · handed off", r.number),
        };
        proposal_view(lead, t.success, &r.proposal, thread, t)
    });
    match (&thread.placement, accepted) {
        (Placement::Proposed(p), _) => proposal_view(format!("Proposal · {} s", p.secs), t.accent, p, thread, t),
        (Placement::Placing { ctx }, accepted) => div().child(notice(format!("Placing {} …", ctx.join(", ")), t.warning)).children(accepted),
        (Placement::Failed { reason }, accepted) => div().child(notice(format!("Last placement failed: {reason}"), t.danger)).children(accepted),
        (Placement::Idle, Some(view)) => view,
        (Placement::Idle, None) => div().text_color(t.secondary).child("Nothing placed yet. Press Place in the Voice notes pane when the takes say enough."),
    }
}

fn hint(key: &'static str, label: &'static str, cx: &Context<'_, Desk>) -> Div {
    div().flex().items_center().gap(SPACE_XS).child(keycap(key, cx)).child(label)
}

fn footer(cx: &Context<'_, Desk>) -> Div {
    let t: &Theme = cx.omarchy();
    div()
        .flex()
        .items_center()
        .gap(SPACE_LG)
        .px(SPACE_XL)
        .py(SPACE_MD)
        .border_t_1()
        .border_color(t.border)
        .text_size(TEXT_CAPTION)
        .text_color(t.secondary)
        .child(hint("F1", "hide", cx))
        .child(hint("[", "older", cx))
        .child(hint("]", "newer", cx))
        .child(hint("0", "latest", cx))
        .child(div().flex_1())
        .child("voice-brief")
}

impl Render for Desk {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let t: Theme = cx.omarchy().clone();
        let body: AnyElement = match self.shown() {
            None => div()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .child(empty_state("No dictations yet", "Run /voice in a Claude Code session with the voice-notes plugin. Takes show up here as you speak.", cx))
                .into_any_element(),
            Some(at) => {
                let session: &Session = &self.sessions[at];
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .min_h_0()
                    .child(header(session, at, self.sessions.len(), &t))
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .min_h_0()
                            .child(takes_column(&session.thread, &t))
                            .child(div().w(px(1.0)).h_full().bg(t.border))
                            .child(div().id("placement").flex_1().min_w_0().h_full().overflow_y_scroll().px(SPACE_XL).pb(SPACE_LG).child(placement_view(&session.thread, &t))),
                    )
                    .into_any_element()
            }
        };
        div()
            .id("desk")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, e: &KeyDownEvent, _, cx| this.on_key(e, cx)))
            .size_full()
            .flex()
            .flex_col()
            .bg(t.background)
            .text_color(t.foreground)
            .font_family(t.font.clone())
            .text_size(TEXT_BODY)
            .child(body)
            .child(footer(cx))
    }
}
