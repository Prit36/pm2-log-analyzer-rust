//! App shell — port of `src/App.tsx` + `src/components/Pm2AppView.tsx`.

use dioxus::prelude::*;

use crate::core::models::{AggregatedEndpoint, CronAggregated};
use crate::store::{
    install_window_size, restore_persisted, use_analysis_store, AppMode, AppModeStore, AnalysisStore,
    ChartLayout,
};
use crate::ui::api_table::ApiTable;
use crate::ui::charts::LatencyChart;
use crate::ui::cron_table::CronTable;
use crate::ui::filters::FilterBar;
use crate::ui::header::AppHeader;
use crate::ui::ingest::IngestPanel;
use crate::ui::kpi::KpiRow;
use crate::ui::skipped::SkippedDisclosure;
use crate::ui::toast::Toast;
use crate::utils::table_ops::{filter_api_endpoints, sort_api_endpoints, sort_cron_jobs};

const STYLE_CSS: &str = include_str!("../assets/style.css");

const SEARCH_SHORTCUT_SCRIPT: &str = r#"
window.addEventListener('keydown', function (e) {
  if (e.key !== '/' || e.metaKey || e.ctrlKey || e.altKey) return;
  const t = e.target;
  if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.isContentEditable)) return;
  const input = document.querySelector('input[data-filter-search]');
  if (input) { e.preventDefault(); input.focus(); }
});
"#;

#[component]
pub fn App() -> Element {
    let analysis = use_context_provider(AnalysisStore::new);
    let app_mode = use_context_provider(AppModeStore::new);

    use_future(move || async move {
        restore_persisted(analysis).await;
    });
    use_future(move || async move {
        crate::store::app_mode_store::restore_persisted_mode(app_mode).await;
        let _ = document::eval(SEARCH_SHORTCUT_SCRIPT);
        install_window_size(analysis).await;
        // Test hook: load a log file at startup (`PM2_ANALYZER_AUTOLOAD=<path>`).
        if let Ok(path) = std::env::var("PM2_ANALYZER_AUTOLOAD") {
            crate::store::handle_log_files_upload(
                analysis,
                vec![crate::core::pm2::LoadedSource::Path(std::path::PathBuf::from(path))],
                false,
            );
        }
    });

    let mode = app_mode.mode();

    rsx! {
        style { "{STYLE_CSS}" }
        document::Link { rel: "preconnect", href: "https://fonts.googleapis.com" }
        document::Link { rel: "preconnect", href: "https://fonts.gstatic.com", crossorigin: "anonymous" }
        document::Link { rel: "stylesheet", href: "https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:ital,wght@0,400;0,500;0,600;1,400&family=IBM+Plex+Sans:ital,wght@0,400;0,500;0,600;0,700;1,400&display=swap" }
        div { class: "min-h-full",
            AppHeader {}
            main { class: "mx-auto flex max-w-7xl flex-col gap-4 px-4 py-4",
                if mode == AppMode::Mongo {
                    crate::ui::mongo::MongoAppView {}
                } else {
                    Pm2AppView {}
                }
            }
            Toast {}
        }
    }
}

#[component]
pub fn Pm2AppView() -> Element {
    let store = use_analysis_store();
    let api_rows = use_filtered_api_rows();
    let cron_rows = use_filtered_cron_rows();
    let has_cron = store.has_cron_events();
    let layout = store.chart_layout();

    rsx! {
        div { class: "flex flex-col gap-4",
            IngestPanel {}
            KpiRow {}
            FilterBar {}
            if layout == ChartLayout::Wide {
                div { class: "flex flex-col gap-4",
                    LatencyChart { rows: api_rows.clone() }
                    ApiTable { rows: api_rows }
                }
            } else {
                div { class: "grid gap-4 lg:grid-cols-5",
                    div { class: "lg:col-span-3",
                        ApiTable { rows: api_rows.clone() }
                    }
                    div { class: "lg:col-span-2",
                        LatencyChart { rows: api_rows }
                    }
                }
            }
            if has_cron {
                CronTable { rows: cron_rows }
            }
            SkippedDisclosure {}
            footer { class: "pb-6 pt-2 text-center text-[11px] text-slate-400",
                "Parses in your browser - logs never leave this machine"
            }
        }
    }
}

fn use_filtered_api_rows() -> Vec<AggregatedEndpoint> {
    let store = use_analysis_store();
    let filters = store.filters();
    let api = store
        .result()
        .map(|r| r.api)
        .unwrap_or_default();
    let filtered = filter_api_endpoints(&api, &filters.methods, &filters.query);
    let sorted = sort_api_endpoints(&filtered, filters.sort_key, filters.sort_dir);
    sorted.into_iter().take(filters.top_n).collect()
}

fn use_filtered_cron_rows() -> Vec<CronAggregated> {
    let store = use_analysis_store();
    let filters = store.filters();
    let cron = store.result().map(|r| r.cron).unwrap_or_default();
    sort_cron_jobs(&cron, filters.cron_sort_key, filters.cron_sort_dir)
}
