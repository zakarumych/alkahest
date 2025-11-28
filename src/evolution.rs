//! Evolution in Alkahest is a process of changing struct and enum `Formula`s
//! in a way that permits deserializing of data serialized
//! with older and or newer versions of the `Formula`.
//!
//! When declaring formula evolution Alkahest can verify that
//! compatibility in desired direction is maintained.
//!
//! Note that Alkahest won't hold your hand and won't forbid changing older formula versions
//! which may easily break compatibility.
//!
//! Alkahest can use old `Formula` version kept in code to see how format evolved.
//! User still required to figure out which version of the formula has been used to serialize data.
//! Alternatively user can provide formula descriptor value that contains information required to map the data.
//! Formula descriptor can be itself serialized with Alkahest.
//!
//! To handle filds and variants removals and reordering, Alkahest uses ids.
//! Ids can be assigned automatically or manually.
//! Automatic ids are assigned using field's or variant's name stable hash.
//! For tuple structs ids must be assigned manually since they don't have names.
//!
//! Manual ids are assigned using `#[alkahest(id = <expr>)]` attribute.
//! `<expr>` must evaluate to `u32`, have no side effects and not change between versions.
//! For these reasons prefer to always use literal values.
//!
//! When field gets removed, add its id to the list of removed ids in the formula header.
//! If id was automatically assigned, use field's name.
//!
//! Alkahest detects id collisions and fails compilation with an error.
//! If automatic ids collide then manual ids must be used.
//!
//! ### Forward compatibility
//! is an ability to deserialize data serialized with newer version.
//!
//! Following forward compatibility rules apply:
//! - Field can be added, it would be ignored by older readers.
//!
//! - Field with default value can be removed, it would be filled with default value by older readers.
//!
//! - Field without default can be marked as `#[deprecated]`.
//!   Default value must be provided for deprecated field.
//!   It will be stored in the formula descriptor to pick up by older readers.
//!
//! - Variants can be removed, older readers will simply never find them in new data.
//!   Formula header should contain ids of removed variants.
//!
//! - Arrays can be extended, older readers will ignore extra elements.
//!
//! - Slice can be replaced by an array, older readers will deserialize array as slice.
//!
//! ### Backward compatibility
//! is an ability to deserialize data serialized with older version.
//!
//! Following backward compatibility rules apply:
//! - Fields can be removed, they would be ignored by newer readers.
//!
//! - Fields can be added with default values provided.
//!   When deserialize older data, default values will be used.
//!
//! - Variants can be added, newer reader will simply never find them in old data.
//!
//! - Variants cannot be removed, but can be marked #[deprecated].
//!
//! - Arrays can be shortened, extra elements will be ignored.
//!
//! - Array
//!
//! ### Sliding window of compatibility
//! is compatibility with older or newer versions up to a certain version.
//!
//! For example, when field is `#[deprecated]` it may be desirable to stop supporting readers
//! with versions before the field was deprecated. And then field can be removed completely.
//! Versions where field is deprecated ignores it anyway.
//!
//! Example:
//!
//! Consider first version of the formula:
//!
//! ```
//! #[formula]
//! struct MyFormula {
//!   a: u8,
//!   b: u8,
//! }
//! ```
//!
//! In second version `b` is deprecated:
//!
//! ```
//! #[formula]
//! struct MyFormula {
//!   a: u8,
//!
//!   #[deprecated]
//!   #[alkahest(default)]
//!   b: u8,
//! }
//! ```
//!
//! Data written with second version can be deserialized with first version.
//!
//! In third version `b` is removed.
//! Its id is kept in the formula header for id collision checks.
//!
//! ```
//! #[formula]
//! #[alkahest(removed = [b])]
//! struct MyFormula {
//!   a: u8,
//! }
//! ```
//!
//! Data serialized with third version can be deserialized with second version,
//! but not first version.
//!
//! Similarly it works for backward compatibility.
//!
//! When field is added with `#[alkahest(default)]` it may be desirable to stop supporting data
//! serialized with versions before the field was added.
//! Then `#[alkahest(default)]` attribute can be removed
//!
//! This approach allows to evolve formulas and eventually drop the legacy.
//! Careful consideration is required to decide which version support to drop.
//!

use core::{
    any::TypeId,
    hash::{BuildHasher, Hasher},
    mem::size_of,
};

use alloc::vec::Vec;
use hashbrown::{
    hash_map::{Entry, VacantEntry},
    HashMap,
};

use crate::{
    advanced::{reference_size, BareFormula, Buffer},
    r#as::As,
    reference::Ref,
    serialize::{SerializeRef, Sizes},
    size::SIZE_STACK,
    Formula,
};

#[derive(Clone, Copy, PartialEq, Eq)]
struct Field {
    id: u32,
    formula: Option<u32>,
}

struct FieldFormula;

impl Formula for FieldFormula {
    const MAX_STACK_SIZE: Option<usize> = Some(size_of::<u32>() * 2 + 1);
    const EXACT_SIZE: bool = false;
    const HEAPLESS: bool = false;
}

impl BareFormula for FieldFormula {}

impl SerializeRef<FieldFormula> for Field {
    #[inline(always)]
    fn serialize<B>(&self, sizes: &mut Sizes, mut buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        buffer.write_stack(sizes.heap, sizes.stack, &self.id.to_le_bytes())?;
        sizes.stack += size_of::<u32>();

        match self.formula {
            None => {
                buffer.write_stack(sizes.heap, sizes.stack, &[0u8])?;
                sizes.stack += 1;
            }
            Some(formula) => {
                buffer.write_stack(sizes.heap, sizes.stack, &[1u8])?;
                sizes.stack += 1;
                buffer.write_stack(sizes.heap, sizes.stack, &formula.to_le_bytes())?;
                sizes.stack += size_of::<u32>();
            }
        }

        Ok(())
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        match self.formula {
            None => Some(Sizes::with_stack(size_of::<u32>() + 1)),
            Some(_) => Some(Sizes::with_stack(size_of::<u32>() * 2 + 1)),
        }
    }
}

struct Variant {
    id: u32,
    fields: Vec<Field>,
}

struct VariantFormula;

impl Formula for VariantFormula {
    const MAX_STACK_SIZE: Option<usize> =
        Some(size_of::<u32>() + reference_size::<[FieldFormula]>());
    const EXACT_SIZE: bool = true;
    const HEAPLESS: bool = false;
}

impl BareFormula for VariantFormula {}

impl SerializeRef<VariantFormula> for Variant {
    #[inline(always)]
    fn serialize<B>(&self, sizes: &mut Sizes, mut buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        buffer.write_stack(sizes.heap, sizes.stack, &self.id.to_le_bytes())?;
        SerializeRef::<Vec<FieldFormula>>::serialize(&*self.fields, sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        let mut sizes = SerializeRef::<Vec<FieldFormula>>::size_hint(&*self.fields)?;
        sizes.add_stack(size_of::<u32>());
        Some(sizes)
    }
}

#[doc(hidden)]
pub enum Flavor {
    /// Reference formula.
    Ref(Option<u32>),

    /// Sequence formula.
    Sequence { elem: Option<u32>, len: Option<u32> },

    /// Map formula.
    Map {
        key: Option<u32>,
        value: Option<u32>,
    },

    /// Evolving struct formula.
    Record(Vec<Field>),

    /// Evolving enum formula.
    Enum(Vec<Variant>),
}

struct KindFormula;

impl Formula for KindFormula {
    const MAX_STACK_SIZE: Option<usize> = Some(1 + SIZE_STACK * 2);
    const EXACT_SIZE: bool = false;
    const HEAPLESS: bool = false;
}

impl BareFormula for KindFormula {}

impl SerializeRef<KindFormula> for Flavor {
    #[inline]
    fn serialize<B>(&self, sizes: &mut Sizes, mut buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        match *self {
            Flavor::Ref(None) => {
                buffer.write_stack(sizes.heap, sizes.stack, &[0x00u8])?;
                sizes.add_stack(1);
            }
            Flavor::Ref(Some(formula)) => {
                buffer.write_stack(sizes.heap, sizes.stack, &[0x01u8])?;
                sizes.add_stack(1);
                buffer.write_stack(sizes.heap, sizes.stack, &formula.to_le_bytes())?;
                sizes.add_stack(size_of::<u32>());
            }
            Flavor::Sequence {
                elem: None,
                len: None,
            } => {
                buffer.write_stack(sizes.heap, sizes.stack, &[0x10u8])?;
                sizes.add_stack(1);
            }
            Flavor::Sequence {
                elem: Some(elem),
                len: None,
            } => {
                buffer.write_stack(sizes.heap, sizes.stack, &[0x11u8])?;
                sizes.add_stack(1);
                buffer.write_stack(sizes.heap, sizes.stack, &elem.to_le_bytes())?;
                sizes.add_stack(size_of::<u32>());
            }
            Flavor::Sequence {
                elem: None,
                len: Some(len),
            } => {
                buffer.write_stack(sizes.heap, sizes.stack, &[0x12u8])?;
                sizes.add_stack(1);
                buffer.write_stack(sizes.heap, sizes.stack, &len.to_le_bytes())?;
                sizes.add_stack(size_of::<u32>());
            }
            Flavor::Sequence {
                elem: Some(elem),
                len: Some(len),
            } => {
                buffer.write_stack(sizes.heap, sizes.stack, &[0x13u8])?;
                sizes.add_stack(1);
                buffer.write_stack(sizes.heap, sizes.stack, &elem.to_le_bytes())?;
                sizes.add_stack(size_of::<u32>());
                buffer.write_stack(sizes.heap, sizes.stack, &len.to_le_bytes())?;
                sizes.add_stack(size_of::<u32>());
            }
            Flavor::Map {
                key: None,
                value: None,
            } => {
                buffer.write_stack(sizes.heap, sizes.stack, &[0x20u8])?;
                sizes.add_stack(1);
            }
            Flavor::Map {
                key: None,
                value: Some(value),
            } => {
                buffer.write_stack(sizes.heap, sizes.stack, &[0x21u8])?;
                sizes.add_stack(1);
                buffer.write_stack(sizes.heap, sizes.stack, &value.to_le_bytes())?;
                sizes.add_stack(size_of::<u32>());
            }
            Flavor::Map {
                key: Some(key),
                value: None,
            } => {
                buffer.write_stack(sizes.heap, sizes.stack, &[0x22u8])?;
                sizes.add_stack(1);
                buffer.write_stack(sizes.heap, sizes.stack, &key.to_le_bytes())?;
                sizes.add_stack(size_of::<u32>());
            }
            Flavor::Map {
                key: Some(key),
                value: Some(value),
            } => {
                buffer.write_stack(sizes.heap, sizes.stack, &[0x23u8])?;
                sizes.add_stack(1);
                buffer.write_stack(sizes.heap, sizes.stack, &key.to_le_bytes())?;
                sizes.add_stack(size_of::<u32>());
                buffer.write_stack(sizes.heap, sizes.stack, &value.to_le_bytes())?;
                sizes.add_stack(size_of::<u32>());
            }
            Flavor::Record(ref fields) => {
                buffer.write_stack(sizes.heap, sizes.stack, &[0x70u8])?;
                sizes.add_stack(1);
                SerializeRef::<Vec<FieldFormula>>::serialize(&**fields, sizes, buffer)?;
            }
            Flavor::Enum(ref variants) => {
                buffer.write_stack(sizes.heap, sizes.stack, &[0xc0u8])?;
                sizes.add_stack(1);
                SerializeRef::<Vec<VariantFormula>>::serialize(&**variants, sizes, buffer)?;
            }
        }

        Ok(())
    }

    #[inline]
    fn size_hint(&self) -> Option<Sizes> {
        let mut sizes = Sizes::with_stack(1);
        match self {
            Flavor::Ref(f) => sizes.add_stack(1 + f.map_or(0, |_| size_of::<u32>())),
            Flavor::Sequence { elem, len } => sizes.add_stack(
                1 + elem.map_or(0, |_| size_of::<u32>()) + len.map_or(0, |_| size_of::<u32>()),
            ),
            Flavor::Map { key, value } => sizes.add_stack(
                1 + key.map_or(0, |_| size_of::<u32>()) + value.map_or(0, |_| size_of::<u32>()),
            ),
            Flavor::Record(ref fields) => {
                sizes += SerializeRef::<Vec<FieldFormula>>::size_hint(&**fields)?
            }
            Flavor::Enum(ref variants) => {
                sizes += SerializeRef::<Vec<VariantFormula>>::size_hint(&**variants)?
            }
        }

        Some(sizes)
    }
}

/// Descriptor of the formula
/// that can be used to deserialize data with different version of the formula.
pub struct Descriptor {
    formulas: Vec<Flavor>,
}

impl Descriptor {
    /// Creates descriptor of primitve formula.
    #[inline(always)]
    pub fn primitve() -> Self {
        Descriptor {
            formulas: Vec::new(),
        }
    }

    /// Returns `true` if formula is primitive.
    pub fn is_primitive(&self) -> bool {
        self.formulas.is_empty()
    }

    /// Returns root formula.
    pub fn root(&self) -> &Flavor {
        self.formulas
            .first()
            .expect("Primitive formula has no root")
    }

    /// Returns formula by index.
    pub fn get(&self, idx: u32) -> &Flavor {
        self.formulas
            .get(idx as usize)
            .expect("Invalid formula index")
    }

    /// Creates descriptor of formula.
    pub fn new<F>() -> Self
    where
        F: Formula,
    {
        let mut map = HashMap::with_hasher(NoopHasherBuilder);
        let mut formulas = Vec::new();
        let mut idx = None;

        <F as Formula>::descriptor(DescriptorBuilder {
            map: &mut map,
            array: &mut formulas,
            idx: &mut idx,
        });

        Descriptor { formulas }
    }
}

/// Formula for `Descriptor` type.
pub struct DescriptorFormula;

impl Formula for DescriptorFormula {
    const MAX_STACK_SIZE: Option<usize> = Some(reference_size::<[KindFormula]>());
    const EXACT_SIZE: bool = true;
    const HEAPLESS: bool = false;
}

impl BareFormula for DescriptorFormula {}

impl SerializeRef<DescriptorFormula> for Descriptor {
    #[inline(always)]
    fn serialize<B>(&self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        SerializeRef::<Vec<KindFormula>>::serialize(&*self.formulas, sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        SerializeRef::<Vec<KindFormula>>::size_hint(&*self.formulas)
    }
}

/// Context used to build descriptors for formulas.
pub struct DescriptorBuilder<'a> {
    map: &'a mut HashMap<TypeId, Option<u32>, NoopHasherBuilder>,
    array: &'a mut Vec<Flavor>,
    idx: &'a mut Option<u32>,
}

impl DescriptorBuilder<'_> {
    /// Set to reference flavor.
    pub fn reference<F>(self)
    where
        F: Formula + ?Sized,
    {
        let idx = match self.map.entry(TypeId::of::<Ref<F>>()) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let idx = u32::try_from(self.array.len()).expect("Too large formula");
                self.array.push(Flavor::Ref(None));
                entry.insert(Some(idx));

                let formula = match self.map.get(&TypeId::of::<F>()) {
                    None => {
                        let mut formula = None;

                        <F as Formula>::descriptor(DescriptorBuilder {
                            map: &mut *self.map,
                            array: &mut *self.array,
                            idx: &mut formula,
                        });

                        self.map.insert(TypeId::of::<F>(), formula);

                        formula
                    }
                    Some(formula) => *formula,
                };

                self.array[idx as usize] = Flavor::Ref(formula);

                Some(idx)
            }
        };

        *self.idx = idx;
    }

    /// Set to sequence flavor.
    pub fn sequence<F>(self, len: Option<u32>)
    where
        F: Formula + ?Sized,
    {
        struct Seq<F: ?Sized>(As<F>);

        let idx = match self.map.entry(TypeId::of::<Seq<F>>()) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let idx = u32::try_from(self.array.len()).expect("Too large formula");
                self.array.push(Flavor::Sequence { elem: None, len });
                entry.insert(Some(idx));

                let elem = match self.map.get(&TypeId::of::<F>()) {
                    None => {
                        let mut elem = None;

                        <F as Formula>::descriptor(DescriptorBuilder {
                            map: &mut *self.map,
                            array: &mut *self.array,
                            idx: &mut elem,
                        });

                        self.map.insert(TypeId::of::<F>(), elem);

                        elem
                    }
                    Some(elem) => *elem,
                };

                self.array[idx as usize] = Flavor::Sequence { elem, len };

                Some(idx)
            }
        };

        *self.idx = idx;
    }

    /// Set to map flavor.
    pub fn map<K, V>(self)
    where
        K: Formula + ?Sized,
        V: Formula + ?Sized,
    {
        struct Map<K: ?Sized, V: ?Sized>(As<K>, As<V>);

        let idx = match self.map.entry(TypeId::of::<Map<K, V>>()) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let idx = u32::try_from(self.array.len()).expect("Too large formula");
                self.array.push(Flavor::Map {
                    key: None,
                    value: None,
                });
                entry.insert(Some(idx));

                let key = match self.map.get(&TypeId::of::<K>()) {
                    None => {
                        let mut key = None;

                        <K as Formula>::descriptor(DescriptorBuilder {
                            map: &mut *self.map,
                            array: &mut *self.array,
                            idx: &mut key,
                        });

                        self.map.insert(TypeId::of::<K>(), key);

                        key
                    }
                    Some(key) => *key,
                };

                let value = match self.map.get(&TypeId::of::<V>()) {
                    None => {
                        let mut value = None;

                        <V as Formula>::descriptor(DescriptorBuilder {
                            map: &mut *self.map,
                            array: &mut *self.array,
                            idx: &mut value,
                        });

                        self.map.insert(TypeId::of::<V>(), value);

                        value
                    }
                    Some(value) => *value,
                };

                self.array[idx as usize] = Flavor::Map { key, value };

                Some(idx)
            }
        };

        *self.idx = idx;
    }

    /// Set to record flavor.
    ///
    /// Provided closure will enumerate fields of the record.
    pub fn record(self, f: impl FnOnce(RecordBuilder)) {
        let idx = u32::try_from(self.array.len()).expect("Too large formula");
        self.array.push(Flavor::Record(Vec::new()));

        let mut fields = Vec::new();

        f(RecordBuilder {
            map: &mut *self.map,
            formulas: &mut *self.array,
            fields: &mut fields,
        });

        self.array[idx as usize] = Flavor::Record(fields);
    }

    /// Set to enum flavor.
    ///
    /// Provided closure will enumerate variants of the enum.
    pub fn enumeration(self, f: impl FnOnce(EnumBuilder)) {
        let idx = u32::try_from(self.array.len()).expect("Too large formula");
        self.array.push(Flavor::Enum(Vec::new()));

        let mut variants = Vec::new();

        f(EnumBuilder {
            map: &mut *self.map,
            formulas: &mut *self.array,
            variants: &mut variants,
        });

        self.array[idx as usize] = Flavor::Enum(variants);
    }
}

/// Context used to build descriptors for enum formulas.
pub struct EnumBuilder<'a> {
    map: &'a mut HashMap<TypeId, Option<u32>, NoopHasherBuilder>,
    formulas: &'a mut Vec<Flavor>,
    variants: &'a mut Vec<Variant>,
}

impl EnumBuilder<'_> {
    /// Adds variant to the enum.
    ///
    /// Provided closure will enumerate fields of the variant.
    pub fn variant(&mut self, id: u32, f: impl FnOnce(RecordBuilder)) {
        let mut fields = Vec::new();

        f(RecordBuilder {
            map: &mut *self.map,
            formulas: &mut *self.formulas,
            fields: &mut fields,
        });

        self.variants.push(Variant {
            id,
            fields: Vec::new(),
        });
    }
}

/// Context used to build descriptors for record formulas.
pub struct RecordBuilder<'a> {
    map: &'a mut HashMap<TypeId, Option<u32>, NoopHasherBuilder>,
    formulas: &'a mut Vec<Flavor>,
    fields: &'a mut Vec<Field>,
}

impl RecordBuilder<'_> {
    /// Adds field to the record.
    pub fn field<F: Formula>(&mut self, id: u32) {
        let idx = match self.map.get(&TypeId::of::<F>()) {
            None => {
                let mut idx = None;

                <F as Formula>::descriptor(DescriptorBuilder {
                    map: &mut *self.map,
                    array: &mut *self.formulas,
                    idx: &mut idx,
                });

                self.map.insert(TypeId::of::<F>(), idx);

                idx
            }
            Some(idx) => *idx,
        };

        self.fields.push(Field { id, formula: idx });
    }
}

/// Cache of formula descriptors.
/// Used to avoid recomputation of formula descriptors.
pub struct DescriptorsCache {
    descriptors: HashMap<TypeId, Descriptor>,
}

impl DescriptorsCache {
    /// Creates new cache.
    pub fn new() -> Self {
        DescriptorsCache {
            descriptors: HashMap::new(),
        }
    }

    /// Returns descriptor of the formula.
    pub fn descriptor<F>(&mut self) -> &Descriptor
    where
        F: Formula,
    {
        #[cold]
        fn new_descriptor<F>(entry: VacantEntry<'_, TypeId, Descriptor>) -> &Descriptor
        where
            F: Formula,
        {
            let descriptor = Descriptor::new::<F>();
            &*entry.insert(descriptor)
        }

        match self.descriptors.entry(TypeId::of::<F>()) {
            Entry::Occupied(entry) => &*entry.into_mut(),
            Entry::Vacant(entry) => new_descriptor::<F>(entry),
        }
    }
}

#[cfg(feature = "evolution-global-cache")]
mod global_cache {
    use core::any::TypeId;

    use hashbrown::{hash_map::Entry, HashMap};

    use crate::formula::Formula;

    use super::{Descriptor, NoopHasherBuilder};

    /// Returns descriptor of the formula.
    pub fn descriptor<'a, F>() -> &'a Descriptor
    where
        F: Formula,
    {
        let cache = GLOBAL_CACHE.read();
        if let Some(&d) = cache.get(&TypeId::of::<F>()) {
            return d;
        }

        drop(cache);
        new_descriptor::<'a, F>()
    }

    #[cold]
    fn new_descriptor<'a, F>() -> &'a Descriptor
    where
        F: Formula,
    {
        let d = Descriptor::new::<F>();

        let mut cache = GLOBAL_CACHE.write();
        match cache.entry(TypeId::of::<F>()) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let d = Box::leak(Box::new(d));
                *entry.insert(d)
            }
        }
    }

    type GlobalCache = HashMap<TypeId, &'static Descriptor, NoopHasherBuilder>;

    static GLOBAL_CACHE: parking_lot::RwLock<GlobalCache> =
        parking_lot::const_rwlock(HashMap::with_hasher(NoopHasherBuilder));
}

#[cfg(feature = "evolution-global-cache")]
pub use global_cache::descriptor;

struct NoopHasher(u64);

impl Hasher for NoopHasher {
    fn write(&mut self, bytes: &[u8]) {
        let len = bytes.len().min(size_of::<u64>());
        let mut buf = [0u8; size_of::<u64>()];
        buf[..len].copy_from_slice(&bytes[..len]);
        self.0 = u64::from_ne_bytes(buf);
    }

    #[inline(always)]
    fn write_u128(&mut self, i: u128) {
        self.0 = i as u64;
    }

    #[inline(always)]
    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }

    #[inline(always)]
    fn write_u32(&mut self, i: u32) {
        self.0 = i as u64;
    }

    #[inline(always)]
    fn write_u16(&mut self, i: u16) {
        self.0 = i as u64;
    }

    #[inline(always)]
    fn write_u8(&mut self, i: u8) {
        self.0 = i as u64;
    }

    #[inline(always)]
    fn write_usize(&mut self, i: usize) {
        self.0 = i as u64;
    }

    #[inline(always)]
    fn finish(&self) -> u64 {
        self.0
    }
}

/// Builds hasher that does nothing.
struct NoopHasherBuilder;

impl BuildHasher for NoopHasherBuilder {
    type Hasher = NoopHasher;

    #[inline(always)]
    fn build_hasher(&self) -> Self::Hasher {
        NoopHasher(0)
    }
}
