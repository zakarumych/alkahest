#![allow(dead_code)]

use alkahest::{ExactSize, Formula, Schema, SchemaKind};
use alkahest_schema::{Snapshot, check};
use serde_json::{Value, json};

#[derive(Formula)]
struct Original {
    field: u8,
}
#[derive(Formula)]
struct SilentEdit {
    field: u16,
}
#[derive(Formula)]
struct PinEdit {
    #[alkahest(version = 1)]
    field: u8,
}
#[derive(Formula)]
#[alkahest(version = 1)]
struct Bumped {
    field: u16,
}
#[derive(Formula)]
#[alkahest(version = 1, versions(0 = SilentEdit))]
struct RewrittenHistory {
    field: u16,
}
#[derive(Formula)]
#[alkahest(version = 1, versions(0 = Original))]
struct RetainedHistory {
    field: u16,
}
#[derive(Formula)]
#[alkahest(version = 0, versions(1 = Bumped))]
struct FutureLinked {
    field: u8,
}
#[derive(Formula)]
struct Parent {
    child: Original,
}
#[derive(Formula)]
struct SilentParent {
    child: SilentEdit,
}
#[derive(Formula)]
#[alkahest(version = 1)]
struct BumpedParent {
    child: SilentEdit,
}

fn snapshot<F: Formula<V>, const V: u32>() -> Snapshot {
    let mut snapshot = Snapshot::new();
    snapshot.add::<F, V>("example.Root").unwrap();
    snapshot
}
fn value<F: Formula<V>, const V: u32>() -> Value {
    serde_json::from_str(&snapshot::<F, V>().to_json().unwrap()).unwrap()
}
fn parsed(value: &Value) -> Result<Snapshot, alkahest_schema::Error> {
    Snapshot::from_json(&serde_json::to_string(value).unwrap())
}

#[test]
fn unchanged_and_roundtrip_are_canonical() {
    let snapshot = snapshot::<Original, 0>();
    let json = snapshot.to_json().unwrap();
    let decoded = Snapshot::from_json(&json).unwrap();
    assert_eq!(json, decoded.to_json().unwrap());
    assert!(check(&snapshot, &decoded).unwrap().is_empty());
}

#[test]
fn silent_change_and_child_pin_require_parent_bump() {
    let old = snapshot::<Original, 0>();
    for new in [snapshot::<SilentEdit, 0>(), snapshot::<PinEdit, 0>()] {
        let failures = check(&old, &new).unwrap();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].id, "example.Root");
        assert!(failures[0].path.starts_with("revisions[0].nodes"));
    }
    let failures = check(&old, &snapshot::<PinEdit, 0>()).unwrap();
    assert!(failures[0].path.ends_with(".version"));
}

#[test]
fn nested_changes_are_observed() {
    assert_eq!(
        check(&snapshot::<Parent, 0>(), &snapshot::<SilentParent, 0>())
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn parent_bump_allows_replacement_but_registered_child_stays_protected() {
    let mut old = snapshot::<Parent, 0>();
    let mut new = snapshot::<BumpedParent, 1>();
    assert!(check(&old, &new).unwrap().is_empty());
    old.add::<Original, 0>("example.Child").unwrap();
    new.add::<SilentEdit, 0>("example.Child").unwrap();
    let failures = check(&old, &new).unwrap();
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].id, "example.Child");
}

#[test]
fn retained_history_is_immutable_and_optional_when_advancing() {
    let old = snapshot::<Original, 0>();
    assert!(check(&old, &snapshot::<Bumped, 1>()).unwrap().is_empty());
    assert!(
        check(&old, &snapshot::<RetainedHistory, 1>())
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        check(&old, &snapshot::<RewrittenHistory, 1>())
            .unwrap()
            .len(),
        1
    );
    assert!(snapshot::<FutureLinked, 0>().to_json().is_ok());
}

#[test]
fn removal_regression_and_new_roots() {
    let old = snapshot::<Bumped, 1>();
    assert!(
        check(&old, &snapshot::<Original, 0>())
            .unwrap()
            .iter()
            .any(|v| v.path == "current")
    );
    let mut new = Snapshot::new();
    new.add::<Original, 0>("another.ID").unwrap();
    assert_eq!(
        check(&old, &new).unwrap()[0].message,
        "registered formula was removed"
    );
    new.add::<Bumped, 1>("example.Root").unwrap();
    assert!(check(&old, &new).unwrap().is_empty());
}

#[test]
fn registration_order_does_not_change_output_and_add_errors_are_atomic() {
    let mut a = Snapshot::new();
    a.add::<Original, 0>("a").unwrap();
    a.add::<Bumped, 1>("b").unwrap();
    let mut b = Snapshot::new();
    b.add::<Bumped, 1>("b").unwrap();
    b.add::<Original, 0>("a").unwrap();
    assert_eq!(a.to_json().unwrap(), b.to_json().unwrap());
    let before = a.to_json().unwrap();
    assert!(a.add::<Original, 0>("a").is_err());
    assert!(a.add::<Original, 0>("").is_err());
    assert!(a.add::<Opaque, 0>("opaque").is_err());
    assert_eq!(before, a.to_json().unwrap());
}

struct Opaque;
impl Formula for Opaque {
    type StackSize<const N: usize> = ExactSize<0>;
    type HeapSize<const N: usize> = ExactSize<0>;
    const INHABITED: bool = true;
}
struct Recursive;
impl Formula for Recursive {
    type StackSize<const N: usize> = ExactSize<0>;
    type HeapSize<const N: usize> = ExactSize<0>;
    const INHABITED: bool = true;
    const SCHEMA: Schema = Schema::new::<Self, 0>(SchemaKind::Option(alkahest::ElementSchema {
        schema: alkahest::schema::<Self, 0>,
        indirect: true,
    }));
}
struct InvalidTable;
impl Formula for InvalidTable {
    type StackSize<const N: usize> = ExactSize<1>;
    type HeapSize<const N: usize> = ExactSize<0>;
    const INHABITED: bool = true;
    const SCHEMA: Schema =
        Schema::new::<Self, 0>(SchemaKind::Primitive(alkahest::Primitive::Unsigned(1)));
    const VERSIONS: &'static [fn() -> &'static Schema] = &[alkahest::schema::<u8, 1>];
}

#[test]
fn opaque_recursive_and_invalid_tables_fail_closed() {
    let mut snapshot = Snapshot::new();
    assert!(
        snapshot
            .add::<Opaque, 0>("opaque")
            .unwrap_err()
            .to_string()
            .contains("opaque")
    );
    assert!(
        snapshot
            .add::<Recursive, 0>("recursive")
            .unwrap_err()
            .to_string()
            .contains("depth")
    );
    assert!(
        snapshot
            .add::<InvalidTable, 0>("bad")
            .unwrap_err()
            .to_string()
            .contains("missing local")
    );
    assert!(snapshot.to_json().is_err());
}

#[test]
fn malformed_manifests_fail_closed() {
    let original = value::<Original, 0>();
    let mutations: Vec<Box<dyn Fn(&mut Value)>> = vec![
        Box::new(|v| v["format"] = json!(2)),
        Box::new(|v| v["unexpected"] = json!(true)),
        Box::new(|v| v["pointer_width"] = json!(7)),
        Box::new(|v| v["roots"] = json!([])),
        Box::new(|v| v["roots"][0]["id"] = json!("")),
        Box::new(|v| v["roots"][0]["current"] = json!(99)),
        Box::new(|v| {
            let root = v["roots"][0].clone();
            v["roots"].as_array_mut().unwrap().push(root);
        }),
        Box::new(|v| v["roots"][0]["revisions"][0]["version"] = json!(1)),
        Box::new(|v| v["roots"][0]["revisions"][0]["nodes"][0]["layouts"] = json!([])),
        Box::new(|v| v["roots"][0]["revisions"][0]["nodes"][0]["layouts"][0]["width"] = json!(2)),
        Box::new(|v| {
            v["roots"][0]["revisions"][0]["nodes"][0]["shape"]["fields"][0]["element"]["node"] =
                json!(0)
        }),
        Box::new(|v| {
            v["roots"][0]["revisions"][0]["nodes"][0]["shape"]["fields"][0]["element"]["node"] =
                json!(99)
        }),
        Box::new(|v| v["roots"][0]["revisions"][0]["nodes"][0]["shape"]["fields"] = json!([])),
        Box::new(|v| v["roots"][0]["revisions"][0]["nodes"][1]["shape"]["bytes"] = json!(3)),
    ];
    for (index, mutate) in mutations.into_iter().enumerate() {
        let mut value = original.clone();
        mutate(&mut value);
        assert!(parsed(&value).is_err(), "mutation {index} accepted");
    }
    let json = serde_json::to_string(&original).unwrap().replacen(
        "\"format\":1",
        "\"format\":1,\"format\":1",
        1,
    );
    assert!(Snapshot::from_json(&json).is_err());
}

#[test]
fn differing_targets_are_not_comparable_and_bounds_fit_target() {
    let original = value::<Original, 0>();
    let old = parsed(&original).unwrap();
    let mut other = original.clone();
    other["pointer_width"] = json!(32);
    let new = parsed(&other).unwrap();
    if usize::BITS != 32 {
        assert!(
            check(&old, &new)
                .unwrap_err()
                .to_string()
                .contains("pointer widths")
        );
    }
    other["roots"][0]["revisions"][0]["nodes"][0]["layouts"][0]["stack"]["size"] = json!(u64::MAX);
    assert!(parsed(&other).is_err());
}

#[test]
fn layouts_names_placement_and_encoding_are_checked() {
    let original = value::<Original, 0>();
    let old = parsed(&original).unwrap();
    let mutations: Vec<Box<dyn Fn(&mut Value)>> = vec![
        Box::new(|v| {
            v["roots"][0]["revisions"][0]["nodes"][0]["layouts"][15]["stack"]["size"] = json!(2)
        }),
        Box::new(|v| {
            v["roots"][0]["revisions"][0]["nodes"][0]["shape"]["fields"][0]["name"] =
                json!("renamed")
        }),
        Box::new(|v| {
            v["roots"][0]["revisions"][0]["nodes"][0]["shape"]["fields"][0]["element"]["indirect"] =
                json!(true)
        }),
        Box::new(|v| v["roots"][0]["revisions"][0]["nodes"][1]["shape"]["kind"] = json!("signed")),
    ];
    for mutate in mutations {
        let mut value = original.clone();
        mutate(&mut value);
        assert_eq!(check(&old, &parsed(&value).unwrap()).unwrap().len(), 1);
    }
}

#[derive(Formula)]
struct Pair {
    first: u8,
    second: u8,
}
#[derive(Formula)]
enum Choices {
    A,
    B(u8),
}

#[test]
fn field_order_and_variant_tags_are_checked() {
    let original = value::<Pair, 0>();
    let mut changed = original.clone();
    let fields = changed["roots"][0]["revisions"][0]["nodes"][0]["shape"]["fields"]
        .as_array_mut()
        .unwrap();
    fields[0]["name"] = json!("second");
    fields[1]["name"] = json!("first");
    assert_eq!(
        check(&parsed(&original).unwrap(), &parsed(&changed).unwrap())
            .unwrap()
            .len(),
        1
    );
    let original = value::<Choices, 0>();
    let mut changed = original.clone();
    let variants = changed["roots"][0]["revisions"][0]["nodes"][0]["shape"]["variants"]
        .as_array_mut()
        .unwrap();
    variants[0]["tag"] = json!(1);
    variants[1]["tag"] = json!(0);
    assert_eq!(
        check(&parsed(&original).unwrap(), &parsed(&changed).unwrap())
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn list_ranges_are_checked_and_validated() {
    let original = value::<alkahest::Array<u8, 4>, 0>();
    let mut changed = original.clone();
    changed["roots"][0]["revisions"][0]["nodes"][0]["shape"]["min"] = json!(3);
    assert_eq!(
        check(&parsed(&original).unwrap(), &parsed(&changed).unwrap())
            .unwrap()
            .len(),
        1
    );
    changed["roots"][0]["revisions"][0]["nodes"][0]["shape"]["min"] = json!(5);
    assert!(parsed(&changed).is_err());
}

#[test]
fn total_node_budget_prevents_expansion_across_roots() {
    type Wide = (u8, u8, u8, u8, u8, u8, u8, u8);
    type Wider = (Wide, Wide, Wide, Wide, Wide, Wide, Wide, Wide);
    let mut snapshot = Snapshot::new();
    let mut rejected = false;
    for index in 0..256 {
        match snapshot.add::<Wider, 0>(format!("root-{index:03}")) {
            Ok(()) => {}
            Err(error) => {
                assert!(error.to_string().contains("node budget"), "{error}");
                rejected = true;
                break;
            }
        }
    }
    assert!(rejected);
    assert!(snapshot.to_json().is_ok());
}

#[test]
fn flat_encoding_supports_full_depth_limit_and_rejects_deeper_trees() {
    let mut value = value::<u8, 0>();
    let leaf = value["roots"][0]["revisions"][0]["nodes"][0].clone();
    for count in [64, 65] {
        let nodes = (0..count)
            .map(|index| {
                let mut node = leaf.clone();
                if index + 1 < count {
                    node["shape"] =
                        json!({"kind":"option", "element":{"node":index+1,"indirect":true}});
                }
                node
            })
            .collect::<Vec<_>>();
        value["roots"][0]["revisions"][0]["nodes"] = json!(nodes);
        assert_eq!(parsed(&value).is_ok(), count == 64);
    }
}

#[test]
fn malformed_enum_ranges_and_duplicate_revision_entries_are_rejected() {
    let mut value = value::<Choices, 0>();
    value["roots"][0]["revisions"][0]["nodes"][0]["shape"]["discriminant_count"] =
        json!(u32::MAX as u64 + 1);
    assert!(parsed(&value).is_err());
    let mut value = self::value::<Choices, 0>();
    value["roots"][0]["revisions"][0]["nodes"][0]["shape"]["variants"][1]["tag"] = json!(0);
    assert!(parsed(&value).is_err());
    let mut value = self::value::<Original, 0>();
    let revision = value["roots"][0]["revisions"][0].clone();
    value["roots"][0]["revisions"]
        .as_array_mut()
        .unwrap()
        .push(revision);
    assert!(parsed(&value).is_err());
}
