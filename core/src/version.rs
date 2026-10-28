//! Static formula descriptions and conservative wire compatibility checks.
//!
//! Descriptions are declarations by the formula implementor. They cannot verify
//! arbitrary handwritten serialization or deserialization implementations.

use crate::{Element, Formula, SizeBound, SizeType};

/// The layout used by a formula for a particular address/length width.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layout {
    pub stack: SizeBound,
    pub heap: SizeBound,
    pub inhabited: bool,
}

/// Primitive encodings. Numeric widths are in bytes; numbers are little endian.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Primitive {
    Bool,
    Unsigned(u8),
    Signed(u8),
    Float(u8),
}

/// A formula selected at one explicit version.
#[derive(Clone, Copy, Debug)]
pub struct Schema {
    pub version: u32,
    pub kind: SchemaKind,
    /// Returns the actual formula layout for widths in `1..=16`.
    pub layout: fn(usize) -> Option<Layout>,
}

impl Schema {
    /// Describe a formula using its actual size types.
    pub const fn new<F: Formula<VERSION> + ?Sized, const VERSION: u32>(kind: SchemaKind) -> Self {
        Self {
            version: VERSION,
            kind,
            layout: layout::<F, VERSION>,
        }
    }

    /// Opaque metadata is never accepted as proof of compatibility.
    pub const fn unknown<F: Formula<VERSION> + ?Sized, const VERSION: u32>() -> Self {
        Self::new::<F, VERSION>(SchemaKind::Unknown)
    }
}

/// A child formula and its placement in its parent.
///
/// The callback permits recursive schemas without constructing an infinite
/// constant. Its result includes the child's selected version.
#[derive(Clone, Copy, Debug)]
pub struct ElementSchema {
    pub schema: fn() -> &'static Schema,
    pub indirect: bool,
}

/// A record field. Names are diagnostic; field order defines the wire order.
#[derive(Clone, Copy, Debug)]
pub struct Field {
    pub name: &'static str,
    pub element: ElementSchema,
}

/// An enum variant and its actual encoded discriminant.
/// `usize::MAX` marks an uninhabited variant, which has no wire representation.
#[derive(Clone, Copy, Debug)]
pub struct Variant {
    pub name: &'static str,
    pub tag: usize,
    pub fields: &'static [Field],
}

/// The wire encoding and value domain declared by a formula.
#[derive(Clone, Copy, Debug)]
pub enum SchemaKind {
    Unknown,
    Never,
    Primitive(Primitive),
    Record(&'static [Field]),
    Enum {
        variants: &'static [Variant],
        /// The count passed to the discriminant codec, not a byte width.
        discriminant_count: usize,
    },
    List {
        element: ElementSchema,
        min: usize,
        max: usize,
    },
    Option(ElementSchema),
    Str,
}

/// Resolve a formula at a particular version without inspecting a value.
pub fn schema<F: Formula<VERSION> + ?Sized, const VERSION: u32>() -> &'static Schema {
    &F::SCHEMA
}

/// Opaque metadata for handwritten elements without an encoding declaration.
#[doc(hidden)]
pub fn unknown_schema<F: Formula<VERSION> + ?Sized, const VERSION: u32>() -> &'static Schema {
    &const { Schema::unknown::<F, VERSION>() }
}

/// Resolve a composite element, including its indirect placement.
pub const fn element_schema<E: Element<VERSION> + ?Sized, const VERSION: u32>() -> ElementSchema {
    E::SCHEMA
}

/// Read layout from the formula's size types for the selected address width.
pub fn layout<F: Formula<VERSION> + ?Sized, const VERSION: u32>(
    size_bytes: usize,
) -> Option<Layout> {
    with_size_bytes!(SIZE_BYTES = size_bytes => {
        Some(Layout {
            stack: <F::StackSize<SIZE_BYTES> as SizeType>::VALUE,
            heap: <F::HeapSize<SIZE_BYTES> as SizeType>::VALUE,
            inhabited: F::INHABITED,
        })
    } else { None })
}

/// Required compatibility relative to an old and a new schema.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Compatibility {
    /// New readers accept values written using the old schema.
    Backward,
    /// Old readers accept values written using the new schema.
    Forward,
    Both,
}

/// A reason compatibility could not be established.
///
/// Failure is conservative: it does not prove that every value is incompatible.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompatibilityError {
    UnsupportedSizeBytes(usize),
    UnknownSchema,
    LayoutMismatch,
    EncodingMismatch,
    FieldCountMismatch,
    ElementPlacementMismatch,
    MissingVariant(usize),
    LengthRangeMismatch,
    InvalidSchema,
    RecursionLimit,
}

impl core::fmt::Display for CompatibilityError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnsupportedSizeBytes(width) => {
                write!(f, "unsupported address/length width: {width}")
            }
            Self::UnknownSchema => f.write_str("formula metadata is opaque"),
            Self::LayoutMismatch => f.write_str("formula layouts differ"),
            Self::EncodingMismatch => f.write_str("wire encodings differ"),
            Self::FieldCountMismatch => f.write_str("record field counts differ"),
            Self::ElementPlacementMismatch => f.write_str("element placements differ"),
            Self::MissingVariant(tag) => write!(f, "reader does not accept variant tag {tag}"),
            Self::LengthRangeMismatch => f.write_str("reader does not accept writer length range"),
            Self::InvalidSchema => f.write_str("formula metadata is inconsistent"),
            Self::RecursionLimit => f.write_str("schema verification depth exceeds 64"),
        }
    }
}

impl core::error::Error for CompatibilityError {}

/// Verify that a reader accepts the writer's declared wire format and domain.
///
/// Both ends must use the supplied address/length width. Layout bounds and
/// their exact/bounded/unbounded classification must match at every node.
/// This intentionally rejects some usable schema changes rather than claiming
/// compatibility from size alone. Names and version numbers do not affect the
/// wire comparison. Opaque nodes and recursion deeper than 64 are unverified.
pub fn verify_compatibility(
    reader: &Schema,
    writer: &Schema,
    size_bytes: usize,
) -> Result<(), CompatibilityError> {
    if !(1..=16).contains(&size_bytes) {
        return Err(CompatibilityError::UnsupportedSizeBytes(size_bytes));
    }
    verify(reader, writer, size_bytes, 0)
}

/// Verify that the new schema reads the old schema.
pub fn verify_backward_compatibility(
    new: &Schema,
    old: &Schema,
    size_bytes: usize,
) -> Result<(), CompatibilityError> {
    verify_compatibility(new, old, size_bytes)
}

/// Verify that the old schema reads the new schema.
pub fn verify_forward_compatibility(
    new: &Schema,
    old: &Schema,
    size_bytes: usize,
) -> Result<(), CompatibilityError> {
    verify_compatibility(old, new, size_bytes)
}

/// Enforce the requested directions. `new` and `old` are explicit to avoid
/// inferring chronology from version numbers or formula type identities.
pub fn require_compatibility(
    new: &Schema,
    old: &Schema,
    requirement: Compatibility,
    size_bytes: usize,
) -> Result<(), CompatibilityError> {
    match requirement {
        Compatibility::Backward => verify_backward_compatibility(new, old, size_bytes),
        Compatibility::Forward => verify_forward_compatibility(new, old, size_bytes),
        Compatibility::Both => {
            verify_backward_compatibility(new, old, size_bytes)?;
            verify_forward_compatibility(new, old, size_bytes)
        }
    }
}

fn verify(
    reader: &Schema,
    writer: &Schema,
    width: usize,
    depth: usize,
) -> Result<(), CompatibilityError> {
    use CompatibilityError as Error;
    use SchemaKind as Kind;
    if depth >= 64 {
        return Err(Error::RecursionLimit);
    }
    // Never use pointer identity to bypass opaque metadata or recursive checks.
    if matches!(reader.kind, Kind::Unknown) || matches!(writer.kind, Kind::Unknown) {
        return Err(Error::UnknownSchema);
    }
    let reader_layout = (reader.layout)(width).ok_or(Error::UnsupportedSizeBytes(width))?;
    let writer_layout = (writer.layout)(width).ok_or(Error::UnsupportedSizeBytes(width))?;
    if reader_layout != writer_layout {
        return Err(Error::LayoutMismatch);
    }
    match (reader.kind, writer.kind) {
        (Kind::Never, Kind::Never) | (Kind::Str, Kind::Str) => Ok(()),
        (Kind::Primitive(r), Kind::Primitive(w)) if r == w => Ok(()),
        (Kind::Record(r), Kind::Record(w)) => fields(r, w, width, depth),
        (Kind::Option(r), Kind::Option(w)) => element(r, w, width, depth),
        (
            Kind::List {
                element: r,
                min: rmin,
                max: rmax,
            },
            Kind::List {
                element: w,
                min: wmin,
                max: wmax,
            },
        ) => {
            if rmin > rmax || wmin > wmax {
                return Err(Error::InvalidSchema);
            }
            // Fixed arrays omit the length prefix; variable lists include it.
            if (rmin == rmax) != (wmin == wmax) {
                return Err(Error::EncodingMismatch);
            }
            if wmin < rmin || wmax > rmax {
                return Err(Error::LengthRangeMismatch);
            }
            element(r, w, width, depth)
        }
        (
            Kind::Enum {
                variants: r,
                discriminant_count: rc,
            },
            Kind::Enum {
                variants: w,
                discriminant_count: wc,
            },
        ) => {
            validate_variants(r, rc)?;
            validate_variants(w, wc)?;
            if discriminant_width(rc) != discriminant_width(wc) {
                return Err(Error::EncodingMismatch);
            }
            for variant in w {
                if variant.tag == usize::MAX {
                    continue;
                }
                let accepted = r
                    .iter()
                    .find(|candidate| candidate.tag == variant.tag)
                    .ok_or(Error::MissingVariant(variant.tag))?;
                fields(accepted.fields, variant.fields, width, depth)?;
            }
            Ok(())
        }
        _ => Err(Error::EncodingMismatch),
    }
}

fn element(
    reader: ElementSchema,
    writer: ElementSchema,
    width: usize,
    depth: usize,
) -> Result<(), CompatibilityError> {
    if reader.indirect != writer.indirect {
        return Err(CompatibilityError::ElementPlacementMismatch);
    }
    verify((reader.schema)(), (writer.schema)(), width, depth + 1)
}

fn fields(
    reader: &[Field],
    writer: &[Field],
    width: usize,
    depth: usize,
) -> Result<(), CompatibilityError> {
    if reader.len() != writer.len() {
        return Err(CompatibilityError::FieldCountMismatch);
    }
    for (reader, writer) in reader.iter().zip(writer) {
        element(reader.element, writer.element, width, depth)?;
    }
    Ok(())
}

fn discriminant_width(count: usize) -> Option<usize> {
    match count {
        0..=0xff => Some(1),
        0x100..=0xffff => Some(2),
        0x10000..=0xffffff => Some(3),
        0x1000000..=0xffffffff => Some(4),
        _ => None,
    }
}

fn validate_variants(variants: &[Variant], count: usize) -> Result<(), CompatibilityError> {
    if discriminant_width(count).is_none() {
        return Err(CompatibilityError::InvalidSchema);
    }
    for (index, variant) in variants.iter().enumerate() {
        if variant.tag == usize::MAX {
            continue;
        }
        if variant.tag >= count
            || variants[..index]
                .iter()
                .any(|other| other.tag == variant.tag)
        {
            return Err(CompatibilityError::InvalidSchema);
        }
    }
    Ok(())
}
