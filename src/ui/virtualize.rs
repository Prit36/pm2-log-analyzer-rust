//! Fixed-row-height virtualization shared by the scrollable tables.
//!
//! iced lays out every child of a scrollable, so the tables render only the
//! rows intersecting the viewport plus an overscan window, padded top/bottom by
//! spacers with the height of the hidden rows. Rendering is O(visible rows)
//! instead of O(total rows) — 6k+ endpoints stay at a handful of widgets.

/// Returns the half-open row range `[start, end)` to render.
pub fn visible_range(
    total: usize,
    offset: f32,
    viewport_height: f32,
    row_height: f32,
    overscan: usize,
) -> (usize, usize) {
    if total == 0 || viewport_height <= 0.0 || row_height <= 0.0 {
        return (0, 0);
    }
    let offset = offset.max(0.0);
    let first = (offset / row_height).floor() as usize;
    let visible = (viewport_height / row_height).ceil() as usize + 1;
    let start = first.saturating_sub(overscan).min(total);
    let end = (first + visible + overscan).min(total);
    (start, end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_the_visible_rows_only() {
        // 10 000 rows, scrolled to row 5 000, 320px viewport, 32px rows.
        let (start, end) = visible_range(10_000, 160_000.0, 320.0, 32.0, 6);
        assert_eq!(start, 5_000 - 6);
        assert_eq!(end, 5_000 + 11 + 6);

        // Overscan never walks past either edge.
        assert_eq!(visible_range(100, 0.0, 320.0, 32.0, 6), (0, 17));
        assert_eq!(visible_range(10, 9_000.0, 320.0, 32.0, 6), (10, 10));
        assert_eq!(visible_range(0, 0.0, 320.0, 32.0, 6), (0, 0));
        assert_eq!(visible_range(100, 0.0, 0.0, 32.0, 6), (0, 0));
    }
}
