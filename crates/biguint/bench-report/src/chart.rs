//! One SVG line chart per operation: the instructions per operation against the width, one line
//! per crate, on a linear scale from zero.
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
const AXIS_WIDTH: f64 = 56.0;
const PLOT_HEIGHT: f64 = 300.0;
const X_AXIS: f64 = 40.0;

/// The chart of one operation.
pub(crate) struct Chart<'a> {
    /// The title.
    pub(crate) title: String,
    /// The line under the title.
    pub(crate) subtitle: &'a str,
    /// The widths along the horizontal axis.
    pub(crate) widths: &'a [u32],
    /// The crates, in the order of their colors.
    pub(crate) crates: &'a [&'a str],
    /// One point per counted case.
    pub(crate) points: Vec<Point<'a>>,
}

/// The instructions per operation of one crate at one width.
#[derive(Clone, Copy)]
pub(crate) struct Point<'a> {
    pub(crate) library: &'a str,
    pub(crate) bits: u32,
    pub(crate) instructions: f64,
}

/// The SVG of `chart`.
pub(crate) fn svg(chart: &Chart<'_>) -> Result<String, Error> {
    if chart.crates.len() > SLOTS.len() {
        return Err(Error::Palette {
            crates: chart.crates.len(),
            slots: SLOTS.len(),
        });
    }
    let scale = LinearScale::new(chart.points.iter().map(|point| point.instructions))
        .ok_or_else(|| Error::missing("positive count", &format!("the chart {}", chart.title)))?;
    let legend_y = PAD + 62.0;
    let plot_top = legend_y + 30.0;
    let height = plot_top + PLOT_HEIGHT + X_AXIS + PAD;
    let mut parts = vec![
        format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{WIDTH}\" height=\"{height}\" \
             viewBox=\"0 0 {WIDTH} {height}\" role=\"img\" aria-labelledby=\"title description\">"
        ),
        format!("<title id=\"title\">{}</title>", escape(&chart.title)),
        format!("<desc id=\"description\">{}</desc>", escape(chart.subtitle)),
        format!("<style>{}</style>", style(chart.crates.len())),
        format!("<rect class=\"surface\" width=\"{WIDTH}\" height=\"{height}\" rx=\"8\"/>"),
        format!(
            "<text class=\"title\" x=\"{PAD}\" y=\"{}\">{}</text>",
            PAD + 14.0,
            escape(&chart.title)
        ),
        format!(
            "<text class=\"subtitle\" x=\"{PAD}\" y=\"{}\">{}</text>",
            PAD + 36.0,
            escape(chart.subtitle)
        ),
    ];
    parts.extend(legend(legend_y, chart.crates));
    parts.extend(plot(chart, plot_top, &scale)?);
    parts.push("</svg>".to_owned());
    Ok(parts.join("\n") + "\n")
}

/// Shapes and type, the colors of the light mode, then the colors of the dark mode under
/// `prefers-color-scheme: dark`. The colors are plain values, which every SVG renderer reads.
fn style(slots: usize) -> String {
    let colors = |ink: &Ink, dark: bool| {
        let mut rules = format!(
            ".surface{{fill:{surface}}}.grid{{stroke:{grid}}}.axis{{stroke:{axis}}}\
             .title{{fill:{ink}}}.subtitle,.legend{{fill:{secondary}}}\
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
         .subtitle,.legend{{font:12px {FONT}}}\
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

/// A linear axis from zero to a clean number at or above every value, in about five steps of
/// 1, 2 or 5 times a power of ten.
struct LinearScale {
    /// The distance between two ticks.
    step: f64,
    /// The top of the axis, a whole number of steps.
    top: f64,
}

impl LinearScale {
    /// `None` unless the largest value is positive.
    fn new(values: impl Iterator<Item = f64>) -> Option<Self> {
        let largest = values.fold(0.0, f64::max);
        if largest <= 0.0 {
            return None;
        }
        let rough = largest / 5.0;
        let power = 10f64.powf(rough.log10().floor());
        let factor = match rough / power {
            normalized if normalized <= 1.0 => 1.0,
            normalized if normalized <= 2.0 => 2.0,
            normalized if normalized <= 5.0 => 5.0,
            _ => 10.0,
        };
        let step = factor * power;
        Some(Self {
            step,
            top: (largest / step).ceil() * step,
        })
    }

    /// 0 at the bottom of the axis, 1 at the top.
    fn fraction(&self, value: f64) -> f64 {
        value / self.top
    }

    /// Every step from zero to the top, with its label: in full below 10,000, and in thousands
    /// or millions (10K, 2M) on an axis that reaches 10,000 or 10,000,000.
    fn ticks(&self) -> Vec<(f64, String)> {
        let (unit, suffix) = if self.top >= 1e7 {
            (1e6, "M")
        } else if self.top >= 1e4 {
            (1e3, "K")
        } else {
            (1.0, "")
        };
        let unit_step = self.step / unit;
        let decimals = if unit_step >= 1.0 {
            0
        } else {
            (-unit_step.log10().floor()) as usize
        };
        let steps = (self.top / self.step).round() as u32;
        (0..=steps)
            .map(|index| {
                let value = f64::from(index) * self.step;
                let label = match (index, decimals) {
                    (0, _) => "0".to_owned(),
                    (_, 0) => format!("{}{suffix}", grouped((value / unit).round() as u64)),
                    (_, decimals) => format!("{:.decimals$}{suffix}", value / unit),
                };
                (value, label)
            })
            .collect()
    }
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

/// The plot below `top`: its axes, and one line of markers per crate.
fn plot(chart: &Chart<'_>, top: f64, scale: &LinearScale) -> Result<Vec<String>, Error> {
    let plot_left = PAD + AXIS_WIDTH;
    let plot_right = WIDTH - PAD - 8.0;
    let plot_bottom = top + PLOT_HEIGHT;
    let band = (plot_right - plot_left) / chart.widths.len() as f64;
    let x_of = |position: usize| plot_left + (position as f64 + 0.5) * band;
    let y_of = |instructions: f64| plot_bottom - scale.fraction(instructions) * PLOT_HEIGHT;
    let mut parts = Vec::new();
    for (value, label) in scale.ticks() {
        let y = y_of(value).round() + 0.5;
        // The baseline is the axis, drawn below.
        if value > 0.0 {
            parts.push(format!(
                "<path class=\"grid\" d=\"M{plot_left},{y}H{plot_right}\"/>"
            ));
        }
        parts.push(format!(
            "<text class=\"tick\" x=\"{}\" y=\"{}\" text-anchor=\"end\">{label}</text>",
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
        for point in chart
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
            let coordinates = format!("{:.1},{:.1}", x_of(*position), y_of(point.instructions));
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
                 <title>{}, {} bits: {} instructions per operation</title></path>",
                x_of(*position),
                y_of(point.instructions),
                escape(library),
                point.bits,
                instructions(point.instructions)
            ));
        }
        parts.push("</g>".to_owned());
    }
    Ok(parts)
}

/// A count per operation: one decimal below 100, whole and grouped from 100 up.
pub(crate) fn instructions(value: f64) -> String {
    if value >= 100.0 {
        grouped(value.round() as u64)
    } else {
        format!("{value:.1}")
    }
}

/// `text` with the XML special characters escaped.
fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
