//! One SVG line chart per operation: the time per operation against the width, one panel per
//! build, one line per crate.
//!
//! The colors are the categorical slots of the reference palette of the dataviz method, in their
//! fixed order, with a light and a dark step each. Every crate also has its own marker shape.

use crate::Error;

/// The light and the dark color of each categorical slot, in the palette's fixed order.
const SLOTS: [(&str, &str); 8] = [
    ("#2a78d6", "#3987e5"),
    ("#eb6834", "#d95926"),
    ("#1baf7a", "#199e70"),
    ("#eda100", "#c98500"),
    ("#e87ba4", "#d55181"),
    ("#008300", "#008300"),
    ("#6250d6", "#9085e9"),
    ("#e34948", "#e66767"),
];

/// The marker outline of each slot, around `(0, 0)`.
const MARKERS: [&str; 8] = [
    "M0,-4.5A4.5,4.5 0 1 1 0,4.5A4.5,4.5 0 1 1 0,-4.5Z",
    "M-4,-4H4V4H-4Z",
    "M0,-5.5L5.5,0L0,5.5L-5.5,0Z",
    "M0,-5.5L5,4H-5Z",
    "M0,5.5L5,-4H-5Z",
    "M-1.8,-5.5H1.8V-1.8H5.5V1.8H1.8V5.5H-1.8V1.8H-5.5V-1.8H-1.8Z",
    "M0,-6L1.6,-2.1L5.7,-1.9L2.6,0.8L3.5,4.9L0,2.6L-3.5,4.9L-2.6,0.8L-5.7,-1.9L-1.6,-2.1Z",
    "M-4.6,-2.5L-2.5,-4.6L0,-2.1L2.5,-4.6L4.6,-2.5L2.1,0L4.6,2.5L2.5,4.6L0,2.1L-2.5,4.6\
     L-4.6,2.5L-2.1,0Z",
];

/// The chart's neutral colors in one mode.
struct Ink {
    surface: &'static str,
    ink: &'static str,
    secondary: &'static str,
    muted: &'static str,
    grid: &'static str,
    axis: &'static str,
}

const LIGHT: Ink = Ink {
    surface: "#fcfcfb",
    ink: "#0b0b0b",
    secondary: "#52514e",
    muted: "#898781",
    grid: "#e1e0d9",
    axis: "#c3c2b7",
};

const DARK: Ink = Ink {
    surface: "#1a1a19",
    ink: "#f0efec",
    secondary: "#c3c2b7",
    muted: "#898781",
    grid: "#2c2c2a",
    axis: "#383835",
};

const FONT: &str = "system-ui, -apple-system, \"Segoe UI\", sans-serif";
/// The average width of a character of the 12-pixel legend.
const CHARACTER_WIDTH: f64 = 6.3;

const WIDTH: f64 = 960.0;
const PAD: f64 = 24.0;
const COLUMN_GAP: f64 = 32.0;
const PANEL_WIDTH: f64 = (WIDTH - 2.0 * PAD - COLUMN_GAP) / 2.0;
const AXIS_WIDTH: f64 = 56.0;
const PANEL_TITLE: f64 = 24.0;
const PLOT_HEIGHT: f64 = 220.0;
const X_AXIS: f64 = 40.0;

/// The chart of one operation.
pub(crate) struct Chart<'a> {
    /// The title.
    pub(crate) title: String,
    /// The lines of the subtitle.
    pub(crate) notes: Vec<&'a str>,
    /// The widths along the horizontal axis.
    pub(crate) widths: &'a [u32],
    /// The crates, in the order of their colors.
    pub(crate) crates: &'a [&'a str],
    /// One panel per build.
    pub(crate) panels: Vec<Panel<'a>>,
}

/// The times of one build.
pub(crate) struct Panel<'a> {
    /// The build.
    pub(crate) build: &'a str,
    /// One point per case.
    pub(crate) points: Vec<Point<'a>>,
}

/// The time per operation of one crate at one width.
#[derive(Clone, Copy)]
pub(crate) struct Point<'a> {
    pub(crate) library: &'a str,
    pub(crate) bits: u32,
    pub(crate) nanoseconds: f64,
}

/// The SVG of `chart`.
pub(crate) fn svg(chart: &Chart<'_>) -> Result<String, Error> {
    if chart.crates.len() > SLOTS.len() {
        return Err(Error::Palette {
            crates: chart.crates.len(),
            slots: SLOTS.len(),
        });
    }
    let legend_y = PAD + 36.0 + 18.0 * chart.notes.len() as f64 + 8.0;
    let panels_top = legend_y + 22.0;
    let height = panels_top + PANEL_TITLE + PLOT_HEIGHT + X_AXIS + PAD;
    let mut parts = vec![
        format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{WIDTH}\" height=\"{height}\" \
             viewBox=\"0 0 {WIDTH} {height}\" role=\"img\" aria-labelledby=\"title description\">"
        ),
        format!("<title id=\"title\">{}</title>", escape(&chart.title)),
        format!(
            "<desc id=\"description\">{}</desc>",
            escape(&chart.notes.join(" "))
        ),
        format!("<style>{}</style>", style(chart.crates.len())),
        format!("<rect class=\"surface\" width=\"{WIDTH}\" height=\"{height}\" rx=\"8\"/>"),
        format!(
            "<text class=\"title\" x=\"{PAD}\" y=\"{}\">{}</text>",
            PAD + 14.0,
            escape(&chart.title)
        ),
    ];
    for (line, note) in chart.notes.iter().enumerate() {
        parts.push(format!(
            "<text class=\"subtitle\" x=\"{PAD}\" y=\"{}\">{}</text>",
            PAD + 36.0 + 18.0 * line as f64,
            escape(note)
        ));
    }
    parts.extend(legend(legend_y, chart.crates));
    let scale = LogScale::new(
        chart
            .panels
            .iter()
            .flat_map(|panel| panel.points.iter().map(|point| point.nanoseconds)),
    );
    for (column, panel) in chart.panels.iter().enumerate() {
        let left = PAD + column as f64 * (PANEL_WIDTH + COLUMN_GAP);
        parts.extend(draw_panel(chart, panel, left, panels_top, &scale)?);
    }
    parts.push("</svg>".to_owned());
    Ok(parts.join("\n") + "\n")
}

/// Shapes and type, the colors of the light mode, then the colors of the dark mode under
/// `prefers-color-scheme: dark`. The colors are plain values, which every SVG renderer reads.
fn style(slots: usize) -> String {
    let colors = |ink: &Ink, dark: bool| {
        let mut rules = format!(
            ".surface{{fill:{surface}}}.grid{{stroke:{grid}}}.axis{{stroke:{axis}}}\
             .title,.panel{{fill:{ink}}}.subtitle,.legend{{fill:{secondary}}}\
             .tick{{fill:{muted}}}.marker{{stroke:{surface}}}",
            surface = ink.surface,
            grid = ink.grid,
            axis = ink.axis,
            ink = ink.ink,
            secondary = ink.secondary,
            muted = ink.muted,
        );
        for (index, (light, dark_color)) in SLOTS.iter().take(slots).enumerate() {
            let color = if dark { dark_color } else { light };
            rules += &format!(".c{index} .line{{stroke:{color}}}.c{index} .marker{{fill:{color}}}");
        }
        rules
    };
    format!(
        ".grid,.axis{{stroke-width:1}}.title{{font:600 16px {FONT}}}\
         .subtitle,.legend{{font:12px {FONT}}}.panel{{font:600 13px {FONT}}}\
         .tick{{font:11px {FONT};font-variant-numeric:tabular-nums}}\
         .line{{fill:none;stroke-width:2;stroke-linejoin:round;stroke-linecap:round}}\
         .marker{{stroke-width:4;paint-order:stroke}}{}\
         @media (prefers-color-scheme: dark){{{}}}",
        colors(&LIGHT, false),
        colors(&DARK, true),
    )
}

/// One row of line keys and crate names at height `y`.
fn legend(y: f64, crates: &[&str]) -> Vec<String> {
    let mut parts = Vec::new();
    let mut x = PAD;
    for (index, (name, marker)) in crates.iter().zip(MARKERS).enumerate() {
        parts.push(format!(
            "<g class=\"c{index}\"><path class=\"line\" d=\"M{x},{y}h20\"/>\
             <path class=\"marker\" transform=\"translate({},{y})\" d=\"{marker}\"/></g>",
            x + 10.0,
        ));
        parts.push(format!(
            "<text class=\"legend\" x=\"{}\" y=\"{}\">{}</text>",
            x + 26.0,
            y + 4.0,
            escape(name)
        ));
        x += 26.0 + (CHARACTER_WIDTH * name.len() as f64).round() + 22.0;
    }
    parts
}

/// A base-10 logarithmic axis over whole decades around the values.
struct LogScale {
    /// The exponent of the bottom decade.
    low: i32,
    /// The exponent of the top decade.
    high: i32,
}

impl LogScale {
    fn new(values: impl Iterator<Item = f64>) -> Self {
        let (low, high) = values.fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), value| {
            (low.min(value), high.max(value))
        });
        let low = low.log10().floor() as i32;
        let high = (high.log10().ceil() as i32).max(low + 1);
        Self { low, high }
    }

    /// 0 at the bottom of the axis, 1 at the top.
    fn fraction(&self, value: f64) -> f64 {
        (value.log10() - f64::from(self.low)) / f64::from(self.high - self.low)
    }

    /// The ticks and their labels: every decade, and 2 and 5 times each decade on an axis of at
    /// most two decades. On an axis that reaches 10,000 the labels are compact (10K).
    fn ticks(&self) -> Vec<(f64, String)> {
        let steps: &[u32] = if self.high - self.low <= 2 {
            &[1, 2, 5]
        } else {
            &[1]
        };
        let compact = self.high >= 4;
        let mut ticks = Vec::new();
        for exponent in self.low..=self.high {
            for &step in steps {
                if exponent == self.high && step > 1 {
                    continue;
                }
                let value = f64::from(step) * 10f64.powi(exponent);
                ticks.push((value, tick_label(step, exponent, compact)));
            }
        }
        ticks
    }
}

/// `step * 10^exponent` as a clean number: in full below 10,000, compact on a compact axis.
fn tick_label(step: u32, exponent: i32, compact: bool) -> String {
    let Ok(power) = u32::try_from(exponent) else {
        let decimals = exponent.unsigned_abs() as usize;
        return format!("{:.decimals$}", f64::from(step) * 10f64.powi(exponent));
    };
    let value = u64::from(step) * 10u64.pow(power);
    if compact {
        for (suffix_power, suffix) in [(9, "G"), (6, "M"), (3, "K")] {
            if power >= suffix_power {
                return format!("{}{suffix}", value / 10u64.pow(suffix_power));
            }
        }
    }
    grouped(value)
}

/// `value` with a comma between groups of three digits.
pub(crate) fn grouped(value: u64) -> String {
    let digits = value.to_string();
    let mut text = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            text.push(',');
        }
        text.push(digit);
    }
    text
}

/// One panel at `left`, `top`: its title, its axes, and one line of markers per crate.
fn draw_panel(
    chart: &Chart<'_>,
    panel: &Panel<'_>,
    left: f64,
    top: f64,
    scale: &LogScale,
) -> Result<Vec<String>, Error> {
    let plot_left = left + AXIS_WIDTH;
    let plot_right = left + PANEL_WIDTH - 8.0;
    let plot_bottom = top + PANEL_TITLE + PLOT_HEIGHT;
    let band = (plot_right - plot_left) / chart.widths.len() as f64;
    let x_of = |position: usize| plot_left + (position as f64 + 0.5) * band;
    let y_of = |nanoseconds: f64| plot_bottom - scale.fraction(nanoseconds) * PLOT_HEIGHT;
    let mut parts = vec![format!(
        "<text class=\"panel\" x=\"{left}\" y=\"{}\">Time per operation (ns) · {}</text>",
        top + 13.0,
        escape(panel.build)
    )];
    for (value, label) in scale.ticks() {
        let y = y_of(value).round() + 0.5;
        parts.push(format!(
            "<path class=\"grid\" d=\"M{plot_left},{y}H{plot_right}\"/>\
             <text class=\"tick\" x=\"{}\" y=\"{}\" text-anchor=\"end\">{label}</text>",
            plot_left - 8.0,
            y + 4.0
        ));
    }
    parts.push(format!(
        "<path class=\"axis\" d=\"M{plot_left},{}H{plot_right}\"/>",
        plot_bottom + 0.5
    ));
    for (position, bits) in chart.widths.iter().enumerate() {
        parts.push(format!(
            "<text class=\"tick\" x=\"{:.1}\" y=\"{}\" text-anchor=\"middle\">{bits}</text>",
            x_of(position),
            plot_bottom + 16.0
        ));
    }
    parts.push(format!(
        "<text class=\"tick\" x=\"{:.1}\" y=\"{}\" text-anchor=\"middle\">bits</text>",
        (plot_left + plot_right) / 2.0,
        plot_bottom + 32.0
    ));
    for (index, (library, marker)) in chart.crates.iter().zip(MARKERS).enumerate() {
        // The crate's points as (position along the axis, point), in width order.
        let mut points = Vec::new();
        for point in panel
            .points
            .iter()
            .filter(|point| point.library == *library)
        {
            let position = chart
                .widths
                .iter()
                .position(|bits| *bits == point.bits)
                .ok_or_else(|| Error::missing(&format!("{} bits", point.bits), "the chart axis"))?;
            points.push((position, *point));
        }
        points.sort_by_key(|(position, _)| *position);
        parts.push(format!("<g class=\"c{index}\">"));
        // A line joins the points of adjacent widths; a width without a case breaks it.
        let mut runs: Vec<Vec<String>> = Vec::new();
        let mut previous = None;
        for (position, point) in &points {
            let coordinates = format!("{:.1},{:.1}", x_of(*position), y_of(point.nanoseconds));
            match (runs.last_mut(), previous) {
                (Some(run), Some(last)) if last + 1 == *position => run.push(coordinates),
                (Some(_) | None, _) => runs.push(vec![coordinates]),
            }
            previous = Some(*position);
        }
        for run in runs.iter().filter(|run| run.len() > 1) {
            parts.push(format!("<path class=\"line\" d=\"M{}\"/>", run.join("L")));
        }
        for (position, point) in &points {
            parts.push(format!(
                "<path class=\"marker\" transform=\"translate({:.1},{:.1})\" d=\"{marker}\">\
                 <title>{}, {} bits: {:.4} ns</title></path>",
                x_of(*position),
                y_of(point.nanoseconds),
                escape(library),
                point.bits,
                point.nanoseconds
            ));
        }
        parts.push("</g>".to_owned());
    }
    Ok(parts)
}

/// `text` with the XML special characters escaped.
fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
