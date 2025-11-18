use chrono::{NaiveDateTime, Utc};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Strategies for budgeting alternative paths to support token price resilience.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BudgetStrategy {
    GradualLiquidity,
    LeanLaunch,
    Other,
}

impl BudgetStrategy {
    pub fn from_str(value: &str) -> Self {
        match value.trim().to_lowercase().as_str() {
            "gradual liquidity building" | "budget alternative" | "gradual" => {
                BudgetStrategy::GradualLiquidity
            }
            "start small & grow" | "lean launch option" | "lean" => BudgetStrategy::LeanLaunch,
            _ => BudgetStrategy::Other,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            BudgetStrategy::GradualLiquidity => "GradualLiquidity",
            BudgetStrategy::LeanLaunch => "LeanLaunch",
            BudgetStrategy::Other => "Other",
        }
    }
}

impl fmt::Display for BudgetStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Represents a budget path that can be simulated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAlternative {
    pub id: String,
    pub option_name: String,
    pub details: String,
    pub strategy: BudgetStrategy,
    pub start_amount: Decimal,
    pub growth_rate: Decimal,
    pub duration_months: i64,
    pub created_at: NaiveDateTime,
}

/// A point in the projected budget simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetSchedulePoint {
    pub month: i64,
    pub amount: Decimal,
}

impl BudgetAlternative {
    /// Simulate the budget path for a number of months (or the stored duration if not provided).
    pub fn simulate_schedule(&self, months: Option<i64>) -> Vec<BudgetSchedulePoint> {
        let window = months.unwrap_or(self.duration_months).max(1);
        let mut schedule = Vec::with_capacity(window as usize);
        let mut current = self.start_amount.round_dp(8);
        let lean_increment = self
            .start_amount
            .checked_mul(self.growth_rate)
            .unwrap_or(Decimal::ZERO);
        let fallback_increment = lean_increment / Decimal::from_f64(2.0).unwrap_or(Decimal::ZERO);

        for month in 1..=window {
            schedule.push(BudgetSchedulePoint {
                month,
                amount: current,
            });
            if month < window {
                let next = match self.strategy {
                    BudgetStrategy::GradualLiquidity => {
                        current
                            + current
                                .checked_mul(self.growth_rate)
                                .unwrap_or(Decimal::ZERO)
                    }
                    BudgetStrategy::LeanLaunch => current + lean_increment,
                    BudgetStrategy::Other => current + fallback_increment,
                };
                current = next.round_dp(8);
            }
        }
        schedule
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn simulate_gradual_liquidity() {
        let alt = BudgetAlternative {
            id: "gradual".to_string(),
            option_name: "Gradual Liquidity".to_string(),
            details: "Build intentionally".to_string(),
            strategy: BudgetStrategy::GradualLiquidity,
            start_amount: Decimal::new(1000, 0),
            growth_rate: Decimal::new(5, 2), // 0.05
            duration_months: 3,
            created_at: Utc::now().naive_utc(),
        };
        let schedule = alt.simulate_schedule(None);
        assert_eq!(schedule.len(), 3);
        assert_eq!(schedule[0].amount, Decimal::new(1000, 0));
        assert!(schedule[1].amount > schedule[0].amount);
    }

    #[test]
    fn simulate_lean_launch() {
        let alt = BudgetAlternative {
            id: "lean".to_string(),
            option_name: "Lean Launch".to_string(),
            details: "Start small".to_string(),
            strategy: BudgetStrategy::LeanLaunch,
            start_amount: Decimal::new(500, 0),
            growth_rate: Decimal::new(2, 2), // 0.02
            duration_months: 4,
            created_at: Utc::now().naive_utc(),
        };
        let schedule = alt.simulate_schedule(Some(2));
        assert_eq!(schedule.len(), 2);
        assert_eq!(schedule[0].amount, Decimal::new(500, 0));
        assert_eq!(
            schedule[1].amount,
            Decimal::new(500, 0) + Decimal::new(10, 2) // 0.02 * 500
        );
    }
}

#[cfg(all(test, feature = "database-tests"))]
mod db_tests {
    use super::*;
    use crate::database::MySqlDatabase;
    use sqlx::MySqlPool;
    use std::env;
    use std::env;

    async fn init_test_db() -> Option<(MySqlDatabase, MySqlPool)> {
        let db_url = env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "mysql://pproject:pprojectpassword@localhost/p_project_test".to_string()
        });
        let db = MySqlDatabase::new(&db_url).await.ok()?;
        db.init_tables().await.ok()?;
        let pool = MySqlPool::connect(&db_url).await.ok()?;
        let _ = sqlx::query("DELETE FROM budget_alternatives")
            .execute(&pool)
            .await;
        Some((db, pool))
    }

    #[tokio::test]
    async fn create_and_list_budget_alternative() {
        let (db, _pool) = match init_test_db().await {
            Some(v) => v,
            None => {
                println!("Budget alternatives DB test skipped (DB unavailable)");
                return;
            }
        };
        let alt = db
            .create_budget_alternative(
                "budget-plan",
                "Gradual Liquidity",
                "Test this path",
                BudgetStrategy::GradualLiquidity,
                Decimal::new(1000, 0),
                Decimal::new(5, 2),
                6,
            )
            .await
            .expect("inserted");
        let list = db.list_budget_alternatives(10).await.unwrap();
        assert!(list.iter().any(|item| item.id == alt.id));
        let fetched = db
            .get_budget_alternative(&alt.id)
            .await
            .unwrap()
            .expect("found");
        assert_eq!(fetched.option_name, alt.option_name);
    }
}
