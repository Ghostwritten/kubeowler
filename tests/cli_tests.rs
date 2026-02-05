use clap::Parser;
use kubeowler::cli::{Args, Commands, InspectionType};

#[test]
fn test_cli_parsing() {
    // Default check
    let args = Args::try_parse_from(&["kubeowler", "check"]).unwrap();
    let Commands::Check { .. } = &args.command;

    // With namespace
    let args = Args::try_parse_from(&["kubeowler", "check", "-n", "kube-system"]).unwrap();
    let Commands::Check { namespace, .. } = &args.command;
    assert_eq!(namespace.as_deref(), Some("kube-system"));

    // With custom output
    let args = Args::try_parse_from(&["kubeowler", "check", "-o", "custom-report.md"]).unwrap();
    let Commands::Check { output, .. } = &args.command;
    assert_eq!(output.as_deref(), Some("custom-report.md"));

    // With format
    let args = Args::try_parse_from(&["kubeowler", "check", "-f", "json"]).unwrap();
    let Commands::Check { .. } = &args.command;
}

#[test]
fn test_inspection_type_variants() {
    use clap::ValueEnum;

    let types = InspectionType::value_variants();
    assert!(types.len() >= 6); // We have at least 6 inspection types

    // Test that all types can be parsed
    assert!(matches!(
        "all".parse::<InspectionType>(),
        Ok(InspectionType::All)
    ));
    assert!(matches!(
        "nodes".parse::<InspectionType>(),
        Ok(InspectionType::Nodes)
    ));
    assert!(matches!(
        "pods".parse::<InspectionType>(),
        Ok(InspectionType::Pods)
    ));
    assert!(matches!(
        "security".parse::<InspectionType>(),
        Ok(InspectionType::Security)
    ));
}
