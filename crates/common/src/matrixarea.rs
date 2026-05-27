//! Migrated from forgottenserver/src/matrixarea.h + matrixarea.cpp
//!
//! `MatrixArea` is a 2D boolean matrix used to define combat area shapes.
//! The C++ uses `(centerX, centerY)` where X = col and Y = row; we store
//! the same pair as `(center_x, center_y)` with x=col, y=row.

#![allow(dead_code)]

/// A 2D boolean matrix representing a combat area shape.
///
/// Internally stored in row-major order: `data[row * cols + col]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatrixArea {
    rows: u32,
    cols: u32,
    // (center_x = col index, center_y = row index) — matches C++ `Center{x, y}`
    center: (u32, u32),
    data: Vec<bool>,
}

impl MatrixArea {
    /// Create a new matrix with all cells set to `false`.
    pub fn new(rows: u32, cols: u32) -> Self {
        let size = (rows as usize).saturating_mul(cols as usize);
        Self {
            rows,
            cols,
            center: (0, 0),
            data: vec![false; size],
        }
    }

    /// Build from existing components (used internally by rotation methods).
    fn from_parts(center: (u32, u32), rows: u32, cols: u32, data: Vec<bool>) -> Self {
        Self {
            rows,
            cols,
            center,
            data,
        }
    }

    // ------------------------------------------------------------------
    // Accessors
    // ------------------------------------------------------------------

    pub fn rows(&self) -> u32 {
        self.rows
    }

    pub fn cols(&self) -> u32 {
        self.cols
    }

    /// `(center_x, center_y)` — col-index, row-index (matches C++ `Center{x, y}`).
    pub fn get_center(&self) -> (u32, u32) {
        self.center
    }

    /// Set center by `(row, col)` as in the C++ `setCenter(y, x)` call.
    /// (C++ signature: `setCenter(uint32_t y, uint32_t x)` → stores `{x, y}`.)
    pub fn set_center(&mut self, row: u32, col: u32) {
        // C++ stores {x, y} where x=col, y=row
        self.center = (col, row);
    }

    /// Returns `true` if rows == 0 or cols == 0 (mirrors C++ `operator bool`).
    pub fn is_empty(&self) -> bool {
        self.rows == 0 || self.cols == 0
    }

    // ------------------------------------------------------------------
    // Cell access
    // ------------------------------------------------------------------

    fn idx(&self, row: u32, col: u32) -> usize {
        row as usize * self.cols as usize + col as usize
    }

    pub fn get(&self, row: u32, col: u32) -> bool {
        self.data[self.idx(row, col)]
    }

    pub fn set(&mut self, row: u32, col: u32, value: bool) {
        let i = self.idx(row, col);
        self.data[i] = value;
    }

    // ------------------------------------------------------------------
    // Iteration
    // ------------------------------------------------------------------

    /// Returns an iterator over `(row, col)` pairs for every set (`true`) cell.
    pub fn iter_set(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        (0..self.rows).flat_map(move |r| {
            (0..self.cols).filter_map(move |c| if self.get(r, c) { Some((r, c)) } else { None })
        })
    }

    // ------------------------------------------------------------------
    // Rotation
    // ------------------------------------------------------------------

    /// Rotate 90° clockwise.
    ///
    /// C++ algorithm: for each original row `i`, the row (top→bottom)
    /// becomes the new col `i` (right→left in new matrix).
    /// New dimensions: rows ↔ cols swapped.
    ///
    /// Center transform: `{rows - centerY - 1, centerX}`
    pub fn rotate_90(&self) -> Self {
        // new dims: new_rows = cols, new_cols = rows
        let new_rows = self.cols;
        let new_cols = self.rows;
        let mut new_data = vec![false; self.data.len()];

        // C++ slice logic:
        //   newArr[slice(i, cols, rows)] = arr[slice((rows-i-1)*cols, cols, 1)]
        // Translating:
        //   dest positions: i, i+rows, i+2*rows, ... (step=new_cols=rows) for cols items
        //   src positions:  (rows-i-1)*cols + 0,1,...cols-1
        for i in 0..self.rows as usize {
            let src_row = self.rows as usize - i - 1;
            for j in 0..self.cols as usize {
                // dest[i + j*new_cols] where new_cols = rows
                let dest = i + j * self.rows as usize;
                let src = src_row * self.cols as usize + j;
                new_data[dest] = self.data[src];
            }
        }

        let (center_x, center_y) = self.center;
        // C++: {rows - centerY - 1, centerX}  where first elem is x (col), second is y (row)
        let new_center = (self.rows - center_y - 1, center_x);
        Self::from_parts(new_center, new_rows, new_cols, new_data)
    }

    /// Rotate 180°.
    ///
    /// C++: reverse the flat array; center = `{cols - centerX - 1, rows - centerY - 1}`.
    pub fn rotate_180(&self) -> Self {
        let mut new_data = self.data.clone();
        new_data.reverse();

        let (center_x, center_y) = self.center;
        let new_center = (self.cols - center_x - 1, self.rows - center_y - 1);
        Self::from_parts(new_center, self.rows, self.cols, new_data)
    }

    /// Rotate 270° clockwise (= 90° counter-clockwise).
    ///
    /// C++ algorithm:
    ///   `newArr[slice(i*rows, rows, 1)] = arr[slice(cols-i-1, rows, cols)]`
    /// New dimensions: rows ↔ cols swapped.
    ///
    /// Center transform: `{centerY, cols - centerX - 1}`
    pub fn rotate_270(&self) -> Self {
        let new_rows = self.cols;
        let new_cols = self.rows;
        let mut new_data = vec![false; self.data.len()];

        for i in 0..self.cols as usize {
            let src_col = self.cols as usize - i - 1;
            for j in 0..self.rows as usize {
                // dest: i*new_cols + j  where new_cols = rows
                let dest = i * self.rows as usize + j;
                // src: slice(cols-i-1, rows, cols) → src_col + j*cols
                let src = src_col + j * self.cols as usize;
                new_data[dest] = self.data[src];
            }
        }

        let (center_x, center_y) = self.center;
        // C++: {centerY, cols - centerX - 1}
        let new_center = (center_y, self.cols - center_x - 1);
        Self::from_parts(new_center, new_rows, new_cols, new_data)
    }
}

// ------------------------------------------------------------------
// createArea helper (mirrors C++ free function)
// ------------------------------------------------------------------

/// Build a `MatrixArea` from a flat vector of u32 values:
/// - `1` or `3` → cell is `true`
/// - `2` or `3` → this cell is the center
pub fn create_area(vec: &[u32], rows: u32) -> MatrixArea {
    let cols = if rows == 0 {
        0
    } else {
        vec.len() as u32 / rows
    };
    let mut area = MatrixArea::new(rows, cols);

    let mut x: u32 = 0;
    let mut y: u32 = 0;

    for &value in vec {
        if value == 1 || value == 3 {
            area.set(y, x, true);
        }
        if value == 2 || value == 3 {
            area.set_center(y, x);
        }

        x += 1;
        if x == cols {
            x = 0;
            y += 1;
        }
    }
    area
}

// ------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ----------------------------------------------------------------
    // Basic construction
    // ----------------------------------------------------------------

    #[test]
    fn new_all_false() {
        let m = MatrixArea::new(3, 3);
        for r in 0..3 {
            for c in 0..3 {
                assert!(!m.get(r, c), "expected false at ({r},{c})");
            }
        }
    }

    #[test]
    fn set_and_get() {
        let mut m = MatrixArea::new(3, 3);
        m.set(1, 1, true);
        assert!(m.get(1, 1));
        assert!(!m.get(0, 0));
    }

    #[test]
    fn dimensions() {
        let m = MatrixArea::new(5, 7);
        assert_eq!(m.rows(), 5);
        assert_eq!(m.cols(), 7);
    }

    #[test]
    fn is_empty_zero_rows() {
        let m = MatrixArea::new(0, 3);
        assert!(m.is_empty());
    }

    #[test]
    fn is_empty_zero_cols() {
        let m = MatrixArea::new(3, 0);
        assert!(m.is_empty());
    }

    #[test]
    fn not_empty() {
        let m = MatrixArea::new(3, 3);
        assert!(!m.is_empty());
    }

    // ----------------------------------------------------------------
    // Center
    // ----------------------------------------------------------------

    #[test]
    fn default_center_is_zero() {
        let m = MatrixArea::new(3, 3);
        assert_eq!(m.get_center(), (0, 0));
    }

    #[test]
    fn set_center_3x3() {
        let mut m = MatrixArea::new(3, 3);
        // set_center(row=1, col=1) → stores (col=1, row=1)
        m.set_center(1, 1);
        assert_eq!(m.get_center(), (1, 1));
    }

    // ----------------------------------------------------------------
    // iter_set
    // ----------------------------------------------------------------

    #[test]
    fn iter_set_empty() {
        let m = MatrixArea::new(3, 3);
        assert_eq!(m.iter_set().count(), 0);
    }

    #[test]
    fn iter_set_single() {
        let mut m = MatrixArea::new(3, 3);
        m.set(2, 1, true);
        let cells: Vec<_> = m.iter_set().collect();
        assert_eq!(cells, vec![(2, 1)]);
    }

    // ----------------------------------------------------------------
    // create_area — matches C++ test_createArea
    // ----------------------------------------------------------------

    #[test]
    fn create_area_basic() {
        #[rustfmt::skip]
        let m = create_area(&[
            0, 0, 1, 1,
            3, 1, 1, 1,
            0, 0, 1, 1,
        ], 3);

        let (cx, cy) = m.get_center();
        // C++ test: centerX==0, centerY==1 → center stored as (x=col=0, y=row=1)
        assert_eq!(cx, 0, "center_x (col)");
        assert_eq!(cy, 1, "center_y (row)");

        assert_eq!(m.cols(), 4);
        assert_eq!(m.rows(), 3);

        assert!(!m.get(0, 0));
        assert!(!m.get(0, 1));
        assert!(m.get(0, 2));
        assert!(m.get(0, 3));
        assert!(m.get(1, 0));
        assert!(m.get(1, 1));
        assert!(m.get(1, 2));
        assert!(m.get(1, 3));
        assert!(!m.get(2, 0));
        assert!(!m.get(2, 1));
        assert!(m.get(2, 2));
        assert!(m.get(2, 3));
    }

    // ----------------------------------------------------------------
    // rotate_90 — matches C++ test_MatrixArea_rotate90
    // ----------------------------------------------------------------

    #[test]
    fn rotate_90_matches_cpp_test() {
        #[rustfmt::skip]
        let m = create_area(&[
            3, 1, 1, 1,
            0, 1, 1, 1,
            0, 0, 1, 0,
        ], 3).rotate_90();

        // expected:
        // 0, 0, 3,
        // 0, 1, 1,
        // 1, 1, 1,
        // 0, 1, 1,
        let (cx, cy) = m.get_center();
        assert_eq!(cx, 2, "center_x after rotate_90");
        assert_eq!(cy, 0, "center_y after rotate_90");

        assert_eq!(m.cols(), 3);
        assert_eq!(m.rows(), 4);

        assert!(!m.get(0, 0));
        assert!(!m.get(0, 1));
        assert!(m.get(0, 2));
        assert!(!m.get(1, 0));
        assert!(m.get(1, 1));
        assert!(m.get(1, 2));
        assert!(m.get(2, 0));
        assert!(m.get(2, 1));
        assert!(m.get(2, 2));
        assert!(!m.get(3, 0));
        assert!(m.get(3, 1));
        assert!(m.get(3, 2));
    }

    // ----------------------------------------------------------------
    // rotate_180 — matches C++ test_MatrixArea_rotate180
    // ----------------------------------------------------------------

    #[test]
    fn rotate_180_matches_cpp_test() {
        #[rustfmt::skip]
        let m = create_area(&[
            3, 1, 1, 1,
            0, 1, 1, 1,
            0, 0, 1, 0,
        ], 3).rotate_180();

        // expected:
        // 0, 1, 0, 0,
        // 1, 1, 1, 0,
        // 1, 1, 1, 3,
        let (cx, cy) = m.get_center();
        assert_eq!(cx, 3, "center_x after rotate_180");
        assert_eq!(cy, 2, "center_y after rotate_180");

        assert_eq!(m.cols(), 4);
        assert_eq!(m.rows(), 3);

        assert!(!m.get(0, 0));
        assert!(m.get(0, 1));
        assert!(!m.get(0, 2));
        assert!(!m.get(0, 3));
        assert!(m.get(1, 0));
        assert!(m.get(1, 1));
        assert!(m.get(1, 2));
        assert!(!m.get(1, 3));
        assert!(m.get(2, 0));
        assert!(m.get(2, 1));
        assert!(m.get(2, 2));
        assert!(m.get(2, 3)); // the center cell (value=3) → true
    }

    // ----------------------------------------------------------------
    // rotate_270 — matches C++ test_MatrixArea_rotate270
    // ----------------------------------------------------------------

    #[test]
    fn rotate_270_matches_cpp_test() {
        #[rustfmt::skip]
        let m = create_area(&[
            3, 1, 1, 1,
            0, 1, 1, 1,
            0, 0, 1, 0,
        ], 3).rotate_270();

        // expected:
        // 1, 1, 0,
        // 1, 1, 1,
        // 1, 1, 0,
        // 3, 0, 0,
        let (cx, cy) = m.get_center();
        assert_eq!(cx, 0, "center_x after rotate_270");
        assert_eq!(cy, 3, "center_y after rotate_270");

        assert_eq!(m.cols(), 3);
        assert_eq!(m.rows(), 4);

        assert!(m.get(0, 0));
        assert!(m.get(0, 1));
        assert!(!m.get(0, 2));
        assert!(m.get(1, 0));
        assert!(m.get(1, 1));
        assert!(m.get(1, 2));
        assert!(m.get(2, 0));
        assert!(m.get(2, 1));
        assert!(!m.get(2, 2));
        assert!(m.get(3, 0));
        assert!(!m.get(3, 1));
        assert!(!m.get(3, 2));
    }

    // ----------------------------------------------------------------
    // rotate_180 applied twice = identity
    // ----------------------------------------------------------------

    #[test]
    fn rotate_180_twice_is_identity() {
        #[rustfmt::skip]
        let original = create_area(&[
            3, 1, 1, 1,
            0, 1, 1, 1,
            0, 0, 1, 0,
        ], 3);
        let restored = original.rotate_180().rotate_180();
        assert_eq!(original, restored);
    }

    // ----------------------------------------------------------------
    // rotate_90 four times = identity
    // ----------------------------------------------------------------

    #[test]
    fn rotate_90_four_times_is_identity() {
        #[rustfmt::skip]
        let original = create_area(&[
            3, 1, 0,
            0, 1, 1,
            0, 0, 1,
        ], 3);
        let r = original.rotate_90().rotate_90().rotate_90().rotate_90();
        assert_eq!(original, r);
    }

    // ----------------------------------------------------------------
    // All four rotations on an explicit 3×3 pattern
    //
    // Pattern (center at (row=1, col=1)):
    //   1 0 0
    //   3 1 0    ← center cell at (1,0) → stored as (col=0, row=1)
    //   1 0 0
    // ----------------------------------------------------------------

    fn make_3x3_pattern() -> MatrixArea {
        create_area(&[1, 0, 0, 3, 1, 0, 1, 0, 0], 3)
    }

    #[test]
    fn rotate_90_3x3_dimensions_and_center() {
        let m = make_3x3_pattern();
        // Original 3×3; center stored as (col=0, row=1)
        assert_eq!(m.rows(), 3);
        assert_eq!(m.cols(), 3);
        assert_eq!(m.get_center(), (0, 1));

        let r = m.rotate_90();
        // Square stays 3×3 after rotation
        assert_eq!(r.rows(), 3);
        assert_eq!(r.cols(), 3);
        // Center transform for rotate_90: new_x = rows - center_y - 1 = 3-1-1 = 1
        //                                 new_y = center_x = 0
        assert_eq!(r.get_center(), (1, 0));

        // Trace the rotate_90 algorithm:
        //   old (src_row=rows-i-1, j) → new (row=j, col=i)
        // Original:
        //   (0,0)=T (0,1)=F (0,2)=F
        //   (1,0)=T (1,1)=T (1,2)=F
        //   (2,0)=T (2,1)=F (2,2)=F
        //
        // New matrix (after 90° CW):
        //   (0,0)=old(2,0)=T  (0,1)=old(1,0)=T  (0,2)=old(0,0)=T
        //   (1,0)=old(2,1)=F  (1,1)=old(1,1)=T  (1,2)=old(0,1)=F
        //   (2,0)=old(2,2)=F  (2,1)=old(1,2)=F  (2,2)=old(0,2)=F
        assert!(r.get(0, 0));
        assert!(r.get(0, 1));
        assert!(r.get(0, 2));
        assert!(!r.get(1, 0));
        assert!(r.get(1, 1));
        assert!(!r.get(1, 2));
        assert!(!r.get(2, 0));
        assert!(!r.get(2, 1));
        assert!(!r.get(2, 2));
    }

    #[test]
    fn rotate_180_3x3_center_and_cells() {
        let m = make_3x3_pattern();
        let r = m.rotate_180();
        // Dims unchanged
        assert_eq!(r.rows(), 3);
        assert_eq!(r.cols(), 3);
        // Center transform: new_x = cols-cx-1 = 3-0-1 = 2; new_y = rows-cy-1 = 3-1-1 = 1
        assert_eq!(r.get_center(), (2, 1));
        // After 180°, flat array is reversed.
        // Original flat: [T,F,F, T,T,F, T,F,F]
        // Reversed:       [F,F,T, F,T,T, F,F,T]
        // So row 0: F,F,T; row 1: F,T,T; row 2: F,F,T
        assert!(!r.get(0, 0));
        assert!(!r.get(0, 1));
        assert!(r.get(0, 2));
        assert!(!r.get(1, 0));
        assert!(r.get(1, 1));
        assert!(r.get(1, 2));
        assert!(!r.get(2, 0));
        assert!(!r.get(2, 1));
        assert!(r.get(2, 2));
    }

    #[test]
    fn rotate_270_3x3_center_and_cells() {
        let m = make_3x3_pattern();
        let r = m.rotate_270();
        // Dims unchanged (square)
        assert_eq!(r.rows(), 3);
        assert_eq!(r.cols(), 3);
        // Center transform: new_x = cy = 1; new_y = cols-cx-1 = 3-0-1 = 2
        assert_eq!(r.get_center(), (1, 2));
    }

    #[test]
    fn all_four_rotations_3x3_roundtrip() {
        let m = make_3x3_pattern();
        let r = m.rotate_90().rotate_90().rotate_90().rotate_90();
        assert_eq!(m, r, "4x rotate_90 on 3x3 must be identity");
    }

    // ----------------------------------------------------------------
    // iter_set yields only set cells — multiple cells
    // ----------------------------------------------------------------

    #[test]
    fn iter_set_multiple_cells() {
        let mut m = MatrixArea::new(3, 3);
        m.set(0, 0, true);
        m.set(1, 2, true);
        m.set(2, 1, true);
        let cells: Vec<_> = m.iter_set().collect();
        // iter_set must enumerate in row-major order
        assert_eq!(cells, vec![(0, 0), (1, 2), (2, 1)]);
    }

    #[test]
    fn iter_set_all_set() {
        let mut m = MatrixArea::new(2, 2);
        m.set(0, 0, true);
        m.set(0, 1, true);
        m.set(1, 0, true);
        m.set(1, 1, true);
        assert_eq!(m.iter_set().count(), 4);
    }

    // ----------------------------------------------------------------
    // out-of-bounds set panics (Vec bounds check)
    //
    // The `idx` function computes `row * cols + col` without bounds
    // checks; a truly out-of-range index (e.g. row >= rows AND col >= cols)
    // will exceed the Vec length and cause a panic.
    // ----------------------------------------------------------------

    #[test]
    #[should_panic]
    fn oob_set_panics() {
        let mut m = MatrixArea::new(3, 3);
        // row=10 × 3 + col=10 = 40 >> 9 (vec len)
        m.set(10, 10, true);
    }

    #[test]
    #[should_panic]
    fn oob_get_panics() {
        let m = MatrixArea::new(3, 3);
        // row=5 × 3 + col=5 = 20 >> 9 (vec len)
        let _ = m.get(5, 5);
    }

    // ----------------------------------------------------------------
    // centre position: set_center round-trips through get_center
    // ----------------------------------------------------------------

    #[test]
    fn center_position_round_trip() {
        let mut m = MatrixArea::new(5, 7);
        m.set_center(3, 6); // row=3, col=6
                            // get_center returns (col, row) = (6, 3)
        assert_eq!(m.get_center(), (6, 3));
    }

    // ----------------------------------------------------------------
    // create_area — value=2 marks ONLY the center (cell stays false).
    //
    // C++ semantics:
    //   value == 1 → cell true
    //   value == 2 → center marker only (cell remains false)
    //   value == 3 → cell true AND center marker
    //   value == 0 → nothing
    //
    // This case (value=2, center-only without setting the cell) is a
    // distinct branch through `create_area` that the original tests
    // never exercise.
    // ----------------------------------------------------------------

    #[test]
    fn create_area_value_2_marks_center_only() {
        #[rustfmt::skip]
        let m = create_area(&[
            0, 0, 0,
            0, 2, 0,
            0, 0, 0,
        ], 3);

        // Center stored as (x=col=1, y=row=1)
        assert_eq!(m.get_center(), (1, 1), "value=2 must set center");

        // Critically: the center cell itself must NOT be set true.
        assert!(!m.get(1, 1), "value=2 must NOT set the cell true");

        // Every other cell is false too.
        for r in 0..3 {
            for c in 0..3 {
                assert!(!m.get(r, c), "cell ({r},{c}) must remain false");
            }
        }
    }

    // ----------------------------------------------------------------
    // create_area — rows == 0 produces an empty matrix.
    //
    // C++ code path:
    //   if (rows == 0) { cols = 0; }
    //
    // The resulting MatrixArea has rows=0, cols=0 and is_empty().
    // ----------------------------------------------------------------

    #[test]
    fn create_area_rows_zero_produces_empty() {
        let m = create_area(&[], 0);
        assert_eq!(m.rows(), 0);
        assert_eq!(m.cols(), 0);
        assert!(m.is_empty());
        assert_eq!(m.iter_set().count(), 0);
    }
}
