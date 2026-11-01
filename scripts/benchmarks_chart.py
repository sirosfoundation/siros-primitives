"""One SVG line chart per operation: time and instructions per operation against the width, for
each build and each crate.

The colors are the categorical slots of the reference palette of the dataviz method, in their
fixed order, with light and dark steps. Each crate also has its own marker shape.
"""

import math
from html import escape

# (light, dark) per categorical slot, in the palette's fixed order.
SLOTS = [
    ("#2a78d6", "#3987e5"),
    ("#eb6834", "#d95926"),
    ("#1baf7a", "#199e70"),
    ("#eda100", "#c98500"),
    ("#e87ba4", "#d55181"),
    ("#008300", "#008300"),
    ("#6250d6", "#9085e9"),
    ("#e34948", "#e66767"),
]
# Marker outlines around (0, 0), one per slot.
MARKERS = [
    "M0,-4.5A4.5,4.5 0 1 1 0,4.5A4.5,4.5 0 1 1 0,-4.5Z",
    "M-4,-4H4V4H-4Z",
    "M0,-5.5L5.5,0L0,5.5L-5.5,0Z",
    "M0,-5.5L5,4H-5Z",
    "M0,5.5L5,-4H-5Z",
    "M-1.8,-5.5H1.8V-1.8H5.5V1.8H1.8V5.5H-1.8V1.8H-5.5V-1.8H-1.8Z",
    "M0,-6L1.6,-2.1L5.7,-1.9L2.6,0.8L3.5,4.9L0,2.6L-3.5,4.9L-2.6,0.8L-5.7,-1.9L-1.6,-2.1Z",
    "M-4.6,-2.5L-2.5,-4.6L0,-2.1L2.5,-4.6L4.6,-2.5L2.1,0"
    "L4.6,2.5L2.5,4.6L0,2.1L-2.5,4.6L-4.6,2.5L-2.1,0Z",
]
LIGHT = {"surface": "#fcfcfb", "ink": "#0b0b0b", "ink-2": "#52514e", "muted": "#898781",
         "grid": "#e1e0d9", "axis": "#c3c2b7"}
DARK = {"surface": "#1a1a19", "ink": "#f0efec", "ink-2": "#c3c2b7", "muted": "#898781",
        "grid": "#2c2c2a", "axis": "#383835"}
FONT = 'system-ui, -apple-system, "Segoe UI", sans-serif'
# The average width of a character of the 12px legend, in pixels.
TEXT_WIDTH = 6.3
METRICS = [("instructions", "Instructions per operation"), ("time", "Time per operation (ns)")]

WIDTH = 960
PAD = 24
COLUMN_GAP = 32
ROW_GAP = 28
PANEL_WIDTH = (WIDTH - 2 * PAD - COLUMN_GAP) / 2
AXIS_WIDTH = 52
PANEL_TITLE = 24
PLOT_HEIGHT = 190
X_AXIS = 40
PANEL_HEIGHT = PANEL_TITLE + PLOT_HEIGHT + X_AXIS
PANELS_TOP = 108
HEIGHT = PANELS_TOP + 2 * PANEL_HEIGHT + ROW_GAP + PAD


def svg(title, note, builds, widths, series):
    """The chart of one operation, with `note` closing its subtitle. `series` lists
    `(crate, values)`, where `values` maps `(build, metric, bits)` to a value or `None`."""
    if len(series) > len(SLOTS):
        raise SystemExit(f"{len(series)} crates; the palette has {len(SLOTS)} slots")
    subtitle = " ".join(filter(None, [
        "Each point is one crate at one width. Log scales; BENCHMARKS.md lists the values.", note,
    ]))
    parts = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{WIDTH}" height="{HEIGHT:.0f}" '
        f'viewBox="0 0 {WIDTH} {HEIGHT:.0f}" role="img" aria-labelledby="title description">',
        f'<title id="title">{escape(title)}</title>',
        f'<desc id="description">{escape(subtitle)}</desc>',
        f"<style>{style(len(series))}</style>",
        f'<rect class="surface" width="{WIDTH}" height="{HEIGHT:.0f}" rx="8"/>',
        f'<text class="title" x="{PAD}" y="{PAD + 14}">{escape(title)}</text>',
        f'<text class="subtitle" x="{PAD}" y="{PAD + 36}">{escape(subtitle)}</text>',
        *legend([crate for crate, _ in series]),
    ]
    for row, (metric, label) in enumerate(METRICS):
        values = [value for _, data in series for (_, kind, _), value in data.items()
                  if kind == metric and value is not None]
        scale = LogScale(min(values), max(values))
        for column, build in enumerate(builds):
            left = PAD + column * (PANEL_WIDTH + COLUMN_GAP)
            top = PANELS_TOP + row * (PANEL_HEIGHT + ROW_GAP)
            parts += panel(left, top, f"{label} · {build}", scale, widths,
                           [(crate, [data[(build, metric, bits)] for bits in widths])
                            for crate, data in series])
    return "\n".join([*parts, "</svg>"]) + "\n"


def style(slots):
    """The stylesheet: shapes and type, then the colors of the light mode, then the colors of
    the dark mode inside `prefers-color-scheme: dark`. The colors are plain values, which every
    SVG renderer reads."""

    def colors(ink, mode):
        rules = [
            f".surface{{fill:{ink['surface']}}}",
            f".grid{{stroke:{ink['grid']}}}",
            f".axis{{stroke:{ink['axis']}}}",
            f".title,.panel{{fill:{ink['ink']}}}",
            f".subtitle,.legend{{fill:{ink['ink-2']}}}",
            f".tick{{fill:{ink['muted']}}}",
            f".marker{{stroke:{ink['surface']}}}",
        ]
        for index in range(slots):
            color = SLOTS[index][mode]
            rules.append(f".c{index} .line{{stroke:{color}}}.c{index} .marker{{fill:{color}}}")
        return "".join(rules)

    shapes = "".join([
        ".grid,.axis{stroke-width:1}",
        f".title{{font:600 16px {FONT}}}",
        f".subtitle,.legend{{font:12px {FONT}}}",
        f".panel{{font:600 13px {FONT}}}",
        f".tick{{font:11px {FONT};font-variant-numeric:tabular-nums}}",
        ".line{fill:none;stroke-width:2;stroke-linejoin:round;stroke-linecap:round}",
        ".marker{stroke-width:4;paint-order:stroke}",
    ])
    return f"{shapes}{colors(LIGHT, 0)}@media (prefers-color-scheme: dark){{{colors(DARK, 1)}}}"


def legend(crates):
    parts = []
    x = PAD
    y = PAD + 62
    for index, crate in enumerate(crates):
        parts.append(f'<g class="c{index}"><path class="line" d="M{x},{y}h20"/>'
                     f'<path class="marker" transform="translate({x + 10},{y})" '
                     f'd="{MARKERS[index]}"/></g>')
        parts.append(f'<text class="legend" x="{x + 26}" y="{y + 4}">{escape(crate)}</text>')
        x += 26 + round(TEXT_WIDTH * len(crate)) + 22
    return parts


class LogScale:
    """A base-10 logarithmic axis that spans whole decades around the values."""

    def __init__(self, low, high):
        self.low = 10 ** math.floor(math.log10(low))
        self.high = 10 ** math.ceil(math.log10(high))
        if self.high <= self.low:
            self.high = self.low * 10

    def fraction(self, value):
        """0 at the bottom of the axis, 1 at the top."""
        return math.log10(value / self.low) / math.log10(self.high / self.low)

    def ticks(self):
        decades = round(math.log10(self.high / self.low))
        steps = (1, 2, 5) if decades <= 2 else (1,)
        values = []
        for exponent in range(decades + 1):
            for step in steps:
                value = step * self.low * 10**exponent
                if value <= self.high * 1.0001:
                    values.append(value)
        compact = self.high >= 10_000
        return [(value, tick_label(value, compact)) for value in values]


def tick_label(value, compact):
    """A clean number: in full below 10,000, compact on an axis that reaches 10,000."""
    if value < 1:
        return f"{value:g}"
    if compact and value >= 1000:
        for divisor, suffix in ((1e9, "G"), (1e6, "M"), (1e3, "K")):
            if value >= divisor:
                return f"{value / divisor:g}{suffix}"
    return f"{value:,.0f}"


def panel(left, top, title, scale, widths, series):
    plot_left = left + AXIS_WIDTH
    plot_right = left + PANEL_WIDTH - 8
    plot_top = top + PANEL_TITLE
    plot_bottom = plot_top + PLOT_HEIGHT
    band = (plot_right - plot_left) / len(widths)
    parts = [f'<text class="panel" x="{left}" y="{top + 13}">{escape(title)}</text>']
    for value, label in scale.ticks():
        y = round(plot_bottom - scale.fraction(value) * PLOT_HEIGHT) + 0.5
        parts.append(f'<path class="grid" d="M{plot_left},{y}H{plot_right}"/>')
        parts.append(f'<text class="tick" x="{plot_left - 8}" y="{y + 4}" '
                     f'text-anchor="end">{label}</text>')
    parts.append(f'<path class="axis" d="M{plot_left},{plot_bottom + 0.5}H{plot_right}"/>')
    centers = [plot_left + (index + 0.5) * band for index in range(len(widths))]
    for x, bits in zip(centers, widths):
        parts.append(f'<text class="tick" x="{x:.1f}" y="{plot_bottom + 16}" '
                     f'text-anchor="middle">{bits}</text>')
    parts.append(f'<text class="tick" x="{(plot_left + plot_right) / 2:.1f}" '
                 f'y="{plot_bottom + 32}" text-anchor="middle">bits</text>')
    for index, (crate, values) in enumerate(series):
        points = [(x, plot_bottom - scale.fraction(value) * PLOT_HEIGHT, bits, value)
                  for x, bits, value in zip(centers, widths, values) if value is not None]
        group = [f'<g class="c{index}">']
        for run in runs(points, centers):
            if len(run) > 1:
                path = "L".join(f"{x:.1f},{y:.1f}" for x, y, _, _ in run)
                group.append(f'<path class="line" d="M{path}"/>')
        for x, y, bits, value in points:
            group.append(f'<path class="marker" transform="translate({x:.1f},{y:.1f})" '
                         f'd="{MARKERS[index]}"><title>{escape(crate)}, {bits} bits: '
                         f"{value:.4g}</title></path>")
        parts += [*group, "</g>"]
    return parts


def runs(points, centers):
    """`points` split where a width has no value, so that a line never bridges a gap."""
    groups = []
    previous = None
    for point in points:
        position = centers.index(point[0])
        if previous is None or position != previous + 1:
            groups.append([])
        groups[-1].append(point)
        previous = position
    return groups
