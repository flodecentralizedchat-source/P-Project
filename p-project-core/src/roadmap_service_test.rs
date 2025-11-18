use super::roadmap_service::{MilestoneStatus, RoadmapService};
use chrono::{NaiveDate, NaiveDateTime};

#[test]
fn parse_markdown_and_compute_adherence() {
    let md = r#"
### Technical Roadmap
1. **Q1 2026**: Database query optimization
2. **Q2 2026**: API response time improvements

| Quarter | Focus Area | Key Features |
|---------|------------|--------------|
| Q1 2026 | Performance | API optimization, database improvements |
| Q2 2026 | Security | Advanced cryptography, threat detection |
"#;

    let mut roadmap = RoadmapService::parse_from_markdown(md);
    assert!(roadmap.milestones.len() >= 4);

    // Mark first milestone completed on time (before end of Q1 2026)
    let ontime = NaiveDate::from_ymd_opt(2026, 3, 15)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    let first_id = roadmap.milestones[0].id.clone();
    RoadmapService::mark_completed(&mut roadmap, &first_id, Some(ontime));
    assert_eq!(roadmap.milestones[0].status, MilestoneStatus::Completed);

    // Compute adherence as of end of Q2 2026
    let now = NaiveDate::from_ymd_opt(2026, 6, 30)
        .unwrap()
        .and_hms_opt(23, 59, 59)
        .unwrap();
    let report = RoadmapService::adherence_report(&roadmap, now);

    // At least some milestones are due by now; one is completed on time
    assert!(report.due_by_now >= 2);
    assert!(report.completed_on_time >= 1);
    // No panic and totals make sense
    assert!(report.total >= report.due_by_now);
}
