use chrono::Utc;
use kubeowler::inspections::types::*;
use kubeowler::reporting::{issue_to_resource_key, ReportGenerator, REPORT_RESOURCE_ORDER};
use std::collections::HashMap;
use tempfile::tempdir;

fn make_issue(category: &str, rule_id: Option<&str>) -> Issue {
    Issue {
        severity: IssueSeverity::Info,
        category: category.to_string(),
        description: String::new(),
        resource: None,
        recommendation: String::new(),
        rule_id: rule_id.map(String::from),
    }
}

#[test]
fn test_issue_to_resource_key_mapping() {
    assert_eq!(issue_to_resource_key(&make_issue("Pod", None)), "Pod");
    assert_eq!(issue_to_resource_key(&make_issue("Container", None)), "Pod");
    assert_eq!(issue_to_resource_key(&make_issue("Node", None)), "Node");
    assert_eq!(
        issue_to_resource_key(&make_issue("Service", None)),
        "Service"
    );
    assert_eq!(
        issue_to_resource_key(&make_issue("Certificates", None)),
        "Certificate"
    );
    assert_eq!(
        issue_to_resource_key(&make_issue("ControlPlane", None)),
        "Control Plane"
    );
    assert_eq!(
        issue_to_resource_key(&make_issue("Autoscaling", None)),
        "HPA"
    );
    assert_eq!(issue_to_resource_key(&make_issue("Policy", None)), "Policy");
    assert_eq!(
        issue_to_resource_key(&make_issue("Observability", None)),
        "Observability"
    );
    assert_eq!(
        issue_to_resource_key(&make_issue("Security", None)),
        "Security"
    );
    assert_eq!(
        issue_to_resource_key(&make_issue("Resource Management", None)),
        "Resource Management"
    );
    assert_eq!(
        issue_to_resource_key(&make_issue("Batch", Some("BATCH-001"))),
        "CronJob"
    );
    assert_eq!(
        issue_to_resource_key(&make_issue("Batch", Some("BATCH-003"))),
        "CronJob"
    );
    assert_eq!(
        issue_to_resource_key(&make_issue("Batch", Some("BATCH-004"))),
        "Job"
    );
    assert_eq!(
        issue_to_resource_key(&make_issue("Batch", Some("BATCH-005"))),
        "Job"
    );
    assert_eq!(
        issue_to_resource_key(&make_issue("PersistentVolume", None)),
        "PersistentVolume"
    );
    assert_eq!(
        issue_to_resource_key(&make_issue("ClusterRole", None)),
        "ClusterRole"
    );
}

#[test]
fn test_report_resource_order_non_empty() {
    assert!(!REPORT_RESOURCE_ORDER.is_empty());
    assert!(REPORT_RESOURCE_ORDER.contains(&"Pod"));
    assert!(REPORT_RESOURCE_ORDER.contains(&"Node"));
    assert!(REPORT_RESOURCE_ORDER.contains(&"Certificate"));
}

#[tokio::test]
async fn test_report_generation() {
    let generator = ReportGenerator::new();

    // Create test data
    let cluster_report = ClusterReport {
        cluster_name: "test-cluster".to_string(),
        report_id: "test-123".to_string(),
        timestamp: Utc::now(),
        overall_score: 85.5,
        inspections: vec![InspectionResult {
            inspection_type: "Node Health".to_string(),
            timestamp: Utc::now(),
            overall_score: 90.0,
            checks: vec![CheckResult {
                name: "Node Readiness".to_string(),
                description: "Test check".to_string(),
                status: CheckStatus::Pass,
                score: 100.0,
                max_score: 100.0,
                details: Some("All nodes ready".to_string()),
                recommendations: vec![],
            }],
            summary: InspectionSummary {
                total_checks: 1,
                passed_checks: 1,
                warning_checks: 0,
                critical_checks: 0,
                error_checks: 0,
                issues: vec![],
            },
            certificate_expiries: None,
            pod_container_states: None,
            namespace_summary_rows: None,
        }],
        executive_summary: ExecutiveSummary {
            health_status: HealthStatus::Good,
            key_findings: vec!["Test finding".to_string()],
            priority_recommendations: vec!["Test recommendation".to_string()],
            score_breakdown: {
                let mut map = HashMap::new();
                map.insert("Node Health".to_string(), 90.0);
                map
            },
        },
        cluster_overview: None,
        node_inspection_results: None,
        recent_events: None,
    };

    // Test report generation
    let temp_dir = tempdir().unwrap();
    let report_path = temp_dir.path().join("test-report.md");
    let report_path_str = report_path.to_str().unwrap();

    let result = generator
        .generate_report(&cluster_report, report_path_str)
        .await;
    assert!(result.is_ok());

    // Check that files were created
    assert!(report_path.exists());

    let summary_path = temp_dir.path().join("test-report-summary.md");
    assert!(summary_path.exists());

    // Check content (report uses English title and Executive Summary)
    let content = std::fs::read_to_string(&report_path).unwrap();
    assert!(content.contains("Executive Summary"));
    assert!(content.contains("test-cluster"));
    assert!(content.contains("85.5"));
}

#[test]
fn test_report_formatting() {
    let _generator = ReportGenerator::new();

    // Test that the generator can be created
    assert!(true); // Basic test for now

    // Test scoring integration
    let scoring_engine = kubeowler::scoring::scoring_engine::ScoringEngine::new();
    let health_status = scoring_engine.get_health_status(85.0);
    assert!(matches!(health_status, HealthStatus::Good));
}
