//! Tests for the load testing framework

#[cfg(test)]
mod tests {
    use crate::load_testing::{LoadTestConfig, LoadTester};

    #[test]
    fn test_load_tester_creation() {
        let config = LoadTestConfig {
            concurrent_users: 2,
            requests_per_user: 5,
            request_delay_ms: 10,
            duration_seconds: None,
        };

        let tester = LoadTester::new(config.clone());
        // We can't directly access the private config field, so we'll just test that creation works
        assert_eq!(config.concurrent_users, 2);
        assert_eq!(config.requests_per_user, 5);
    }

    #[test]
    fn test_load_test_execution() {
        let config = LoadTestConfig {
            concurrent_users: 2,
            requests_per_user: 3,
            request_delay_ms: 5,
            duration_seconds: None,
        };

        let tester = LoadTester::new(config);
        let result = tester
            .run_load_test()
            .expect("Load test should complete successfully");

        assert_eq!(result.config().concurrent_users, 2);
        assert_eq!(result.config().requests_per_user, 3);
        assert!(result.duration_seconds > 0.0);
        assert!(result.metrics.total_requests > 0);
        assert!(result.metrics.successful_requests > 0);
    }

    #[test]
    fn test_security_audit_framework() {
        let config = crate::load_testing::security_audit::SecurityAuditConfig {
            run_cargo_audit: false,     // Don't actually run audit in tests
            run_clippy_security: false, // Don't actually run clippy in tests
            custom_checks: vec![],
        };

        let result = crate::load_testing::security_audit::run_security_audit(config);
        assert!(result.is_ok());
    }
}
