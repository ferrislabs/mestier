use super::*;

#[test]
fn only_reads_are_read_only_and_every_delete_names_what_it_removed() {
    for spec in SPECS {
        let verb = spec.kind.rsplit('.').next().unwrap();
        let reads = matches!(verb, "list" | "get");
        assert_eq!(spec.read_only, reads, "{}", spec.kind);
        if verb == "delete" || verb == "remove" {
            let sample = (spec.sample_output)();
            assert!(sample.get("deleted").or(sample.get("removed")).is_some());
        }
    }
}

#[test]
fn every_list_can_be_paged() {
    for spec in SPECS.iter().filter(|spec| spec.kind.ends_with(".list")) {
        if spec.kind == "mestier.cost_basis.list" {
            continue;
        }
        let names: Vec<&str> = spec.fields.iter().map(|field| field.name).collect();
        assert_eq!(names, ["limit", "offset"], "{}", spec.kind);
    }
}
