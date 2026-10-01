# Egui UI parity plan

The React/Wasm app in `../pm2-log-analyzer` is the product and behavior
reference. This native app uses `eframe`/`egui`; [FastFrame](https://github.com/crmne/fastframe)
supplies shared desktop foundations, while page structure and product styling
stay in this repository.

## Current state

- The app entry point runs `EguiApp` through `eframe`.
- FastFrame text rendering, font fallbacks, Lucide icons, palette mapping, and
  theme transitions are wired into the egui app.
- PM2 and MongoDB views exist, including their tables, charts, filters, and
  query inspection flows. The previous iced modules remain compiled for now.
- Mongo view orchestration now delegates to separate KPI, controls, patterns,
  slow-query, users, charts, and diagnostics modules.
- The first responsive parity pass fixes the Mongo filter/card/table clipping,
  keeps the PM2 split layout above the reference breakpoint, and stacks the
  PM2 table and chart below it. Compact PM2 filters place Reset on its own row;
  stacked PM2 tables have distinct egui IDs.
- Mongo patterns, slow queries, user activity, charts, and diagnostics now use
  viewport-aware widths. Narrow tables preserve all columns, shortening only
  secondary headings where needed.
- Native captures were reviewed in light mode at 1280x850 for PM2 and all Mongo
  tabs; 1024x768 and 900x600 for PM2; and 900px for every Mongo tab, chart
  mode, and diagnostics subtab. User detail was also captured at 900px. One
  dark 900px Mongo pattern capture was reviewed; the full dark matrix remains.
- `rustfmt` on changed modules and `cargo check --bin pm2-log-analyzer` pass
  without warnings. No kernel code changed; data parity was not rerun.

## Work plan

### 1. Establish shared egui foundations

- Keep IBM Plex as the product face, with FastFrame desktop rendering settings
  and script fallbacks.
- Use FastFrame's icon loader and theme transition support consistently.
- Keep Tailwind-derived product colors and component decisions in
  `egui_app/theme.rs` and `egui_app/widgets.rs`.
- Consolidate repeated card, field, chip, button, table-header, and status
  treatments into app-local widgets.

### 2. Match the PM2 view

- Preserve the reference order: ingest, KPIs, filters, API table/chart,
  conditional cron table, skipped-line disclosure, footer.
- Match the 1280px maximum content width, 16px page padding and section gaps.
- Follow the reference 1024px split breakpoint; stack the split view below it.
- Keep filtering, sorting, copy, paste, file/folder ingest, progress, cancel,
  export, theme, and empty states reachable at the minimum window size.

### 3. Match MongoDB views

- Match the reference KPI grid and five-tab control bar.
- Keep search, plan, operation, collection, user, duration, scan-ratio, and
  reset controls visible through wrapping at narrow widths.
- Make patterns, slow-query, user-activity, chart, and diagnostics views fill
  their cards and remain navigable at narrow widths.
- Preserve the query inspection modal, per-user detail actions, chart modes,
  sorting, and copy actions.
- **First pass complete at 900px and 1280px; 1024px Mongo users checked.** Run
  the remaining Mongo tabs at 1024px and verify all control interactions.

### 4. Simplify the app code

- Keep `EguiApp` focused on state, background events, and action routing.
- Keep PM2 sections as small view modules and split the oversized Mongo view
  into tab and shared-widget modules. **Mongo view split completed; PM2/EguiApp
  responsibilities still need review.**
- Remove the unused iced path and dependencies after egui coverage and behavior
  checks replace the old headless tests.

### 5. Verify parity

- Capture the same synthetic PM2 and Mongo samples in the browser reference and
  native app at 900x600, 1024x768, and 1280x850, in light and dark themes. The
  light matrix is broad but still needs exact reference comparisons at every
  size; the complete dark matrix remains open.
- Compare section order, spacing, typography, wrapping, table widths, chart
  bounds, and interaction results. Track renderer-only differences separately.
- Keep `parity_runner --dump-wires` and `parity/compare.mjs` as the data-parity
  gate; UI edits must not change kernel output.

## Acceptance criteria

- No labels, controls, KPI cards, or table columns are cut off at 900px, 1024px,
  or 1280px window widths.
- PM2 and MongoDB section order, labels, default states, and actions match the
  React/Wasm reference.
- Light and dark palettes use the same semantic colors, and native spacing
  follows the reference at matched viewport sizes.
- Changed modules are rustfmt-clean and `cargo check --bin pm2-log-analyzer`
  passes without warnings.
- Data parity remains unchanged.
