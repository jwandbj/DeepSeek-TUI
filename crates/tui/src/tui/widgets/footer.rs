//! Footer bar widget displaying mode, status, model, and auxiliary chips.
//!
//! `FooterWidget` is a pure render of a [`FooterProps`] struct: all content
//! (labels, colors, span clusters) is computed once per redraw at a higher
//! level, then `FooterWidget::new(props).render(area, buf)` paints the
//! result. The widget owns no `App` knowledge; this mirrors the layout used
//! by `HeaderWidget` (and Codex's `bottom_pane::footer::Footer`).

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};
use unicode_width::UnicodeWidthStr;

use crate::palette;
use crate::tui::app::{App, AppMode};

use super::Renderable;

/// Pre-computed data the footer needs to render.
///
/// All fields are owned `String` / `Vec<Span<'static>>` values so the props
/// can be built once per redraw and then handed to a borrow-free widget.
#[derive(Debug, Clone)]
pub struct FooterProps {
    /// The current model identifier shown after the mode chip.
    pub model: String,
    /// `"agent"` / `"yolo"` / `"plan"` — the canonical setting label.
    pub mode_label: &'static str,
    /// Color used for the mode chip.
    pub mode_color: Color,
    /// Status label like `"ready"`, `"thinking ⌫"`, `"working"`. When the
    /// label equals `"ready"` the footer hides the status segment entirely.
    pub state_label: String,
    /// Color used for the status label.
    pub state_color: Color,
    /// Coherence chip spans (empty when no active intervention).
    pub coherence: Vec<Span<'static>>,
    /// Sub-agent count chip spans (empty when zero in-flight).
    pub agents: Vec<Span<'static>>,
    /// Reasoning-replay chip spans (empty when zero / not applicable).
    pub reasoning_replay: Vec<Span<'static>>,
    /// Cache-hit-rate chip spans (empty when no usage reported).
    pub cache: Vec<Span<'static>>,
    /// Session-cost chip spans (empty when below the display threshold).
    pub cost: Vec<Span<'static>>,
    /// Optional toast that, when present, replaces the left status line.
    pub toast: Option<FooterToast>,
    /// When `Some(frame_idx)`, the gap between the left status line and the
    /// right-hand chips is filled with an animated water-spout strip keyed
    /// off `frame_idx` (deterministic given the frame). `None` keeps the gap
    /// as plain whitespace, which is the idle/ready state.
    pub working_strip_frame: Option<u64>,
}

/// One frame of the footer's water-spout animation. `col` is the cell index
/// inside the strip, `width` the strip's total width, `frame` the discrete
/// frame counter. Returns the glyph that should appear in that cell on that
/// frame.
///
/// Visual: a single calm water line of `─` with one upward spout glyph that
/// drifts back and forth via a triangle-wave bounce. Minimal, artistic, and
/// purely deterministic so the test suite can pin a specific frame.
#[must_use]
pub fn footer_working_strip_glyph_at(col: usize, width: usize, frame: u64) -> char {
    if width == 0 {
        return ' ';
    }
    let w = width as i64;
    let frame = frame as i64;

    // Bounce a value that counts up forever between [0, w-1] using a
    // triangle wave so the spout rides back and forth instead of wrapping.
    let span = (w * 2).max(2);
    let t = frame.rem_euclid(span);
    let pos = if t < w { t } else { (span - 1) - t };

    let dist = (col as i64 - pos).abs();
    match dist {
        0 => '\u{257F}', // ╿  — vertical bar with a stronger top half: a spout standing up out of the surface
        1 => '\u{2576}', // ╶  — short stub on the spout's shoulder, like a splash
        _ => '\u{2500}', // ─  — calm water surface
    }
}

/// Build the per-frame water-spout string of `width` characters. Empty string
/// when width is 0. The result is the same visual width as requested (one
/// char per column for box-drawing chars) and is safe to drop into a `Span`
/// between the footer's left and right segments.
#[must_use]
pub fn footer_working_strip_string(width: usize, frame: u64) -> String {
    let mut out = String::with_capacity(width * 4);
    for col in 0..width {
        out.push(footer_working_strip_glyph_at(col, width, frame));
    }
    out
}

/// Build a "N agents" chip span list when there are sub-agents in flight.
/// Empty list when N == 0 hides the chip entirely. Singular for N == 1
/// reads naturally; plural otherwise.
#[must_use]
pub fn footer_agents_chip(running: usize) -> Vec<Span<'static>> {
    if running == 0 {
        return Vec::new();
    }
    let text = if running == 1 {
        "1 agent".to_string()
    } else {
        format!("{running} agents")
    };
    vec![Span::styled(
        text,
        Style::default().fg(palette::DEEPSEEK_SKY),
    )]
}

/// A status toast routed to the footer's left segment for a short time.
#[derive(Debug, Clone)]
pub struct FooterToast {
    pub text: String,
    pub color: Color,
}

impl FooterProps {
    /// Build footer props from common app state. Helpers in `tui/ui.rs`
    /// (e.g. `footer_state_label`, `footer_coherence_spans`) supply the
    /// pre-styled spans and labels — this constructor just bundles them.
    ///
    /// Argument fan-out is intentional: each input maps 1:1 to a piece of
    /// pre-computed footer content the caller resolved from `App`. Forcing
    /// these into a builder would obscure the call site without making the
    /// data flow any clearer.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn from_app(
        app: &App,
        toast: Option<FooterToast>,
        state_label: &'static str,
        state_color: Color,
        coherence: Vec<Span<'static>>,
        agents: Vec<Span<'static>>,
        reasoning_replay: Vec<Span<'static>>,
        cache: Vec<Span<'static>>,
        cost: Vec<Span<'static>>,
    ) -> Self {
        let (mode_label, mode_color) = mode_style(app.mode);
        Self {
            model: app.model.clone(),
            mode_label,
            mode_color,
            state_label: state_label.to_string(),
            state_color,
            coherence,
            agents,
            reasoning_replay,
            cache,
            cost,
            toast,
            working_strip_frame: None,
        }
    }
}

fn mode_style(mode: AppMode) -> (&'static str, Color) {
    let label = match mode {
        AppMode::Agent => "agent",
        AppMode::Yolo => "yolo",
        AppMode::Plan => "plan",
    };
    let color = match mode {
        AppMode::Agent => palette::MODE_AGENT,
        AppMode::Yolo => palette::MODE_YOLO,
        AppMode::Plan => palette::MODE_PLAN,
    };
    (label, color)
}

/// Pure-render footer. Build once per frame, then `render(area, buf)`.
pub struct FooterWidget {
    props: FooterProps,
}

impl FooterWidget {
    #[must_use]
    pub fn new(props: FooterProps) -> Self {
        Self { props }
    }

    fn auxiliary_spans(&self, max_width: usize) -> Vec<Span<'static>> {
        let parts: Vec<&Vec<Span<'static>>> = [
            &self.props.coherence,
            &self.props.agents,
            &self.props.reasoning_replay,
            &self.props.cache,
            &self.props.cost,
        ]
        .into_iter()
        .filter(|spans| !spans.is_empty())
        .collect();

        // Try to fit as many parts as possible, dropping from the end.
        for end in (0..=parts.len()).rev() {
            let mut combined: Vec<Span<'static>> = Vec::new();
            for (i, part) in parts[..end].iter().enumerate() {
                if i > 0 {
                    combined.push(Span::raw("  "));
                }
                combined.extend(part.iter().cloned());
            }
            if span_width(&combined) <= max_width {
                return combined;
            }
        }
        Vec::new()
    }

    fn toast_spans(toast: &FooterToast, max_width: usize) -> Vec<Span<'static>> {
        let truncated = truncate_to_width(&toast.text, max_width.max(1));
        vec![Span::styled(truncated, Style::default().fg(toast.color))]
    }

    fn status_line_spans(&self, max_width: usize) -> Vec<Span<'static>> {
        if max_width == 0 {
            return Vec::new();
        }

        let mode_label = self.props.mode_label;
        let sep = " \u{00B7} ";
        let show_status = self.props.state_label != "ready";
        let status_label = self.props.state_label.as_str();

        let fixed_width = mode_label.width()
            + sep.width()
            + if show_status {
                sep.width() + status_label.width()
            } else {
                0
            };

        if max_width <= mode_label.width() {
            return vec![Span::styled(
                truncate_to_width(mode_label, max_width),
                Style::default().fg(self.props.mode_color),
            )];
        }

        let model_budget = max_width.saturating_sub(fixed_width).max(1);
        let model_label = truncate_to_width(&self.props.model, model_budget);

        let mut spans = vec![
            Span::styled(
                mode_label.to_string(),
                Style::default().fg(self.props.mode_color),
            ),
            Span::styled(sep.to_string(), Style::default().fg(palette::TEXT_DIM)),
            Span::styled(model_label, Style::default().fg(palette::TEXT_HINT)),
        ];

        if show_status {
            spans.push(Span::styled(
                sep.to_string(),
                Style::default().fg(palette::TEXT_DIM),
            ));
            spans.push(Span::styled(
                status_label.to_string(),
                Style::default().fg(self.props.state_color),
            ));
        }

        spans
    }
}

impl Renderable for FooterWidget {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }
        let available_width = area.width as usize;
        if available_width == 0 {
            return;
        }

        let right_spans = self.auxiliary_spans(available_width);
        let right_width = span_width(&right_spans);
        let min_gap = if right_width > 0 { 2 } else { 0 };
        let max_left_width = available_width
            .saturating_sub(right_width)
            .saturating_sub(min_gap)
            .max(1);

        let left_spans = if let Some(toast) = self.props.toast.as_ref() {
            Self::toast_spans(toast, max_left_width)
        } else {
            self.status_line_spans(max_left_width)
        };

        let left_width = span_width(&left_spans);
        let spacer_width = available_width.saturating_sub(left_width + right_width);

        // When a turn is in flight, fill the gap with a thin animated water-
        // spout strip; otherwise the gap stays as plain whitespace.
        let spacer_span = match self.props.working_strip_frame {
            Some(frame) if spacer_width > 0 => Span::styled(
                footer_working_strip_string(spacer_width, frame),
                Style::default().fg(palette::DEEPSEEK_SKY),
            ),
            _ => Span::raw(" ".repeat(spacer_width)),
        };

        let mut all_spans = left_spans;
        all_spans.push(spacer_span);
        all_spans.extend(right_spans);

        let paragraph = Paragraph::new(Line::from(all_spans));
        paragraph.render(area, buf);
    }

    fn desired_height(&self, _width: u16) -> u16 {
        1
    }
}

fn span_width(spans: &[Span<'_>]) -> usize {
    spans.iter().map(|span| span.content.width()).sum()
}

fn truncate_to_width(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    if UnicodeWidthStr::width(text) <= max_width {
        return text.to_string();
    }
    if max_width <= 3 {
        return text.chars().take(max_width).collect();
    }

    let mut out = String::new();
    let mut width = 0usize;
    let limit = max_width.saturating_sub(3);
    for ch in text.chars() {
        let ch_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if width + ch_width > limit {
            break;
        }
        out.push(ch);
        width += ch_width;
    }
    out.push_str("...");
    out
}

#[cfg(test)]
mod tests {
    use super::{FooterProps, FooterWidget, Renderable};
    use crate::config::Config;
    use crate::palette;
    use crate::tui::app::{App, AppMode, TuiOptions};
    use ratatui::{style::Color, text::Span};
    use std::path::PathBuf;

    fn make_app() -> App {
        let options = TuiOptions {
            model: "deepseek-v4-flash".to_string(),
            workspace: PathBuf::from("."),
            allow_shell: false,
            use_alt_screen: true,
            use_mouse_capture: false,
            use_bracketed_paste: true,
            max_subagents: 1,
            skills_dir: PathBuf::from("."),
            memory_path: PathBuf::from("memory.md"),
            notes_path: PathBuf::from("notes.txt"),
            mcp_config_path: PathBuf::from("mcp.json"),
            use_memory: false,
            start_in_agent_mode: true,
            skip_onboarding: true,
            yolo: false,
            resume_session_id: None,
        };
        let mut app = App::new(options, &Config::default());
        // App::new may pick up `default_model` from a local user Settings
        // file, which overrides the option above. Pin the model explicitly
        // so these tests are independent of any host-side configuration.
        app.model = "deepseek-v4-flash".to_string();
        app
    }

    fn idle_props_for(app: &App) -> FooterProps {
        FooterProps::from_app(
            app,
            None,
            "ready",
            palette::TEXT_MUTED,
            Vec::<Span<'static>>::new(),
            Vec::<Span<'static>>::new(),
            Vec::<Span<'static>>::new(),
            Vec::<Span<'static>>::new(),
            Vec::<Span<'static>>::new(),
        )
    }

    #[test]
    fn from_app_idle_state_carries_ready_label_and_no_chips() {
        let app = make_app();
        let props = idle_props_for(&app);

        assert_eq!(props.state_label, "ready");
        assert_eq!(props.state_color, palette::TEXT_MUTED);
        assert_eq!(props.mode_label, "agent");
        assert_eq!(props.mode_color, palette::MODE_AGENT);
        assert_eq!(props.model, "deepseek-v4-flash");
        assert!(props.coherence.is_empty());
        assert!(props.agents.is_empty());
        assert!(props.cache.is_empty());
        assert!(props.cost.is_empty());
        assert!(props.reasoning_replay.is_empty());
        assert!(props.toast.is_none());
    }

    #[test]
    fn from_app_loading_state_uses_thinking_label_and_warning_color() {
        let app = make_app();
        let props = FooterProps::from_app(
            &app,
            None,
            "thinking \u{238B}",
            palette::STATUS_WARNING,
            Vec::<Span<'static>>::new(),
            Vec::<Span<'static>>::new(),
            Vec::<Span<'static>>::new(),
            Vec::<Span<'static>>::new(),
            Vec::<Span<'static>>::new(),
        );

        assert!(props.state_label.starts_with("thinking"));
        assert_eq!(props.state_color, palette::STATUS_WARNING);
    }

    // ---- agents chip wording ----
    #[test]
    fn footer_agents_chip_is_empty_when_no_agents_running() {
        let chip = super::footer_agents_chip(0);
        assert!(chip.is_empty(), "0 agents in flight → no chip");
    }

    #[test]
    fn footer_agents_chip_uses_singular_for_one() {
        let chip = super::footer_agents_chip(1);
        assert_eq!(chip.len(), 1);
        assert_eq!(chip[0].content.as_ref(), "1 agent");
    }

    #[test]
    fn footer_agents_chip_uses_plural_for_many() {
        let chip = super::footer_agents_chip(3);
        assert_eq!(chip.len(), 1);
        assert_eq!(chip[0].content.as_ref(), "3 agents");
    }

    #[test]
    fn footer_agents_chip_renders_into_widget() {
        let app = make_app();
        let agents = super::footer_agents_chip(2);
        let props = FooterProps::from_app(
            &app,
            None,
            "ready",
            palette::TEXT_MUTED,
            Vec::<Span<'static>>::new(),
            agents,
            Vec::<Span<'static>>::new(),
            Vec::<Span<'static>>::new(),
            Vec::<Span<'static>>::new(),
        );
        let widget = FooterWidget::new(props);
        let area = ratatui::layout::Rect::new(0, 0, 60, 1);
        let mut buf = ratatui::buffer::Buffer::empty(area);
        widget.render(area, &mut buf);
        let rendered: String = (0..area.width).map(|x| buf[(x, 0)].symbol()).collect();
        assert!(
            rendered.contains("2 agents"),
            "expected agents chip in render: {rendered:?}",
        );
    }

    #[test]
    fn from_app_mode_color_matches_mode_for_each_variant() {
        let mut app = make_app();
        let cases = [
            (AppMode::Agent, "agent", palette::MODE_AGENT),
            (AppMode::Yolo, "yolo", palette::MODE_YOLO),
            (AppMode::Plan, "plan", palette::MODE_PLAN),
        ];
        for (mode, expected_label, expected_color) in cases {
            app.mode = mode;
            let props = idle_props_for(&app);
            assert_eq!(
                props.mode_label, expected_label,
                "label mismatch for {mode:?}",
            );
            assert_eq!(
                props.mode_color, expected_color,
                "color mismatch for {mode:?}",
            );
        }
    }

    #[test]
    fn render_emits_mode_and_model_when_idle() {
        let app = make_app();
        let props = idle_props_for(&app);
        let widget = FooterWidget::new(props);

        let area = ratatui::layout::Rect::new(0, 0, 60, 1);
        let mut buf = ratatui::buffer::Buffer::empty(area);
        widget.render(area, &mut buf);

        let rendered: String = (0..area.width).map(|x| buf[(x, 0)].symbol()).collect();
        assert!(rendered.contains("agent"));
        assert!(rendered.contains("deepseek-v4-flash"));
        assert!(!rendered.contains("ready"));
    }

    #[test]
    fn working_strip_string_width_matches_request() {
        // The strip must produce exactly `width` characters per frame —
        // otherwise the spacer math in `FooterWidget::render` would
        // mis-align the right-hand chips. (Glyphs are all ASCII / Latin-1
        // so char count equals visual width here.)
        for width in [0usize, 1, 8, 60, 200] {
            let s = super::footer_working_strip_string(width, 7);
            assert_eq!(s.chars().count(), width, "width {width} mismatch");
        }
    }

    #[test]
    fn working_strip_glyph_is_deterministic_per_frame() {
        // Same (col, width, frame) → same glyph. Different `frame` values
        // produce different overall strings, which is what makes the
        // animation visible.
        let a = super::footer_working_strip_string(40, 1);
        let b = super::footer_working_strip_string(40, 1);
        assert_eq!(a, b, "deterministic given the same frame");
        let c = super::footer_working_strip_string(40, 2);
        assert_ne!(a, c, "advancing the frame must change the strip");
    }

    #[test]
    fn working_strip_renders_glyphs_only_when_frame_is_some() {
        // Idle: spacer is plain whitespace. Active: spacer contains the
        // box-drawing animation glyphs (`╿` spout, `╶` splash, `─` water
        // surface) and visibly differs from the idle render.
        let app = make_app();
        let mut props = idle_props_for(&app);

        let area = ratatui::layout::Rect::new(0, 0, 80, 1);
        let mut buf = ratatui::buffer::Buffer::empty(area);
        FooterWidget::new(props.clone()).render(area, &mut buf);
        let idle: String = (0..area.width).map(|x| buf[(x, 0)].symbol()).collect();

        props.working_strip_frame = Some(13);
        let mut buf2 = ratatui::buffer::Buffer::empty(area);
        FooterWidget::new(props).render(area, &mut buf2);
        let active: String = (0..area.width).map(|x| buf2[(x, 0)].symbol()).collect();

        assert_ne!(
            idle, active,
            "active footer must visibly differ from idle one"
        );
        assert!(
            active.contains('\u{257F}')
                || active.contains('\u{2576}')
                || active.contains('\u{2500}'),
            "active strip must contain at least one animation glyph: {active:?}",
        );
    }

    #[test]
    fn working_strip_spout_position_advances_with_frame() {
        // The single spout column must move between consecutive frames so
        // the animation reads as drift rather than a static pattern.
        let width = 16;
        let f0 = super::footer_working_strip_string(width, 1);
        let f1 = super::footer_working_strip_string(width, 2);
        let pos = |s: &str| s.chars().position(|c| c == '\u{257F}');
        let p0 = pos(&f0).expect("frame 1 has a spout");
        let p1 = pos(&f1).expect("frame 2 has a spout");
        assert_ne!(p0, p1, "spout column must advance between frames");
    }

    #[test]
    fn render_swaps_toast_for_status_line() {
        let app = make_app();
        let toast = super::FooterToast {
            text: "session saved".to_string(),
            color: Color::Green,
        };
        let props = FooterProps::from_app(
            &app,
            Some(toast),
            "ready",
            palette::TEXT_MUTED,
            Vec::<Span<'static>>::new(),
            Vec::<Span<'static>>::new(),
            Vec::<Span<'static>>::new(),
            Vec::<Span<'static>>::new(),
            Vec::<Span<'static>>::new(),
        );
        let widget = FooterWidget::new(props);

        let area = ratatui::layout::Rect::new(0, 0, 60, 1);
        let mut buf = ratatui::buffer::Buffer::empty(area);
        widget.render(area, &mut buf);

        let rendered: String = (0..area.width).map(|x| buf[(x, 0)].symbol()).collect();
        assert!(rendered.contains("session saved"));
        assert!(!rendered.contains("agent"));
        assert!(!rendered.contains("deepseek-v4-flash"));
    }
}
