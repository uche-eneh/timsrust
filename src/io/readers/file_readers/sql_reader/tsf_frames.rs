use super::{ParseDefault, ReadableSqlTable};

// Minimal frame data extracted from the TSF `Frames` table.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SqlTsfFrame {
    pub id: usize,
    pub time: f64,
    pub offset: usize,
    pub num_peaks: usize,
}

impl ReadableSqlTable for SqlTsfFrame {
    fn get_sql_query() -> String {
        "SELECT Id, Time, TimsId, NumPeaks FROM Frames".to_string()
    }

    fn from_sql_row(row: &rusqlite::Row) -> Self {
        Self {
            id: row.parse_default(0),
            time: row.parse_default(1),
            offset: row.parse_default(2),
            num_peaks: row.parse_default(3),
        }
    }
}
