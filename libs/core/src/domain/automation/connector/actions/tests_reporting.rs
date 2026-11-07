use super::*;

#[test]
fn every_report_is_read_only_and_asks_for_a_period() {
    for spec in SPECS {
        assert!(spec.read_only, "{}", spec.kind);
        let names: Vec<&str> = spec.fields.iter().map(|field| field.name).collect();
        assert_eq!(names, ["from", "to"], "{}", spec.kind);
        assert!(spec.fields.iter().all(|field| field.required));
    }
}
