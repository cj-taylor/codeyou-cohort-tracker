#[derive(Debug, Default)]
pub struct SyncStats {
    pub total_records: i32,
    pub students_inserted: i32,
    pub assignments_inserted: i32,
    pub progressions_inserted: i32,
    pub pages_fetched: i32,
}

impl SyncStats {
    pub fn merge(&mut self, other: SyncStats) {
        self.total_records += other.total_records;
        self.students_inserted += other.students_inserted;
        self.assignments_inserted += other.assignments_inserted;
        self.progressions_inserted += other.progressions_inserted;
        self.pages_fetched += other.pages_fetched;
    }
}
