use core::{fmt, marker::PhantomData, ops};

use crate::{
    buffer::{Buffer, BufferExhausted, CheckedFixedBuffer, DryBuffer, MaybeFixedBuffer},
    formula::{unwrap_size, BareFormula, Formula},
    size::{FixedUsize, SIZE_STACK},
};

#[cfg(feature = "alloc")]
use crate::buffer::VecBuffer;

/// Heap and stack sizes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sizes {
    /// Heap size.
    pub heap: usize,

    /// Stack size.
    pub stack: usize,
}

impl Sizes {
    /// Zero sizes.
    pub const ZERO: Self = Sizes { heap: 0, stack: 0 };

    /// Create new `Sizes` with specified heap size.
    #[must_use]
    #[inline(always)]
    pub const fn with_heap(heap: usize) -> Self {
        Sizes { heap, stack: 0 }
    }

    /// Create new `Sizes` with specified stack size.
    #[must_use]
    #[inline(always)]
    pub const fn with_stack(stack: usize) -> Self {
        Sizes { heap: 0, stack }
    }

    /// Adds to the heap size.
    #[inline(always)]
    pub fn add_heap(&mut self, heap: usize) {
        self.heap += heap;
    }

    /// Adds to the stack size.
    #[inline(always)]
    pub fn add_stack(&mut self, stack: usize) {
        self.stack += stack;
    }

    /// Moves stack size to heap size.
    #[inline(always)]
    pub fn to_heap(&mut self, until: usize) -> usize {
        let len = self.stack - until;
        self.heap += len;
        self.stack = until;
        len
    }

    /// Returns total size.
    #[inline(always)]
    pub fn total(&self) -> usize {
        self.heap + self.stack
    }
}

impl ops::Add for Sizes {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self {
        Self {
            heap: self.heap + rhs.heap,
            stack: self.stack + rhs.stack,
        }
    }
}

impl ops::AddAssign for Sizes {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.heap += rhs.heap;
        self.stack += rhs.stack;
    }
}

/// Trait for types that can be serialized
/// into raw bytes with specified `F: `[`Formula`].
///
/// Implementations *must* write data according to the formula.
/// Doing otherwise may result in errors during deserialization.
/// Where errors may be both failures to deserialize and
/// incorrect deserialized values.
///
/// # Examples
///
/// ```
/// # use alkahest::{*, advanced::*};
///
/// struct ThreeBytes;
///
/// impl Formula for ThreeBytes {
///     const MAX_STACK_SIZE: Option<usize> = Some(3);
///     const EXACT_SIZE: bool = true;
///     const HEAPLESS: bool = true;
/// }
///
/// struct Qwe;
///
/// impl Serialize<ThreeBytes> for Qwe {
///     fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
///     where
///         Self: Sized,
///         B: Buffer,
///     {
///         write_bytes(b"qwe", sizes, buffer)
///     }
///
///     fn size_hint(&self) -> Option<Sizes> {
///         Some(Sizes::with_stack(3))
///     }
/// }
/// ```
#[cfg_attr(
    feature = "derive",
    doc = r#"

When "derive" feature is enabled, `derive(Serialize)` is also available.

```
# use alkahest::*;


/// Self-serializable empty formula.
#[alkahest(Formula, Serialize)]
struct EmptyFormula {}

/// Another type serializable with `EmptyFormula`.
#[alkahest(Serialize<EmptyFormula>)]
struct EmptySerialize;


/// Formula for serializing tuple structures with fields
/// that are serializable with `u8` and `[u16]` formulas.
/// Slice formulas are serialized from some `IntoIterator`s and `SerIter` wrapper over any `Iterator`
/// with serializable item type.
#[alkahest(Formula)]
struct TupleFormula(u8, [u16]);


#[alkahest(Serialize<TupleFormula>)]
struct TupleSerialize(u8, std::iter::Once<u16>);


/// Formula for serializing structures with fields
/// that are serializable with `u8` and `str` formulas.
#[alkahest(Formula)]
struct StructFormula {
    a: u8,
    b: str,
}

# #[cfg(feature = "alloc")]
/// `String` can be serialized with `str` formula.
#[alkahest(Serialize<StructFormula>)]
struct StructSerialize {
    a: u8,
    b: String,
}

# #[cfg(feature = "alloc")]
/// Formula for serializing enums.
#[alkahest(Formula, Serialize)]
enum EnumFormula {
    A,
    B(u8),
    C { y: String },
}

# #[cfg(feature = "alloc")]
/// `&str` can be serialized with `String` formula.
#[alkahest(Serialize<EnumFormula>)]
# // While `Formula` derive macro makes all variants and fields used,
# // this is not the case for `Serialize` derive macro.
# #[allow(dead_code)]
enum EnumSerialize<'a> {
    A,
    B(u8),
    C { y: &'a str },
}

# #[cfg(feature = "alloc")]
/// `&str` can be serialized with `String` formula.
#[alkahest(Serialize<EnumFormula @C>)]
struct CVariantSerialize {
    y: String,
}
```

Names of the formula variants and fields are important for `Serialize` and `Deserialize` derive macros.
"#
)]
pub trait Serialize<F: Formula + ?Sized> {
    /// Serializes `self` into the given buffer.
    /// `heap` specifies the size of the buffer's heap occupied prior to this call.
    ///
    /// # Errors
    ///
    /// Returns error if buffer write fails.
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        Self: Sized,
        B: Buffer;

    /// Returns heap and stack sizes required to serialize `self`.
    /// If some sizes are returned they must be exact.
    ///
    /// This function may return none conservatively.
    ///
    /// Returning incorrect sizes may cause panics during implementation
    /// or broken data.
    fn size_hint(&self) -> Option<Sizes>;
}

// impl<'ser, F, T: ?Sized> Serialize<F> for &&'ser T
// where
//     F: BareFormula + ?Sized,
//     &'ser T: Serialize<F>,
// {
//     #[inline(always)]
//     fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
//     where
//         Self: Sized,
//         B: Buffer,
//     {
//         <&'ser T as Serialize<F>>::serialize(self, sizes, buffer)
//     }

//     #[inline(always)]
//     fn size_hint(&self) -> Option<Sizes> {
//         <&'ser T as Serialize<F>>::size_hint(self)
//     }
// }

/// `Serialize` but for references.
pub trait SerializeRef<F: Formula + ?Sized> {
    /// Serializes `self` into the given buffer.
    /// `heap` specifies the size of the buffer's heap occupied prior to this call.
    ///
    /// # Errors
    ///
    /// Returns error if buffer write fails.
    fn serialize<B>(&self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer;

    /// Returns heap and stack sizes required to serialize `self`.
    /// If some sizes are returned they must be exact.
    ///
    /// This function may return none conservatively.
    ///
    /// Returning incorrect sizes may cause panics during implementation
    /// or broken data.
    fn size_hint(&self) -> Option<Sizes>;
}

impl<F, T> SerializeRef<F> for &T
where
    F: Formula + ?Sized,
    T: ?Sized,
    for<'a> &'a T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<B>(&self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        Self: Sized,
        B: Buffer,
    {
        <&T as Serialize<F>>::serialize(self, sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        <&T as Serialize<F>>::size_hint(self)
    }
}

impl<F, T> Serialize<F> for &T
where
    F: BareFormula + ?Sized,
    T: SerializeRef<F> + ?Sized,
{
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        Self: Sized,
        B: Buffer,
    {
        <T as SerializeRef<F>>::serialize(self, sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        <T as SerializeRef<F>>::size_hint(self)
    }
}

/// Serialize value into buffer.
/// Returns total number of bytes written and size of the root value.
/// The buffer type controls bytes writing and failing strategy.
#[inline(always)]
pub fn serialize_into<F, T, B>(value: T, buffer: B) -> Result<(usize, usize), B::Error>
where
    F: Formula + ?Sized,
    T: Serialize<F>,
    B: Buffer,
{
    let mut sizes = Sizes { heap: 0, stack: 0 };
    let size = write_ref(value, &mut sizes, buffer)?;
    Ok((sizes.heap, size))
}

/// Serialize value into bytes slice.
/// Returns the number of bytes written.
/// Fails if the buffer is too small.
///
/// To retrieve the number of bytes required to serialize the value,
/// use [`serialized_size`] or [`serialize_or_size`].
///
/// # Errors
///
/// Returns [`BufferExhausted`] if the buffer is too small.
#[inline(always)]
pub fn serialize<F, T>(value: T, output: &mut [u8]) -> Result<(usize, usize), BufferExhausted>
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    serialize_into::<F, T, _>(value, CheckedFixedBuffer::new(output))
}

/// Slightly faster version of [`serialize`].
/// Panics if the buffer is too small instead of returning an error.
///
/// Use instead of using [`serialize`] with immediate [`unwrap`](Result::unwrap).
#[inline(always)]
pub fn serialize_unchecked<F, T>(value: T, output: &mut [u8]) -> (usize, usize)
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    match serialize_into::<F, T, _>(value, output) {
        Ok(sizes) => sizes,
        Err(never) => match never {},
    }
}

/// Error that may occur during serialization
/// if buffer is too small to fit serialized data.
///
/// Contains the size of the buffer required to fit serialized data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct BufferSizeRequired {
    /// Size of the buffer required to fit serialized data.
    pub required: usize,
}

impl fmt::Display for BufferSizeRequired {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "buffer size required: {}", self.required)
    }
}

/// Serialize value into bytes slice.
/// Returns the number of bytes written.
///
/// If the buffer is too small, returns error that contains
/// the exact number of bytes required.
///
/// Use [`serialize`] if this information is not needed.
///
/// # Errors
///
/// Returns [`BufferSizeRequired`] error if the buffer is too small.
/// Error contains the exact number of bytes required.
#[inline(always)]
pub fn serialize_or_size<F, T>(
    value: T,
    output: &mut [u8],
) -> Result<(usize, usize), BufferSizeRequired>
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    let mut exhausted = false;
    let result = serialize_into::<F, T, _>(value, MaybeFixedBuffer::new(output, &mut exhausted));
    let sizes = match result {
        Ok(sizes) => sizes,
        Err(never) => match never {},
    };
    if exhausted {
        Err(BufferSizeRequired { required: sizes.0 })
    } else {
        Ok(sizes)
    }
}

/// Serialize value into byte vector.
/// Returns the number of bytes written.
///
/// Grows the vector if needed.
/// Infallible except for allocation errors.
///
/// Use pre-allocated vector when possible to avoid reallocations.
#[cfg(feature = "alloc")]
#[inline(always)]
pub fn serialize_to_vec<F, T>(value: T, output: &mut alloc::vec::Vec<u8>) -> (usize, usize)
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    match serialize_into::<F, T, _>(value, VecBuffer::new(output)) {
        Ok(sizes) => sizes,
        Err(never) => match never {},
    }
}

/// Returns the number of bytes required to serialize the value.
/// Note that value is consumed.
///
/// Use when value is `Copy` or can be cheaply replicated to allocate
/// the buffer for serialization in advance.
/// Or to find out required size after [`serialize`] fails.
#[inline(always)]
pub fn serialized_size<F, T>(value: T) -> (usize, usize)
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    let mut sizes = Sizes::ZERO;
    match Serialize::<F>::serialize(value, &mut sizes, DryBuffer) {
        Ok(()) => (sizes.total(), sizes.stack),
        Err(never) => match never {},
    }
}

/// Size hint for serializing a field.
///
/// Use in [`Serialize::size_hint`](Serialize::size_hint) implementation.
#[inline(always)]
pub fn field_size_hint<F: Formula + ?Sized>(
    value: &impl Serialize<F>,
    last: bool,
) -> Option<Sizes> {
    match (last, F::MAX_STACK_SIZE) {
        (false, None) => None,
        (true, _) => {
            let sizes = value.size_hint()?;
            Some(sizes)
        }
        (false, Some(max_stack)) => {
            let sizes = value.size_hint()?;
            debug_assert!(sizes.stack <= max_stack);
            Some(Sizes {
                heap: sizes.heap,
                stack: max_stack,
            })
        }
    }
}

/// Writes reference into the buffer.
///
/// Use in [`Serialize::serialize`](Serialize::serialize) implementation
/// after writing value to the heap.
///
/// # Errors
///
/// Returns error if buffer write fails.
#[inline(always)]
pub fn write_reference<F, B>(
    size: usize,
    address: usize,
    heap: usize,
    stack: usize,
    mut buffer: B,
) -> Result<(), B::Error>
where
    F: Formula + ?Sized,
    B: Buffer,
{
    let address = FixedUsize::truncate_unchecked(address);
    let size = FixedUsize::truncate_unchecked(size);

    if F::EXACT_SIZE {
        buffer.write_stack(heap, stack, &address.to_le_bytes())?;
    } else {
        buffer.write_stack(heap, stack, &size.to_le_bytes())?;
        buffer.write_stack(heap, stack + SIZE_STACK, &address.to_le_bytes())?;
    }
    Ok(())
}

/// Writes field value into the buffer.
///
/// Use in [`Serialize::serialize`](Serialize::serialize) implementation.
///
/// # Errors
///
/// Returns error if buffer write fails.
#[inline(always)]
pub fn write_field<F, T, B>(
    value: T,
    sizes: &mut Sizes,
    mut buffer: B,
    last: bool,
) -> Result<(), B::Error>
where
    F: Formula + ?Sized,
    T: Serialize<F>,
    B: Buffer,
{
    if !last && F::MAX_STACK_SIZE.is_none() {
        buffer.write_stack(sizes.heap, sizes.stack, &[0; SIZE_STACK])?;
        sizes.stack += SIZE_STACK;
    }

    let old_stack = sizes.stack;
    <T as Serialize<F>>::serialize(value, sizes, buffer.reborrow())?;

    match (F::MAX_STACK_SIZE, F::EXACT_SIZE, last) {
        (None, _, false) => {
            let size = FixedUsize::truncate_unchecked(sizes.stack - old_stack);
            let res = buffer.write_stack(sizes.heap, old_stack - SIZE_STACK, &size.to_le_bytes());
            if res.is_err() {
                unreachable!("Successfully written before");
            };
        }
        (None, _, true) => {}
        (Some(max_stack), false, false) => {
            debug_assert!(sizes.stack - old_stack <= max_stack);
            buffer.pad_stack(sizes.heap, sizes.stack, old_stack + max_stack - sizes.stack)?;
            sizes.stack = old_stack + max_stack;
        }
        (Some(max_stack), false, true) => {
            debug_assert!(sizes.stack - old_stack <= max_stack);
        }
        (Some(max_stack), true, _) => {
            debug_assert_eq!(sizes.stack - old_stack, max_stack);
        }
    }

    Ok(())
}

/// Write a field with exact size into buffer.
/// Requires that `F::EXACT_SIZE` is `true` and
/// `F::MAX_STACK_SIZE` is `Some`.
///
/// Use in [`Serialize::serialize`](Serialize::serialize) implementation.
///
/// # Errors
///
/// Returns error if buffer write fails.
#[inline(always)]
pub fn write_exact_size_field<F, T, B>(
    value: T,
    sizes: &mut Sizes,
    buffer: B,
) -> Result<(), B::Error>
where
    F: Formula + ?Sized,
    T: Serialize<F>,
    B: Buffer,
{
    debug_assert!(F::EXACT_SIZE);

    let old_stack = sizes.stack;
    <T as Serialize<F>>::serialize(value, sizes, buffer)?;
    debug_assert_eq!(old_stack + unwrap_size(F::MAX_STACK_SIZE), sizes.stack);
    Ok(())
}

/// Write raw bytes to the buffer.
///
/// Use in [`Serialize::serialize`](Serialize::serialize) implementation.
///
/// # Errors
///
/// Returns error if buffer write fails.
#[inline(always)]
pub fn write_bytes<B>(bytes: &[u8], sizes: &mut Sizes, mut buffer: B) -> Result<(), B::Error>
where
    B: Buffer,
{
    buffer.write_stack(sizes.heap, sizes.stack, bytes)?;
    sizes.stack += bytes.len();
    Ok(())
}

#[cold]
#[inline(always)]
fn write_ref_slow<F, T, B>(value: T, sizes: &mut Sizes, mut buffer: B) -> Result<usize, B::Error>
where
    F: Formula + ?Sized,
    T: Serialize<F>,
    B: Buffer,
{
    let old_stack = sizes.stack;
    write_field(value, sizes, buffer.reborrow(), true)?;
    let len = sizes.to_heap(old_stack);
    buffer.move_to_heap(sizes.heap - len, sizes.stack + len, len);
    Ok(len)
}

/// Write value to the buffer as a reference,
/// placing value into the heap and reference into the stack.
///
/// Use in [`Serialize::serialize`](Serialize::serialize) implementation.
///
/// # Errors
///
/// Returns error if buffer write fails.
#[must_use]
#[inline(always)]
pub fn write_ref<F, T, B>(value: T, sizes: &mut Sizes, mut buffer: B) -> Result<usize, B::Error>
where
    F: Formula + ?Sized,
    T: Serialize<F>,
    B: Buffer,
{
    // Can we get promised sizes?
    let promised = <T as Serialize<F>>::size_hint(&value);

    let stack = match promised {
        None => write_ref_slow(value, sizes, buffer.reborrow())?,
        Some(promised) => match buffer.reserve_heap(sizes.heap, sizes.stack, promised.total())? {
            [] => match write_ref_slow(value, sizes, DryBuffer) {
                Ok(stack) => stack,
                Err(never) => match never {},
            },
            reserved => {
                let mut reserved_sizes = Sizes {
                    heap: sizes.heap,
                    stack: 0,
                };
                <T as Serialize<F>>::serialize(value, &mut reserved_sizes, reserved)
                    .expect("Reserved enough space");

                debug_assert_eq!(reserved_sizes.heap, sizes.heap + promised.heap);
                debug_assert_eq!(reserved_sizes.stack, promised.stack);

                sizes.heap = reserved_sizes.total();
                reserved_sizes.stack
            }
        },
    };

    Ok(stack)
}

/// Writes elements of a slice one by one into associated buffer.
///
/// Use in [`Serialize::serialize`](Serialize::serialize) implementation
/// for slice formulas.
#[must_use]
pub struct SliceWriter<'a, F: Formula + ?Sized, B: Buffer + ?Sized> {
    buffer: &'a mut B,
    sizes: &'a mut Sizes,
    count: usize,
    marker: PhantomData<fn(&F)>,
}

impl<'a, F, B> SliceWriter<'a, F, B>
where
    F: Formula + ?Sized,
    B: Buffer + ?Sized,
{
    /// Serialize next element of a slice.
    ///
    /// # Errors
    ///
    /// Returns error if buffer write fails.
    #[inline(always)]
    pub fn write_elem<T>(&mut self, value: T) -> Result<(), B::Error>
    where
        T: Serialize<F>,
    {
        if let Some(0) = <F as Formula>::MAX_STACK_SIZE {
            debug_assert!(<F as Formula>::HEAPLESS);
            debug_assert!(serialize::<F, T>(value, &mut []).is_ok());
            self.count += 1;
            Ok(())
        } else {
            write_field::<F, _, _>(value, self.sizes, self.buffer.reborrow(), false)
        }
    }

    /// Finishes the slice serialization.
    ///
    /// # Errors
    ///
    /// Returns error if buffer write fails.
    #[inline(always)]
    pub fn finish(self) -> Result<(), B::Error> {
        if let Some(0) = <F as Formula>::MAX_STACK_SIZE {
            debug_assert!(<F as Formula>::HEAPLESS);
            write_field::<FixedUsize, _, _>(self.count, self.sizes, self.buffer.reborrow(), true)?;
        }
        Ok(())
    }
}

/// Returns a writer to write elements of a slice
/// one by one into associated buffer.
///
/// Use in [`Serialize::serialize`](Serialize::serialize) implementation
/// for slice formulas.
#[inline(always)]
pub fn slice_writer<'a, F, B>(sizes: &'a mut Sizes, buffer: &'a mut B) -> SliceWriter<'a, F, B>
where
    F: Formula + ?Sized,
    B: Buffer,
{
    SliceWriter {
        buffer,
        sizes,
        count: 0,
        marker: PhantomData,
    }
}

/// Writes iterator into buffer.
///
/// Use in [`Serialize::serialize`](Serialize::serialize) implementation
/// for slice formulas.
/// Prefer this over `slice_writer` and manual iteration when
/// got an iterator.
///
/// # Errors
///
/// Returns error if buffer write fails.
#[inline(always)]
pub fn write_slice<F, T, B>(
    mut iter: impl Iterator<Item = T>,
    sizes: &mut Sizes,
    mut buffer: B,
) -> Result<(), B::Error>
where
    F: Formula + ?Sized,
    T: Serialize<F>,
    B: Buffer,
{
    if let Some(0) = <F as Formula>::MAX_STACK_SIZE {
        debug_assert!(<F as Formula>::HEAPLESS);
        let count = if cfg!(debug_assertions) {
            iter.fold(0, |acc, item| {
                let r = serialize::<F, T>(item, &mut []);
                assert!(r.is_ok());
                acc + 1
            })
        } else {
            iter.count()
        };
        write_field::<FixedUsize, _, _>(count, sizes, buffer, true)
    } else {
        iter.try_fold((), |(), elem| {
            write_field::<F, _, _>(elem, sizes, buffer.reborrow(), false)
        })
    }
}

/// Writes array into buffer.
///
/// Use in [`Serialize::serialize`](Serialize::serialize) implementation
/// for slice formulas.
/// Prefer this over `slice_writer` and manual iteration when
/// got an iterator.
///
/// # Errors
///
/// Returns error if buffer write fails.
#[inline(always)]
pub fn write_array<F, T, B>(
    mut iter: impl Iterator<Item = T>,
    sizes: &mut Sizes,
    mut buffer: B,
) -> Result<(), B::Error>
where
    F: Formula + ?Sized,
    T: Serialize<F>,
    B: Buffer,
{
    iter.try_fold((), |(), elem| {
        write_field::<F, _, _>(elem, sizes, buffer.reborrow(), false)
    })
}

/// Returns size hint for the formula if it is known at compile time.
///
/// Use in [`Serialize::size_hint`](Serialize::size_hint) implementation
/// before manual calculation.
#[must_use]
#[inline(always)]
pub const fn formula_fast_sizes<F>() -> Option<Sizes>
where
    F: Formula + ?Sized,
{
    match (F::EXACT_SIZE, F::HEAPLESS, F::MAX_STACK_SIZE) {
        (true, true, Some(max_stack_size)) => Some(Sizes::with_stack(max_stack_size)),
        _ => None,
    }
}
