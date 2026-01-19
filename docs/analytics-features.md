# Analytics Features

This document describes the analytics and diagnostic features available in the Cohort Tracker dashboard.

## Implemented Features

### Quick Wins

1. **Assignment Type Breakdown**
   - Shows completion rates and average grades by assignment type (lesson, quiz)
   - Helps identify if students struggle more with certain types of work
   - Endpoint: `/classes/:class_id/metrics/assignment-types`

2. **Grade Distribution**
   - Histogram visualization of grade spread (6 buckets: 0-50%, 50-60%, 60-70%, 70-80%, 80-90%, 90-100%)
   - Identifies if grading is too harsh/lenient or if there are struggling subgroups
   - Endpoint: `/classes/:class_id/metrics/grade-distribution`

3. **Engagement Gap Detection**
   - Identifies students who are on track (>50% complete) but inactive for 7-14 days
   - Early warning system before students become at-risk
   - Prominent alert widget at top of dashboard
   - Endpoint: `/classes/:class_id/metrics/engagement-gaps`

4. **Assignment Difficulty Ranking**
   - Composite difficulty score: (1 - completion_rate) × 0.6 + (1 - avg_grade) × 0.4
   - Color-coded: red (high), yellow (medium), green (low)
   - Replaces simple "Top Blockers" with more sophisticated analysis
   - Endpoint: `/classes/:class_id/metrics/assignment-difficulty`

### High Impact

5. **Velocity/Pace Tracking**
   - Line chart showing average assignments completed per student per week
   - Identifies if class is speeding up, slowing down, or maintaining pace
   - Critical for early identification of students falling behind
   - Endpoint: `/classes/:class_id/metrics/velocity`

## Future Enhancements

The following features are planned for future development:

6. **Predictive Risk Scoring**
   - Track completion rate trends week-over-week
   - Flag students whose pace is declining
   - Proactive intervention based on trajectory

7. **Cohort Comparison**
   - Side-by-side comparison of different classes (Module 1 vs Module 2)
   - Identify if certain cohorts need different support strategies

8. **Mentor Effectiveness Comparison**
   - Statistical comparison of outcomes by mentor/night
   - Identify best practices from high-performing groups

## UI Improvements

- **Sticky Header** ✅: Night filter and demo mode controls always accessible at top of page
- **Color Coding**: Visual indicators for difficulty levels and risk scores
- **Interactive Charts**: Click-through from charts to detailed views
- **Demo Mode**: Masks student names and emails for screenshots/presentations

## All Features Support

- Night filtering (compare cohorts)
- Demo mode (privacy protection)
- Sortable tables
- Click-through to student/assignment details
- Real-time updates

## API Endpoints

All endpoints support optional `?night=<night_name>` query parameter for filtering.

- `GET /classes/:class_id/metrics/assignment-types` - Assignment type performance
- `GET /classes/:class_id/metrics/grade-distribution` - Grade histogram
- `GET /classes/:class_id/metrics/velocity` - Weekly pace tracking
- `GET /classes/:class_id/metrics/engagement-gaps` - Silent students
- `GET /classes/:class_id/metrics/assignment-difficulty` - Difficulty ranking
- `GET /classes/:class_id/metrics/student-health` - At-risk students
- `GET /classes/:class_id/metrics/student-activity` - Activity monitoring
- `GET /classes/:class_id/metrics/progress-over-time` - Weekly trends
- `GET /classes/:class_id/metrics/section-progress` - Section completion
- `GET /classes/:class_id/metrics/night-summary` - Performance by night
- `GET /classes/:class_id/metrics/day-of-week` - Activity by day
- `GET /classes/:class_id/metrics/time-of-day` - Activity by time
- `GET /classes/:class_id/metrics/blockers` - Low completion assignments (legacy)

## Testing

Tests are located in `tests/db_tests.rs` and cover:
- Assignment type aggregation
- Grade distribution bucketing
- Velocity calculations
- Engagement gap detection
- Difficulty scoring

Run tests: `cargo test --test db_tests`
