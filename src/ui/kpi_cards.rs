use crate::core::models::LogAnalysisSummary;
use egui::{Align, Color32, CornerRadius, Layout, RichText, Sense, StrokeKind, Ui, UiBuilder, Vec2};

pub fn render_kpi_dashboard(ui: &mut Ui, summary: &LogAnalysisSummary) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(12.0, 0.0);

        let format_bytes = |b: u64| -> String {
            if b >= 1024 * 1024 * 1024 {
                format!("{:.2} GiB", b as f64 / (1024.0 * 1024.0 * 1024.0))
            } else if b >= 1024 * 1024 {
                format!("{:.2} MiB", b as f64 / (1024.0 * 1024.0))
            } else if b >= 1024 {
                format!("{:.2} KiB", b as f64 / 1024.0)
            } else {
                format!("{} B", b)
            }
        };

        kpi_card(
            ui,
            "Total Ingested",
            &format_bytes(summary.total_file_size_bytes),
            &format!("{} lines parsed in {} ms", summary.total_lines_parsed, summary.parse_duration_ms),
            Color32::from_rgb(59, 130, 246),
        );

        kpi_card(
            ui,
            "Matched HTTP Requests",
            &format!("{}", summary.matched_http_requests),
            &format!("{} unique endpoints", summary.endpoints.len()),
            Color32::from_rgb(34, 197, 94),
        );

        let error_color = if summary.overall_error_rate > 5.0 {
            Color32::from_rgb(239, 68, 68)
        } else if summary.overall_error_rate > 1.0 {
            Color32::from_rgb(245, 158, 11)
        } else {
            Color32::from_rgb(34, 197, 94)
        };

        kpi_card(
            ui,
            "Error Rate",
            &format!("{:.2}%", summary.overall_error_rate),
            &format!("{} error responses", summary.overall_error_count),
            error_color,
        );

        kpi_card(
            ui,
            "p95 Latency",
            &format!("{:.1} ms", summary.overall_p95_ms),
            &format!("p50: {:.1} ms | p99: {:.1} ms", summary.overall_p50_ms, summary.overall_p99_ms),
            Color32::from_rgb(168, 85, 247),
        );

        kpi_card(
            ui,
            "Cron Executions",
            &format!("{}", summary.total_cron_events),
            &format!("{} active cron jobs", summary.cron_jobs.len()),
            Color32::from_rgb(236, 72, 153),
        );
    });
}

fn kpi_card(ui: &mut Ui, title: &str, value: &str, subtitle: &str, accent_color: Color32) {
    let card_width = 200.0;
    let card_height = 80.0;

    let (rect, _response) = ui.allocate_exact_size(Vec2::new(card_width, card_height), Sense::hover());

    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(8), Color32::from_rgb(30, 41, 59));
    painter.rect_stroke(rect, CornerRadius::same(8), (1.0f32, Color32::from_rgb(51, 65, 85)), StrokeKind::Outside);

    // Accent line at top
    let accent_rect = egui::Rect::from_min_size(rect.min, Vec2::new(card_width, 4.0));
    painter.rect_filled(accent_rect, CornerRadius::ZERO, accent_color);

    ui.allocate_new_ui(
        UiBuilder::new()
            .max_rect(rect.shrink2(Vec2::new(12.0, 10.0)))
            .layout(Layout::top_down(Align::Min)),
        |child_ui| {
            child_ui.label(RichText::new(title).size(12.0).color(Color32::from_rgb(148, 163, 184)));
            child_ui.label(RichText::new(value).size(20.0).strong().color(Color32::WHITE));
            child_ui.label(RichText::new(subtitle).size(11.0).color(Color32::from_rgb(203, 213, 225)));
        },
    );
}
