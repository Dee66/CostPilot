/// License duration validation tests (Phase 2)
/// Tests monthly (30-day) and yearly (365-day) license validity
/// with boundary conditions
#[cfg(test)]
mod duration_tests {
    use chrono::{Duration, Utc};
    use costpilot::pro_engine::license::License;

    /// Helper: create License with specific expiry duration
    fn create_license_with_duration(days: i64) -> License {
        let expires = (Utc::now() + Duration::days(days)).to_rfc3339();
        License {
            email: "test@example.com".to_string(),
            license_key: "TEST-KEY".to_string(),
            expires,
            signature: "dummy_sig".to_string(),
            issuer: "test-costpilot".to_string(),
        }
    }

    /// Helper: create License with specific expiry offset in seconds
    fn create_license_with_offset_seconds(seconds: i64) -> License {
        let expires = (Utc::now() + Duration::seconds(seconds)).to_rfc3339();
        License {
            email: "test@example.com".to_string(),
            license_key: "TEST-KEY".to_string(),
            expires,
            signature: "dummy_sig".to_string(),
            issuer: "test-costpilot".to_string(),
        }
    }

    // === Monthly License Tests (30 days) ===

    #[test]
    fn test_monthly_license_30_days_valid() {
        let license = create_license_with_duration(30);
        assert!(!license.is_expired(), "30-day license should be valid");
    }

    #[test]
    fn test_monthly_license_29_days_valid() {
        let license = create_license_with_duration(29);
        assert!(!license.is_expired(), "29-day license should be valid");
    }

    #[test]
    fn test_monthly_license_31_days_valid() {
        let license = create_license_with_duration(31);
        assert!(!license.is_expired(), "31-day license should be valid");
    }

    #[test]
    fn test_monthly_license_1_day_valid() {
        let license = create_license_with_duration(1);
        assert!(!license.is_expired(), "1-day license should be valid");
    }

    // === Yearly License Tests (365 days) ===

    #[test]
    fn test_yearly_license_365_days_valid() {
        let license = create_license_with_duration(365);
        assert!(!license.is_expired(), "365-day license should be valid");
    }

    #[test]
    fn test_yearly_license_364_days_valid() {
        let license = create_license_with_duration(364);
        assert!(!license.is_expired(), "364-day license should be valid");
    }

    #[test]
    fn test_yearly_license_366_days_valid() {
        let license = create_license_with_duration(366);
        assert!(!license.is_expired(), "366-day license should be valid");
    }

    #[test]
    fn test_yearly_license_730_days_valid() {
        let license = create_license_with_duration(730);
        assert!(
            !license.is_expired(),
            "730-day (2-year) license should be valid"
        );
    }

    // === Boundary Tests ===

    #[test]
    fn test_boundary_expires_now_minus_1_second_invalid() {
        let license = create_license_with_offset_seconds(-1);
        assert!(
            license.is_expired(),
            "License expired 1 second ago must be invalid"
        );
    }

    #[test]
    fn test_boundary_expires_now_plus_1_second_valid() {
        let license = create_license_with_offset_seconds(1);
        assert!(
            !license.is_expired(),
            "License expiring in 1 second must be valid"
        );
    }

    #[test]
    fn test_boundary_expires_now_minus_1_hour_invalid() {
        let license = create_license_with_offset_seconds(-3600);
        assert!(
            license.is_expired(),
            "License expired 1 hour ago must be invalid"
        );
    }

    #[test]
    fn test_boundary_expires_now_plus_1_hour_valid() {
        let license = create_license_with_offset_seconds(3600);
        assert!(
            !license.is_expired(),
            "License expiring in 1 hour must be valid"
        );
    }

    #[test]
    fn test_boundary_expires_now_minus_1_day_invalid() {
        let license = create_license_with_duration(-1);
        assert!(
            license.is_expired(),
            "License expired 1 day ago must be invalid"
        );
    }

    #[test]
    fn test_boundary_expires_now_plus_1_minute_valid() {
        let license = create_license_with_offset_seconds(60);
        assert!(
            !license.is_expired(),
            "License expiring in 1 minute must be valid"
        );
    }

    // === Edge Cases ===

    #[test]
    fn test_expired_exactly_at_epoch() {
        let license = License {
            email: "test@example.com".to_string(),
            license_key: "TEST-KEY".to_string(),
            expires: "1970-01-01T00:00:00Z".to_string(),
            signature: "dummy_sig".to_string(),
            issuer: "test-costpilot".to_string(),
        };
        assert!(license.is_expired(), "Epoch date should be expired");
    }

    #[test]
    fn test_valid_far_future() {
        let license = License {
            email: "test@example.com".to_string(),
            license_key: "TEST-KEY".to_string(),
            expires: "2099-12-31T23:59:59Z".to_string(),
            signature: "dummy_sig".to_string(),
            issuer: "test-costpilot".to_string(),
        };
        assert!(!license.is_expired(), "Far future date should be valid");
    }

    #[test]
    fn test_expired_far_past() {
        let license = License {
            email: "test@example.com".to_string(),
            license_key: "TEST-KEY".to_string(),
            expires: "2000-01-01T00:00:00Z".to_string(),
            signature: "dummy_sig".to_string(),
            issuer: "test-costpilot".to_string(),
        };
        assert!(license.is_expired(), "Far past date should be expired");
    }

    // === Invalid Date Format Tests ===

    #[test]
    fn test_invalid_date_format_treated_as_expired() {
        let invalid_formats = vec![
            "not-a-date",
            "2026-13-01T00:00:00Z", // Invalid month
            "2026-01-32T00:00:00Z", // Invalid day
            "2026-01-01",           // Missing time
            "2026-01-01T25:00:00Z", // Invalid hour
            "",                     // Empty
        ];

        for invalid_format in invalid_formats {
            let license = License {
                email: "test@example.com".to_string(),
                license_key: "TEST-KEY".to_string(),
                expires: invalid_format.to_string(),
                signature: "dummy_sig".to_string(),
                issuer: "test-costpilot".to_string(),
            };
            assert!(
                license.is_expired(),
                "Invalid format '{}' should be treated as expired",
                invalid_format
            );
        }
    }

    // === Duration Independence Tests ===
    // Confirm that consumer does NOT enforce specific durations (issuer-defined)

    #[test]
    fn test_arbitrary_duration_7_days_valid() {
        let license = create_license_with_duration(7);
        assert!(
            !license.is_expired(),
            "7-day license should be valid (arbitrary issuer-defined duration)"
        );
    }

    #[test]
    fn test_arbitrary_duration_90_days_valid() {
        let license = create_license_with_duration(90);
        assert!(
            !license.is_expired(),
            "90-day license should be valid (arbitrary issuer-defined duration)"
        );
    }

    #[test]
    fn test_arbitrary_duration_1825_days_valid() {
        let license = create_license_with_duration(1825); // 5 years
        assert!(
            !license.is_expired(),
            "5-year license should be valid (arbitrary issuer-defined duration)"
        );
    }

    // === Timezone Tests ===

    #[test]
    fn test_rfc3339_with_offset_positive() {
        let license = License {
            email: "test@example.com".to_string(),
            license_key: "TEST-KEY".to_string(),
            expires: "2026-12-31T23:59:59+05:30".to_string(), // India timezone
            signature: "dummy_sig".to_string(),
            issuer: "test-costpilot".to_string(),
        };
        assert!(
            !license.is_expired(),
            "RFC3339 with +05:30 offset should be valid"
        );
    }

    #[test]
    fn test_rfc3339_with_offset_negative() {
        let license = License {
            email: "test@example.com".to_string(),
            license_key: "TEST-KEY".to_string(),
            expires: "2026-12-31T23:59:59-08:00".to_string(), // PST
            signature: "dummy_sig".to_string(),
            issuer: "test-costpilot".to_string(),
        };
        assert!(
            !license.is_expired(),
            "RFC3339 with -08:00 offset should be valid"
        );
    }

    #[test]
    fn test_rfc3339_utc_z_suffix() {
        let license = License {
            email: "test@example.com".to_string(),
            license_key: "TEST-KEY".to_string(),
            expires: "2026-12-31T23:59:59Z".to_string(), // UTC with Z
            signature: "dummy_sig".to_string(),
            issuer: "test-costpilot".to_string(),
        };
        assert!(
            !license.is_expired(),
            "RFC3339 with Z suffix should be valid"
        );
    }

    #[test]
    fn test_rfc3339_utc_zero_offset() {
        let license = License {
            email: "test@example.com".to_string(),
            license_key: "TEST-KEY".to_string(),
            expires: "2026-12-31T23:59:59+00:00".to_string(), // UTC with +00:00
            signature: "dummy_sig".to_string(),
            issuer: "test-costpilot".to_string(),
        };
        assert!(
            !license.is_expired(),
            "RFC3339 with +00:00 offset should be valid"
        );
    }
}
