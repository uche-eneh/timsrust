use super::{ParseDefault, ReadableSqlTable};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SqlTsfFrame {
    pub id: usize,
    pub scan_mode: u8,
    pub msms_type: u8,
    pub peak_count: usize,
    pub rt: f64,
    pub binary_offset: usize,
    pub calibration_id: u8,
    pub t1: f64,
    pub t2: f64,
}

impl ReadableSqlTable for SqlTsfFrame {
    fn get_sql_query() -> String {
        "SELECT Id, ScanMode, MsMsType, NumPeaks, Time, TimsId, MzCalibration, T1, T2 FROM Frames".to_string()
    }

    fn from_sql_row(row: &rusqlite::Row) -> Self {
        Self {
            id: row.parse_default(0),
            scan_mode: row.parse_default(1),
            msms_type: row.parse_default(2),
            peak_count: row.parse_default(3),
            rt: row.parse_default(4),
            binary_offset: row.parse_default(5),
            calibration_id: row.parse_default( 6),
            t1: row.parse_default(7),
            t2: row.parse_default( 8),
        }
    }
}
