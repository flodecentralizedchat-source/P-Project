use chrono::{Datelike, NaiveDate, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MilestoneStatus {
    Planned,
    InProgress,
    Completed,
    Delayed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: String,
    pub name: String,
    pub due: NaiveDateTime,
    pub status: MilestoneStatus,
    pub completed_at: Option<NaiveDateTime>,
    pub phase: Option<String>,
    pub focus_area: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Roadmap {
    pub milestones: Vec<Milestone>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdherenceReport {
    pub generated_at: NaiveDateTime,
    pub total: usize,
    pub due_by_now: usize,
    pub completed_on_time: usize,
    pub completed_late: usize,
    pub overdue: usize,
    pub on_track: bool,
}

pub struct RoadmapService;

impl RoadmapService {
    pub fn parse_from_markdown(md: &str) -> Roadmap {
        let mut milestones = Vec::new();

        // Parse lines like: "1. **Q1 2026**: Database query optimization"
        for line in md.lines() {
            let line = line.trim();
            if let Some((quarter, rest)) = Self::parse_quarter_line(line) {
                let due = Self::end_of_quarter(quarter.0, quarter.1);
                let name = rest.trim().trim_matches(':').trim().to_string();
                let id = format!("ms_{}_Q{}{}", name.replace(' ', "_"), quarter.0, quarter.1);
                milestones.push(Milestone {
                    id,
                    name,
                    due,
                    status: MilestoneStatus::Planned,
                    completed_at: None,
                    phase: None,
                    focus_area: None,
                });
            }

            // Parse simple table rows: | Q1 2026 | Focus | Key Features |
            if line.starts_with('|') && line.contains("Q") && line.contains("|") {
                let parts: Vec<_> = line.split('|').map(|s| s.trim()).collect();
                if parts.len() >= 4 && parts[1].starts_with('Q') {
                    if let Some((q, y)) = Self::parse_quarter_and_year(parts[1]) {
                        let due = Self::end_of_quarter(q, y);
                        let focus = parts[2].to_string();
                        let features = parts[3].to_string();
                        for feat in features.split(',') {
                            let name = format!("{} - {}", focus, feat.trim());
                            if name.trim().is_empty() {
                                continue;
                            }
                            let id = format!("ms_{}_Q{}{}", name.replace(' ', "_"), q, y);
                            milestones.push(Milestone {
                                id,
                                name,
                                due,
                                status: MilestoneStatus::Planned,
                                completed_at: None,
                                phase: None,
                                focus_area: Some(focus.clone()),
                            });
                        }
                    }
                }
            }
        }

        Roadmap { milestones }
    }

    pub fn mark_completed(roadmap: &mut Roadmap, milestone_id: &str, when: Option<NaiveDateTime>) {
        if let Some(ms) = roadmap.milestones.iter_mut().find(|m| m.id == milestone_id) {
            ms.status = MilestoneStatus::Completed;
            ms.completed_at = Some(when.unwrap_or_else(|| Utc::now().naive_utc()));
        }
    }

    pub fn adherence_report(roadmap: &Roadmap, now: NaiveDateTime) -> AdherenceReport {
        let due: Vec<&Milestone> = roadmap.milestones.iter().filter(|m| m.due <= now).collect();
        let mut completed_on_time = 0usize;
        let mut completed_late = 0usize;
        let mut overdue = 0usize;

        for m in &due {
            match (m.status.clone(), m.completed_at) {
                (MilestoneStatus::Completed, Some(done)) if done <= m.due => completed_on_time += 1,
                (MilestoneStatus::Completed, Some(done)) if done > m.due => completed_late += 1,
                (MilestoneStatus::Completed, None) => completed_late += 1,
                (MilestoneStatus::Planned | MilestoneStatus::InProgress, _) => overdue += 1,
                (MilestoneStatus::Delayed, _) => overdue += 1,
            }
        }

        let on_track = overdue == 0;
        AdherenceReport {
            generated_at: now,
            total: roadmap.milestones.len(),
            due_by_now: due.len(),
            completed_on_time,
            completed_late,
            overdue,
            on_track,
        }
    }

    fn parse_quarter_line(line: &str) -> Option<((u32, i32), &str)> {
        // e.g., "1. **Q1 2026**: Database query optimization"
        let mut idx = line.find("Q")?;
        let slice = &line[idx..];
        // Accept formats like Q1 2026 or Q1 2026**
        let mut tokens = slice
            .trim_matches('*')
            .split_whitespace()
            .take(2)
            .collect::<Vec<_>>();
        if tokens.len() < 2 {
            return None;
        }
        if let Some((q, _)) = Self::parse_quarter(tokens[0]) {
            let year = tokens[1].trim_matches('*');
            let y2: i32 = year.parse().ok()?;
            // rest after ':' if present
            if let Some(colon) = line.find(':') {
                return Some(((q, y2), &line[colon + 1..]));
            }
            return Some(((q, y2), ""));
        }
        None
    }

    fn parse_quarter(token: &str) -> Option<(u32, i32)> {
        // token: Q1 or Q4
        if !token.starts_with('Q') {
            return None;
        }
        let q: u32 = token[1..].parse().ok()?;
        if !(1..=4).contains(&q) {
            return None;
        }
        // Year is parsed separately by caller where needed.
        // Return dummy year 0 here; caller validates.
        Some((q, 0))
    }

    fn parse_quarter_and_year(s: &str) -> Option<(u32, i32)> {
        // s like "Q1 2026"
        let mut it = s.split_whitespace();
        let qtok = it.next()?;
        let ytok = it.next()?;
        let (q, _) = Self::parse_quarter(qtok)?;
        let y: i32 = ytok.parse().ok()?;
        Some((q, y))
    }

    fn end_of_quarter(mut q: u32, mut y: i32) -> NaiveDateTime {
        // If year not parsed by parse_quarter (0), try to infer from text around
        if y == 0 {
            y = Utc::now().year();
        }
        let (month, day) = match q {
            1 => (3, 31),
            2 => (6, 30),
            3 => (9, 30),
            _ => (12, 31),
        };
        NaiveDate::from_ymd_opt(y, month, day)
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(y, 12, 31).unwrap())
            .and_hms_opt(23, 59, 59)
            .unwrap()
    }
}
