use crate::core::models::LogAnalysisSummary;
use egui::{Color32, RichText, Ui};
use egui_plot::{Bar, BarChart, Legend, Line, Plot, PlotPoints};

pub fn render_charts(ui: &mut Ui, summary: &LogAnalysisSummary) {
    if summary.endpoints.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.label(RichText::new("No endpoints to chart").size(16.0).color(Color32::GRAY));
        });
        return;
    }

    ui.heading(RichText::new("Top 10 Endpoints Latency Comparison (ms)").color(Color32::WHITE));

    // Sort endpoints by calls
    let mut sorted_eps = summary.endpoints.clone();
    sorted_eps.sort_by(|a, b| b.total_calls.cmp(&a.total_calls));
    sorted_eps.truncate(10);

    let mut bars = Vec::new();
    for (i, ep) in sorted_eps.iter().enumerate() {
        bars.push(Bar::new(i as f64, ep.p95_ms as f64).fill(Color32::from_rgb(99, 102, 241)).name(&ep.path));
    }

    let chart = BarChart::new(bars).width(0.6).name("p95 Latency");

    Plot::new("latency_plot")
        .legend(Legend::default())
        .height(300.0)
        .show(ui, |plot_ui| {
            plot_ui.bar_chart(chart);
        });

    ui.add_space(20.0);
    ui.heading(RichText::new("Overall Response Time Percentiles (p50, p90, p95, p99)").color(Color32::WHITE));

    let points = vec![
        [50.0, summary.overall_p50_ms as f64],
        [90.0, summary.overall_p90_ms as f64],
        [95.0, summary.overall_p95_ms as f64],
        [99.0, summary.overall_p99_ms as f64],
    ];

    let line = Line::new(PlotPoints::from(points))
        .color(Color32::from_rgb(34, 197, 94))
        .name("Percentile Curve (ms)");

    Plot::new("percentile_plot")
        .legend(Legend::default())
        .height(250.0)
        .show(ui, |plot_ui| {
            plot_ui.line(line);
        });
}
