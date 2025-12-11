use std::fmt::Write;

use super::snapshot_types::{CostSnapshot, TrendHistory};

/// Configuration for SVG graph generation
#[derive(Debug, Clone)]
pub struct SvgConfig {
    /// Width of SVG canvas
    pub width: u32,

    /// Height of SVG canvas
    pub height: u32,

    /// Padding around the graph
    pub padding: u32,

    /// Show grid lines
    pub show_grid: bool,

    /// Primary color for line
    pub line_color: String,

    /// Background color
    pub background_color: String,

    /// Show data points
    pub show_points: bool,

    /// Point radius
    pub point_radius: u32,
}

impl Default for SvgConfig {
    fn default() -> Self {
        Self {
            width: 800,
            height: 400,
            padding: 40,
            show_grid: true,
            line_color: "#2563eb".to_string(),
            background_color: "#ffffff".to_string(),
            show_points: true,
            point_radius: 4,
        }
    }
}

/// Generates SVG graphs from trend data
pub struct SvgGenerator {
    config: SvgConfig,
}

impl SvgGenerator {
    /// Create a new SVG generator with default config
    pub fn new() -> Self {
        Self {
            config: SvgConfig::default(),
        }
    }

    /// Create a new SVG generator with custom config
    pub fn with_config(config: SvgConfig) -> Self {
        Self { config }
    }

    /// Generate SVG graph from trend history
    pub fn generate(&self, history: &TrendHistory) -> Result<String, String> {
        if history.snapshots.is_empty() {
            return Err("No snapshots to visualize".to_string());
        }

        let mut svg = String::new();

        // SVG header
        writeln!(
            &mut svg,
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">"#,
            self.config.width, self.config.height, self.config.width, self.config.height
        ).unwrap();

        // Background
        writeln!(
            &mut svg,
            r#"  <rect width="{}" height="{}" fill="{}"/>"#,
            self.config.width, self.config.height, self.config.background_color
        )
        .unwrap();

        // Calculate data bounds
        let costs: Vec<f64> = history
            .snapshots
            .iter()
            .map(|s| s.total_monthly_cost)
            .collect();

        let min_cost = costs.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_cost = costs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let cost_range = max_cost - min_cost;

        // Add some padding to the range
        let y_min = (min_cost - cost_range * 0.1).max(0.0);
        let y_max = max_cost + cost_range * 0.1;
        let y_range = y_max - y_min;

        // Calculate graph area
        let graph_x = self.config.padding as f64;
        let graph_y = self.config.padding as f64;
        let graph_width = (self.config.width - 2 * self.config.padding) as f64;
        let graph_height = (self.config.height - 2 * self.config.padding) as f64;

        // Draw grid
        if self.config.show_grid {
            self.draw_grid(
                &mut svg,
                graph_x,
                graph_y,
                graph_width,
                graph_height,
                y_min,
                y_max,
            );
        }

        // Draw axes
        self.draw_axes(&mut svg, graph_x, graph_y, graph_width, graph_height);

        // Draw cost line
        self.draw_cost_line(
            &mut svg,
            &history.snapshots,
            graph_x,
            graph_y,
            graph_width,
            graph_height,
            y_min,
            y_range,
        );

        // Draw regression annotations
        self.draw_regression_annotations(
            &mut svg,
            &history.snapshots,
            graph_x,
            graph_y,
            graph_width,
            graph_height,
            y_min,
            y_range,
        );

        // Draw SLO violation annotations
        self.draw_slo_annotations(
            &mut svg,
            &history.snapshots,
            graph_x,
            graph_y,
            graph_width,
            graph_height,
            y_min,
            y_range,
        );

        // Draw labels
        self.draw_labels(
            &mut svg,
            graph_x,
            graph_y,
            graph_width,
            graph_height,
            y_min,
            y_max,
        );

        // SVG footer
        writeln!(&mut svg, "</svg>").unwrap();

        Ok(svg)
    }

    fn draw_grid(
        &self,
        svg: &mut String,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        _y_min: f64,
        _y_max: f64,
    ) {
        writeln!(
            svg,
            r##"  <g id="grid" stroke="#e5e7eb" stroke-width="1" opacity="0.5">"##
        )
        .unwrap();

        // Horizontal grid lines (5 lines)
        for i in 0..=5 {
            let y_pos = y + (i as f64 / 5.0) * height;
            writeln!(
                svg,
                r#"    <line x1="{}" y1="{}" x2="{}" y2="{}"/>"#,
                x,
                y_pos,
                x + width,
                y_pos
            )
            .unwrap();
        }

        // Vertical grid lines (based on snapshot count)
        let line_count = 6;
        for i in 0..=line_count {
            let x_pos = x + (i as f64 / line_count as f64) * width;
            writeln!(
                svg,
                r#"    <line x1="{}" y1="{}" x2="{}" y2="{}"/>"#,
                x_pos,
                y,
                x_pos,
                y + height
            )
            .unwrap();
        }

        writeln!(svg, "  </g>").unwrap();
    }

    fn draw_axes(&self, svg: &mut String, x: f64, y: f64, width: f64, height: f64) {
        writeln!(
            svg,
            r##"  <g id="axes" stroke="#374151" stroke-width="2">"##
        )
        .unwrap();

        // X axis
        writeln!(
            svg,
            r#"    <line x1="{}" y1="{}" x2="{}" y2="{}"/>"#,
            x,
            y + height,
            x + width,
            y + height
        )
        .unwrap();

        // Y axis
        writeln!(
            svg,
            r#"    <line x1="{}" y1="{}" x2="{}" y2="{}"/>"#,
            x,
            y,
            x,
            y + height
        )
        .unwrap();

        writeln!(svg, "  </g>").unwrap();
    }

    fn draw_cost_line(
        &self,
        svg: &mut String,
        snapshots: &[CostSnapshot],
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        y_min: f64,
        y_range: f64,
    ) {
        if snapshots.is_empty() {
            return;
        }

        writeln!(
            svg,
            r#"  <g id="cost-line" stroke="{}" stroke-width="3" fill="none">"#,
            self.config.line_color
        )
        .unwrap();

        // Build path
        let mut path = String::from("    <path d=\"");

        for (i, snapshot) in snapshots.iter().enumerate() {
            let x_pos = x + (i as f64 / (snapshots.len() - 1).max(1) as f64) * width;
            let y_pos = y + height - ((snapshot.total_monthly_cost - y_min) / y_range) * height;

            if i == 0 {
                write!(&mut path, "M {} {}", x_pos, y_pos).unwrap();
            } else {
                write!(&mut path, " L {} {}", x_pos, y_pos).unwrap();
            }
        }

        path.push_str("\"/>");
        writeln!(svg, "{}", path).unwrap();

        // Draw points if enabled
        if self.config.show_points {
            for (i, snapshot) in snapshots.iter().enumerate() {
                let x_pos = x + (i as f64 / (snapshots.len() - 1).max(1) as f64) * width;
                let y_pos = y + height - ((snapshot.total_monthly_cost - y_min) / y_range) * height;

                writeln!(
                    svg,
                    r#"    <circle cx="{}" cy="{}" r="{}" fill="{}"/>"#,
                    x_pos, y_pos, self.config.point_radius, self.config.line_color
                )
                .unwrap();
            }
        }

        writeln!(svg, "  </g>").unwrap();
    }

    fn draw_regression_annotations(
        &self,
        svg: &mut String,
        snapshots: &[CostSnapshot],
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        y_min: f64,
        y_range: f64,
    ) {
        writeln!(svg, r#"  <g id="regressions">"#).unwrap();

        for (i, snapshot) in snapshots.iter().enumerate() {
            if !snapshot.regressions.is_empty() {
                let x_pos = x + (i as f64 / (snapshots.len() - 1).max(1) as f64) * width;
                let y_pos = y + height - ((snapshot.total_monthly_cost - y_min) / y_range) * height;

                // Draw warning marker
                writeln!(
                    svg,
                    r##"    <circle cx="{}" cy="{}" r="8" fill="#ef4444" opacity="0.8"/>"##,
                    x_pos, y_pos
                )
                .unwrap();

                // Draw warning icon (!)
                writeln!(
                    svg,
                    r#"    <text x="{}" y="{}" text-anchor="middle" dominant-baseline="middle" font-size="12" font-weight="bold" fill="white">!</text>"#,
                    x_pos, y_pos
                ).unwrap();
            }
        }

        writeln!(svg, "  </g>").unwrap();
    }

    fn draw_slo_annotations(
        &self,
        svg: &mut String,
        snapshots: &[CostSnapshot],
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        y_min: f64,
        y_range: f64,
    ) {
        writeln!(svg, r#"  <g id="slo-violations">"#).unwrap();

        for (i, snapshot) in snapshots.iter().enumerate() {
            if !snapshot.slo_violations.is_empty() {
                let x_pos = x + (i as f64 / (snapshots.len() - 1).max(1) as f64) * width;
                let y_pos = y + height - ((snapshot.total_monthly_cost - y_min) / y_range) * height;

                // Draw SLO violation marker (different color from regression)
                writeln!(
                    svg,
                    r##"    <rect x="{}" y="{}" width="12" height="12" fill="#f59e0b" opacity="0.8" transform="rotate(45 {} {})"/>"##,
                    x_pos - 6.0, y_pos - 18.0, x_pos, y_pos - 12.0
                ).unwrap();
            }
        }

        writeln!(svg, "  </g>").unwrap();
    }

    fn draw_labels(
        &self,
        svg: &mut String,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        y_min: f64,
        y_max: f64,
    ) {
        writeln!(
            svg,
            r##"  <g id="labels" font-family="monospace" font-size="12" fill="#374151">"##
        )
        .unwrap();

        // Y-axis labels (cost values)
        for i in 0..=5 {
            let value = y_min + (y_max - y_min) * (i as f64 / 5.0);
            let y_pos = y + height - (i as f64 / 5.0) * height;

            writeln!(
                svg,
                r#"    <text x="{}" y="{}" text-anchor="end" dominant-baseline="middle">${:.0}</text>"#,
                x - 5.0,
                y_pos,
                value
            ).unwrap();
        }

        // Axis titles
        writeln!(
            svg,
            r#"    <text x="{}" y="{}" text-anchor="middle" font-weight="bold">Time</text>"#,
            x + width / 2.0,
            y + height + 30.0
        )
        .unwrap();

        writeln!(
            svg,
            r#"    <text x="{}" y="{}" text-anchor="middle" font-weight="bold" transform="rotate(-90 {} {})">Monthly Cost ($)</text>"#,
            x - 30.0,
            y + height / 2.0,
            x - 30.0,
            y + height / 2.0
        ).unwrap();

        writeln!(svg, "  </g>").unwrap();
    }
}

impl Default for SvgGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svg_generator_creation() {
        let generator = SvgGenerator::new();
        assert_eq!(generator.config.width, 800);
        assert_eq!(generator.config.height, 400);
    }

    #[test]
    fn test_generate_empty_history() {
        let generator = SvgGenerator::new();
        let history = TrendHistory::new();

        let result = generator.generate(&history);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_with_snapshots() {
        let generator = SvgGenerator::new();
        let mut history = TrendHistory::new();

        history.add_snapshot(CostSnapshot::new("snap-001".to_string(), 1000.0));
        history.add_snapshot(CostSnapshot::new("snap-002".to_string(), 1200.0));
        history.add_snapshot(CostSnapshot::new("snap-003".to_string(), 1100.0));

        let result = generator.generate(&history);
        assert!(result.is_ok());

        let svg = result.unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("cost-line"));
    }

    #[test]
    fn test_custom_config() {
        let config = SvgConfig {
            width: 1000,
            height: 500,
            line_color: "#ff0000".to_string(),
            ..Default::default()
        };

        let generator = SvgGenerator::with_config(config);
        assert_eq!(generator.config.width, 1000);
        assert_eq!(generator.config.line_color, "#ff0000");
    }
}
