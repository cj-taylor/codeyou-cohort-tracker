use crate::db::Database;
use crate::lms::LmsProvider;
use crate::sync::types::SyncStats;
use anyhow::Result;
use std::time::Duration;

pub struct SyncEngine {
    provider: Box<dyn LmsProvider>,
}

impl SyncEngine {
    pub fn new(provider: Box<dyn LmsProvider>) -> Self {
        Self { provider }
    }

    pub async fn sync_all(&mut self, db: &Database, full: bool) -> Result<SyncStats> {
        let active_classes = db.get_active_classes()?;

        if active_classes.is_empty() {
            println!("No active classes found. Run 'init' or 'activate' first.");
            return Ok(SyncStats::default());
        }

        println!("Found {} active class(es) to sync\n", active_classes.len());

        let mut total_stats = SyncStats::default();

        for (i, class) in active_classes.iter().enumerate() {
            println!(
                "=== [{}/{}] Syncing: {} ===",
                i + 1,
                active_classes.len(),
                class.name
            );
            let class_stats = self.sync_class(&class.id, db, full).await?;
            total_stats.merge(class_stats);

            let now = chrono::Utc::now().to_rfc3339();
            db.update_class_sync_time(&class.id, &now)?;
        }

        println!("\n=== All Classes Synced ===");
        println!("Total pages fetched: {}", total_stats.pages_fetched);
        println!("Total progressions: {}", total_stats.progressions_inserted);

        Ok(total_stats)
    }

    pub async fn sync_class(
        &mut self,
        class_id: &str,
        db: &Database,
        full: bool,
    ) -> Result<SyncStats> {
        let mut stats = SyncStats::default();

        println!("Fetching class structure...");
        let assignment_sections = match self.provider.fetch_class_structure(class_id).await {
            Ok(sections) => {
                println!("Found {} assignments with section info", sections.len());
                sections
            }
            Err(e) => {
                println!("Warning: Could not fetch class structure: {}. Continuing without section info.", e);
                std::collections::HashMap::new()
            }
        };

        let mut existing_progressions = db.get_progression_ids_by_class(class_id)?;
        println!(
            "Found {} existing progressions in database",
            existing_progressions.len()
        );

        let mut page = 0;
        let mut consecutive_all_duplicate_pages = 0;
        const MAX_DUPLICATE_PAGES: i32 = 1;

        loop {
            print!("Fetching page {}...", page);
            std::io::Write::flush(&mut std::io::stdout()).ok();

            let batch = self.provider.fetch_progressions(class_id, page).await?;
            let records_count = batch.progressions.len();

            if records_count == 0 {
                println!("No more records to fetch.");
                break;
            }

            let mut new_records = 0;
            let mut duplicate_records = 0;

            for progression in batch.progressions {
                if existing_progressions.contains(&progression.id) {
                    duplicate_records += 1;
                    continue;
                }

                new_records += 1;

                db.insert_student(
                    &progression.student.id,
                    class_id,
                    &progression.student.first_name,
                    &progression.student.last_name,
                    &progression.student.email,
                )?;
                stats.students_inserted += 1;

                let section = assignment_sections
                    .get(&progression.assignment.id)
                    .map(|s| s.as_str());
                db.insert_assignment(
                    &progression.assignment.id,
                    class_id,
                    &progression.assignment.name,
                    &progression.assignment.assignment_type,
                    section,
                )?;
                stats.assignments_inserted += 1;

                db.insert_progression(
                    &progression.id,
                    class_id,
                    &progression.student.id,
                    &progression.assignment.id,
                    progression.grade,
                    &progression.started_at,
                    &progression.completed_at,
                    progression.reviewed_at.as_deref(),
                )?;
                stats.progressions_inserted += 1;

                stats.total_records += 1;
                existing_progressions.insert(progression.id);
            }

            db.record_sync(class_id, page, records_count as i32)?;

            stats.pages_fetched += 1;
            page += 1;

            println!(
                "  Page {}: {} new, {} duplicates",
                page - 1,
                new_records,
                duplicate_records
            );

            if new_records == 0 && duplicate_records > 0 {
                consecutive_all_duplicate_pages += 1;
            } else {
                consecutive_all_duplicate_pages = 0;
            }

            if !full && consecutive_all_duplicate_pages >= MAX_DUPLICATE_PAGES {
                println!(
                    "  Stopping: {} consecutive pages of all duplicates (incremental sync)",
                    MAX_DUPLICATE_PAGES
                );
                break;
            }

            if !batch.can_load_more {
                break;
            }

            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        println!("\n✓ Class sync complete:");
        println!("  Pages fetched: {}", stats.pages_fetched);
        println!("  Total records: {}", stats.total_records);
        println!(
            "  Students: {} (unique)",
            db.get_student_count_by_class(class_id)?
        );
        println!(
            "  Assignments: {} (unique)",
            db.get_assignment_count_by_class(class_id)?
        );
        println!("  Progressions: {}", stats.progressions_inserted);

        Ok(stats)
    }
}
