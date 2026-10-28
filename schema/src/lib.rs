//! Persist formula declarations and detect changes to published versions.
//!
//! Register each independently versioned formula under a stable application ID.
//! A parent's snapshot contains every child revision, so changing a child pin
//! requires a parent bump. Registering the child separately also protects its
//! own revision when the parent version increases. Snapshots verify declared
//! metadata, not arbitrary handwritten codec implementations.

use std::{collections::BTreeSet, fmt};

use alkahest_core::{Formula, Schema, SchemaKind, SizeBound};
use serde::{Deserialize, Serialize};

const FORMAT: u32 = 1;
const MAX_ROOTS: usize = 256;
const MAX_REVISIONS: usize = 256;
const MAX_NODES: usize = 4_096;
const MAX_DEPTH: usize = 64;
/// Maximum encoded snapshot size accepted or emitted by this format (32 MiB).
pub const MAX_JSON_BYTES: usize = 32 * 1024 * 1024;

/// An invalid declaration, unsupported snapshot, or malformed input.
#[derive(Debug)]
pub struct Error(String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Error {}
impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self(value.to_string())
    }
}
fn invalid(message: impl Into<String>) -> Error {
    Error(message.into())
}

/// A stable, canonical catalog. Obtain JSON using [`Self::to_json`].
///
/// Each revision stores an expanded tree in a flat preorder node array. Child
/// indices are structural positions, never addresses or Rust type identities.
/// Limits: 256 roots, 256 revisions per root, 4,096 total nodes and enum
/// variants, depth 64,
/// and 32 MiB of JSON. Recursive and opaque declarations fail closed.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    format: u32,
    pointer_width: u8,
    roots: Vec<Root>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Root {
    id: String,
    current: u32,
    revisions: Vec<Revision>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Revision {
    version: u32,
    nodes: Vec<Node>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Node {
    version: u32,
    layouts: Vec<Layout>,
    shape: Shape,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Layout {
    width: u8,
    stack: Bound,
    heap: Bound,
    inhabited: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "size",
    rename_all = "snake_case",
    deny_unknown_fields
)]
enum Bound {
    Unbounded,
    Bounded(u64),
    Exact(u64),
}
impl From<SizeBound> for Bound {
    fn from(bound: SizeBound) -> Self {
        match bound {
            SizeBound::Unbounded => Self::Unbounded,
            SizeBound::Bounded(n) => Self::Bounded(n as u64),
            SizeBound::Exact(n) => Self::Exact(n as u64),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Shape {
    Never,
    Bool,
    Unsigned {
        bytes: u8,
    },
    Signed {
        bytes: u8,
    },
    Float {
        bytes: u8,
    },
    Record {
        fields: Vec<Field>,
    },
    Enum {
        variants: Vec<Variant>,
        discriminant_count: u64,
    },
    List {
        element: Element,
        min: u64,
        max: u64,
    },
    Option {
        element: Element,
    },
    Str,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Element {
    node: usize,
    indirect: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Field {
    name: String,
    element: Element,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Variant {
    name: String,
    /// Null denotes an uninhabited variant, independent of host usize width.
    tag: Option<u64>,
    fields: Vec<Field>,
}

/// A versioning violation. Paths locate the first changed snapshot property.
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct Violation {
    pub id: String,
    pub path: String,
    pub message: String,
}

impl Default for Snapshot {
    fn default() -> Self {
        Self::new()
    }
}

impl Snapshot {
    /// Start a catalog for the current target. An empty catalog cannot be saved.
    pub fn new() -> Self {
        Self {
            format: FORMAT,
            pointer_width: usize::BITS as u8,
            roots: Vec::new(),
        }
    }

    /// Register a stable formula family ID, its selected revision, and every
    /// revision in `F::VERSIONS`. Errors leave this catalog unchanged.
    pub fn add<F: Formula<V> + ?Sized, const V: u32>(
        &mut self,
        id: impl Into<String>,
    ) -> Result<(), Error> {
        let id = id.into();
        if self.format != FORMAT || self.pointer_width != usize::BITS as u8 {
            return Err(invalid(
                "cannot add formulas to a snapshot from another format or target",
            ));
        }
        if !self.roots.is_empty() {
            self.validate()?;
        }
        validate_id(&id)?;
        if self.roots.len() >= MAX_ROOTS {
            return Err(invalid("root limit exceeded"));
        }
        if self.roots.iter().any(|root| root.id == id) {
            return Err(invalid(format!("duplicate formula ID: {id}")));
        }
        if F::SCHEMA.version != V {
            return Err(invalid(
                "local schema version does not match Formula version",
            ));
        }
        if F::VERSIONS.len() > MAX_REVISIONS {
            return Err(invalid("revision limit exceeded"));
        }
        let mut total = self.node_count();
        let mut seen = BTreeSet::new();
        let mut revisions = Vec::new();
        for resolve in F::VERSIONS {
            let declared = resolve();
            if !seen.insert(declared.version) {
                return Err(invalid("duplicate declared revision"));
            }
            let schema = if declared.version == V {
                &F::SCHEMA
            } else {
                declared
            };
            let mut nodes = Vec::new();
            export_node(schema, &mut nodes, 0, &mut total).map_err(|error| {
                invalid(format!(
                    "formula {id:?}, revision {}: {error}",
                    schema.version
                ))
            })?;
            revisions.push(Revision {
                version: schema.version,
                nodes,
            });
        }
        if !seen.contains(&V) {
            return Err(invalid("revision table is missing local version"));
        }
        revisions.sort_by_key(|revision| revision.version);
        let root = Root {
            id,
            current: V,
            revisions,
        };
        let mut checked_total = self.node_count();
        validate_root(&root, self.pointer_width, &mut checked_total)
            .map_err(|error| invalid(format!("formula {:?}: {error}", root.id)))?;
        self.roots.push(root);
        self.roots.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(())
    }

    /// Serialize canonical, readable JSON with a trailing newline.
    pub fn to_json(&self) -> Result<String, Error> {
        self.validate()?;
        let mut json = serde_json::to_string_pretty(self)?;
        json.push('\n');
        if json.len() > MAX_JSON_BYTES {
            return Err(invalid("snapshot JSON exceeds 32 MiB"));
        }
        Ok(json)
    }

    /// Parse and validate a snapshot. Unknown fields, duplicate IDs/revisions,
    /// incomplete layouts and noncanonical graphs are errors.
    pub fn from_json(json: &str) -> Result<Self, Error> {
        if json.len() > MAX_JSON_BYTES {
            return Err(invalid("snapshot JSON exceeds 32 MiB"));
        }
        let snapshot: Self = serde_json::from_str(json)?;
        snapshot.validate()?;
        Ok(snapshot)
    }

    fn node_count(&self) -> usize {
        self.roots
            .iter()
            .flat_map(|root| &root.revisions)
            .map(|revision| node_cost(&revision.nodes))
            .sum()
    }

    fn validate(&self) -> Result<(), Error> {
        if self.format != FORMAT {
            return Err(invalid("unsupported snapshot format"));
        }
        if ![16, 32, 64].contains(&self.pointer_width) {
            return Err(invalid("unsupported target pointer width"));
        }
        if self.roots.is_empty() || self.roots.len() > MAX_ROOTS {
            return Err(invalid("catalog must contain 1..=256 roots"));
        }
        if self.roots.windows(2).any(|pair| pair[0].id >= pair[1].id) {
            return Err(invalid("root IDs must be unique and sorted"));
        }
        let mut total = 0;
        for root in &self.roots {
            validate_root(root, self.pointer_width, &mut total)?;
        }
        Ok(())
    }
}

fn validate_id(id: &str) -> Result<(), Error> {
    if id.trim().is_empty() || id.len() > 1024 || id.chars().any(char::is_control) {
        return Err(invalid(
            "formula IDs must be nonempty, at most 1024 bytes and contain no control characters",
        ));
    }
    Ok(())
}

fn export_node(
    schema: &Schema,
    nodes: &mut Vec<Node>,
    depth: usize,
    total: &mut usize,
) -> Result<usize, Error> {
    if depth >= MAX_DEPTH {
        return Err(invalid(
            "schema depth exceeds 64; recursive schemas are unsupported",
        ));
    }
    if *total >= MAX_NODES {
        return Err(invalid("snapshot node budget exceeded"));
    }
    *total += 1;
    let index = nodes.len();
    let layouts = (1..=16)
        .map(|width| {
            let layout = (schema.layout)(width)
                .ok_or_else(|| invalid(format!("missing layout for width {width}")))?;
            Ok(Layout {
                width: width as u8,
                stack: layout.stack.into(),
                heap: layout.heap.into(),
                inhabited: layout.inhabited,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    nodes.push(Node {
        version: schema.version,
        layouts,
        shape: Shape::Never,
    });
    let shape = match schema.kind {
        SchemaKind::Unknown => return Err(invalid("opaque schema cannot be snapshotted")),
        SchemaKind::Never => Shape::Never,
        SchemaKind::Primitive(primitive) => match primitive {
            alkahest_core::Primitive::Bool => Shape::Bool,
            alkahest_core::Primitive::Unsigned(bytes) => Shape::Unsigned { bytes },
            alkahest_core::Primitive::Signed(bytes) => Shape::Signed { bytes },
            alkahest_core::Primitive::Float(bytes) => Shape::Float { bytes },
        },
        SchemaKind::Record(fields) => Shape::Record {
            fields: export_fields(fields, nodes, depth, total)?,
        },
        SchemaKind::Enum {
            variants,
            discriminant_count,
        } => {
            if variants.len() > MAX_NODES.saturating_sub(*total) {
                return Err(invalid("snapshot node budget exceeded"));
            }
            *total += variants.len();
            let variants = variants
                .iter()
                .map(|variant| {
                    Ok(Variant {
                        name: variant.name.into(),
                        tag: (variant.tag != usize::MAX).then_some(variant.tag as u64),
                        fields: export_fields(variant.fields, nodes, depth, total)?,
                    })
                })
                .collect::<Result<Vec<_>, Error>>()?;
            Shape::Enum {
                variants,
                discriminant_count: discriminant_count as u64,
            }
        }
        SchemaKind::List { element, min, max } => Shape::List {
            element: export_element(element, nodes, depth, total)?,
            min: min as u64,
            max: max as u64,
        },
        SchemaKind::Option(element) => Shape::Option {
            element: export_element(element, nodes, depth, total)?,
        },
        SchemaKind::Str => Shape::Str,
    };
    nodes[index].shape = shape;
    Ok(index)
}

fn export_fields(
    fields: &[alkahest_core::Field],
    nodes: &mut Vec<Node>,
    depth: usize,
    total: &mut usize,
) -> Result<Vec<Field>, Error> {
    if fields.len() > MAX_NODES.saturating_sub(*total) {
        return Err(invalid("snapshot node budget exceeded"));
    }
    fields
        .iter()
        .map(|field| {
            Ok(Field {
                name: field.name.into(),
                element: export_element(field.element, nodes, depth, total)?,
            })
        })
        .collect()
}

fn export_element(
    element: alkahest_core::ElementSchema,
    nodes: &mut Vec<Node>,
    depth: usize,
    total: &mut usize,
) -> Result<Element, Error> {
    Ok(Element {
        node: export_node((element.schema)(), nodes, depth + 1, total)?,
        indirect: element.indirect,
    })
}

fn validate_root(root: &Root, pointer_width: u8, total: &mut usize) -> Result<(), Error> {
    validate_id(&root.id)?;
    if root.revisions.is_empty() || root.revisions.len() > MAX_REVISIONS {
        return Err(invalid("root must contain 1..=256 revisions"));
    }
    if root
        .revisions
        .windows(2)
        .any(|pair| pair[0].version >= pair[1].version)
    {
        return Err(invalid("revisions must be unique and sorted"));
    }
    if !root
        .revisions
        .iter()
        .any(|revision| revision.version == root.current)
    {
        return Err(invalid("current revision is missing"));
    }
    let max = if pointer_width == 64 {
        u64::MAX
    } else {
        (1u64 << pointer_width) - 1
    };
    for revision in &root.revisions {
        if revision.nodes.is_empty() || revision.nodes[0].version != revision.version {
            return Err(invalid("revision root version mismatch"));
        }
        *total = total
            .checked_add(node_cost(&revision.nodes))
            .ok_or_else(|| invalid("node count overflow"))?;
        if *total > MAX_NODES {
            return Err(invalid("snapshot node budget exceeded"));
        }
        let mut next = 0;
        validate_node(&revision.nodes, 0, 0, &mut next, max)?;
        if next != revision.nodes.len() {
            return Err(invalid("unreachable snapshot nodes"));
        }
    }
    Ok(())
}

fn node_cost(nodes: &[Node]) -> usize {
    nodes.iter().fold(nodes.len(), |cost, node| {
        cost.saturating_add(match &node.shape {
            Shape::Enum { variants, .. } => variants.len(),
            _ => 0,
        })
    })
}

fn validate_node(
    nodes: &[Node],
    index: usize,
    depth: usize,
    next: &mut usize,
    max: u64,
) -> Result<(), Error> {
    if depth >= MAX_DEPTH {
        return Err(invalid("schema depth exceeds 64"));
    }
    if index != *next {
        return Err(invalid("child nodes must form a canonical preorder tree"));
    }
    let node = nodes
        .get(index)
        .ok_or_else(|| invalid("child node index is out of bounds"))?;
    *next += 1;
    if node.layouts.len() != 16 {
        return Err(invalid("every node needs layouts for widths 1..=16"));
    }
    let inhabited = node.layouts[0].inhabited;
    for (offset, layout) in node.layouts.iter().enumerate() {
        if layout.width as usize != offset + 1 || layout.inhabited != inhabited {
            return Err(invalid("invalid layout widths or inconsistent inhabitance"));
        }
        for bound in [&layout.stack, &layout.heap] {
            if matches!(bound, Bound::Exact(n) | Bound::Bounded(n) if *n > max) {
                return Err(invalid("size bound exceeds target usize"));
            }
        }
    }
    let mut child = |element: &Element| validate_node(nodes, element.node, depth + 1, next, max);
    match &node.shape {
        Shape::Unsigned { bytes } | Shape::Signed { bytes }
            if ![1, 2, 4, 8, 16].contains(bytes) =>
        {
            return Err(invalid("invalid integer byte width"));
        }
        Shape::Float { bytes } if ![4, 8].contains(bytes) => {
            return Err(invalid("invalid float byte width"));
        }
        Shape::Never if inhabited => return Err(invalid("never schema cannot be inhabited")),
        Shape::Record { fields } => validate_fields(fields, &mut child)?,
        Shape::Enum {
            variants,
            discriminant_count,
        } => {
            if *discriminant_count > max.min(u32::MAX as u64) || variants.len() > MAX_NODES {
                return Err(invalid("invalid enum discriminant count or variant count"));
            }
            let mut names = BTreeSet::new();
            let mut tags = BTreeSet::new();
            for variant in variants {
                if variant.name.is_empty() || !names.insert(&variant.name) {
                    return Err(invalid("variant names must be nonempty and unique"));
                }
                if let Some(tag) = variant.tag {
                    if tag >= *discriminant_count || !tags.insert(tag) {
                        return Err(invalid(
                            "enum tags must be distinct and below discriminant count",
                        ));
                    }
                }
                validate_fields(&variant.fields, &mut child)?;
            }
        }
        Shape::List {
            element,
            min,
            max: length_max,
        } => {
            if min > length_max || *length_max > max {
                return Err(invalid("invalid list length bounds"));
            }
            child(element)?;
        }
        Shape::Option { element } => child(element)?,
        _ => {}
    }
    Ok(())
}

fn validate_fields(
    fields: &[Field],
    child: &mut impl FnMut(&Element) -> Result<(), Error>,
) -> Result<(), Error> {
    if fields.len() > MAX_NODES {
        return Err(invalid("field limit exceeded"));
    }
    let mut names = BTreeSet::new();
    for field in fields {
        // Built-in tuples use empty names; their ordered positions identify fields.
        if !field.name.is_empty() && !names.insert(&field.name) {
            return Err(invalid("named fields must have unique names"));
        }
        child(&field.element)?;
    }
    Ok(())
}

/// Compare a trusted published baseline with a freshly generated catalog.
///
/// New IDs and increased current revisions are allowed. Removed IDs, decreasing
/// current versions, and changes to any revision present on both sides fail.
/// Historical revisions need not be retained when the current version advances.
/// Target pointer widths must match. Compatibility checks remain a separate API.
pub fn check(baseline: &Snapshot, current: &Snapshot) -> Result<Vec<Violation>, Error> {
    baseline.validate()?;
    current.validate()?;
    if baseline.pointer_width != current.pointer_width {
        return Err(invalid("snapshot target pointer widths differ"));
    }
    let mut violations = Vec::new();
    for old in &baseline.roots {
        let mut report = |path: String, message: String| {
            violations.push(Violation {
                id: old.id.clone(),
                path,
                message,
            })
        };
        let Some(new) = current.roots.iter().find(|root| root.id == old.id) else {
            report("root".into(), "registered formula was removed".into());
            continue;
        };
        if new.current < old.current {
            report(
                "current".into(),
                format!(
                    "current version decreased from {} to {}",
                    old.current, new.current
                ),
            );
        }
        for old_revision in &old.revisions {
            if let Some(new_revision) = new
                .revisions
                .iter()
                .find(|revision| revision.version == old_revision.version)
            {
                if old_revision != new_revision {
                    let old_json = serde_json::to_value(old_revision)?;
                    let new_json = serde_json::to_value(new_revision)?;
                    let path = difference(
                        &old_json,
                        &new_json,
                        format!("revisions[{}]", old_revision.version),
                    );
                    report(
                        path,
                        format!(
                            "formula revision {} changed without a version bump",
                            old_revision.version
                        ),
                    );
                }
            } else if new.current <= old.current {
                report(
                    format!("revisions[{}]", old_revision.version),
                    "declared revision was removed without advancing current version".into(),
                );
            }
        }
    }
    Ok(violations)
}

fn difference(old: &serde_json::Value, new: &serde_json::Value, path: String) -> String {
    match (old, new) {
        (serde_json::Value::Object(old), serde_json::Value::Object(new)) => {
            for (key, value) in old {
                let next = format!("{path}.{key}");
                match new.get(key) {
                    Some(other) if value != other => return difference(value, other, next),
                    None => return next,
                    _ => {}
                }
            }
            if let Some(key) = new.keys().find(|key| !old.contains_key(*key)) {
                return format!("{path}.{key}");
            }
        }
        (serde_json::Value::Array(old), serde_json::Value::Array(new)) => {
            for (index, (value, other)) in old.iter().zip(new).enumerate() {
                if value != other {
                    return difference(value, other, format!("{path}[{index}]"));
                }
            }
            if old.len() != new.len() {
                return format!("{path}.length");
            }
        }
        _ => {}
    }
    path
}
