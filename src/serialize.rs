use core::{any::type_name, marker::PhantomData, mem::size_of, ops};

use crate::{
    buffer::*,
    formula::{reference_size, BareFormula, Formula},
    size::FixedUsize,
};

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
    #[inline(always)]
    pub const fn with_heap(heap: usize) -> Self {
        Sizes { heap, stack: 0 }
    }

    /// Create new `Sizes` with specified stack size.
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
/// # use alkahest::advanced::*;
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
#[derive(Formula, Serialize)]
struct EmptyFormula {}

/// Another type serializable with `EmptyFormula`.
#[derive(Serialize)]
#[alkahest(EmptyFormula)]
struct EmptySerialize;


/// Formula for serializing tuple structures with fields
/// that are serializable with `u8` and `[u16]` formulas.
/// Slice formulas are serialized from some `IntoIterator`s and `SerIter` wrapper over any `Iterator`
/// with serializable item type.
#[derive(Formula)]
struct TupleFormula(u8, [u16]);


#[derive(Serialize)]
#[alkahest(owned(TupleFormula))] // `owned()` because iterators cannot be serialized by reference.
struct TupleSerialize(u8, std::iter::Once<u16>);


/// Formula for serializing structures with fields
/// that are serializable with `u8` and `str` formulas.
#[derive(Formula)]
struct StructFormula {
    a: u8,
    b: str,
}

# #[cfg(feature = "alloc")]
/// `String` can be serialized with `str` formula.
#[derive(Serialize)]
#[alkahest(StructFormula)]
struct StructSerialize {
    a: u8,
    b: String,
}

# #[cfg(feature = "alloc")]
/// Formula for serializing enums.
#[derive(Formula, Serialize)]
enum EnumFormula {
    A,
    B(u8),
    C { y: String },
}

# #[cfg(feature = "alloc")]
/// `&str` can be serialized with `String` formula.
#[derive(Serialize)]
#[alkahest(EnumFormula)]
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
#[derive(Serialize)]
#[alkahest(EnumFormula, @C)]
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

impl<'ser, F, T: ?Sized> Serialize<F> for &&'ser T
where
    F: BareFormula + ?Sized,
    &'ser T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        Self: Sized,
        B: Buffer,
    {
        <&'ser T as Serialize<F>>::serialize(self, sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        <&'ser T as Serialize<F>>::size_hint(self)
    }
}

/// Serialize value into buffer.
/// The buffer type controls bytes writing and failing strategy.
#[inline]
pub fn serialize_into<F, T, B>(value: T, mut buffer: B) -> Result<usize, B::Error>
where
    F: Formula + ?Sized,
    T: Serialize<F>,
    B: Buffer,
{
    let reference_size = reference_size::<F>();
    buffer.reserve_heap(0, 0, reference_size)?;

    // Can we get promised sizes?
    let promised = <T as Serialize<F>>::size_hint(&value);

    let mut sizes = Sizes {
        heap: reference_size,
        stack: 0,
    };

    match promised {
        None => <T as Serialize<F>>::serialize(value, &mut sizes, buffer.reborrow())?,
        Some(promised) => {
            match buffer.reserve_heap(reference_size, 0, promised.heap + promised.stack)? {
                None => {
                    sizes += serialized_sizes(value);
                }
                Some(reserved) => {
                    <T as Serialize<F>>::serialize(value, &mut sizes, reserved).unwrap();
                    debug_assert_eq!(sizes.heap, promised.heap + reference_size);
                    debug_assert_eq!(sizes.stack, promised.stack);
                }
            }
        }
    };
    check_stack::<F, T>(sizes.stack);

    match promised {
        None => buffer.move_to_heap(sizes.heap, sizes.stack, sizes.stack),
        Some(promised) => {
            debug_assert_eq!(
                reference_size + promised.heap,
                sizes.heap,
                "<{} as Serialize<{}>>::size_hint() result is incorrect",
                type_name::<T>(),
                type_name::<F>()
            );
            debug_assert_eq!(
                promised.stack,
                sizes.stack,
                "<{} as Serialize<{}>>::size_hint() result is incorrect",
                type_name::<T>(),
                type_name::<F>()
            );
        }
    }

    if let Some(reserved) = buffer.reserve_heap(0, 0, reference_size)? {
        write_reference::<F, _>(sizes.stack, sizes.heap + sizes.stack, 0, 0, reserved).unwrap();
    }

    Ok(sizes.heap + sizes.stack)
}

#[inline(always)]
pub fn serialize<F, T>(value: T, output: &mut [u8]) -> Result<usize, BufferExhausted>
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    serialize_into::<F, T, _>(value, CheckedFixedBuffer::new(output))
}

#[inline(always)]
pub fn serialize_unchecked<F, T>(value: T, output: &mut [u8]) -> usize
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    match serialize_into::<F, T, _>(value, output) {
        Ok(size) => size,
        Err(never) => match never {},
    }
}

#[inline(always)]
pub fn serialize_or_size<F, T>(value: T, output: &mut [u8]) -> Result<usize, BufferSizeRequired>
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    let mut exhausted = false;
    let size = serialize_into::<F, T, _>(value, MaybeFixedBuffer::new(output, &mut exhausted))?;
    if exhausted {
        Err(BufferSizeRequired { required: size })
    } else {
        Ok(size)
    }
}

#[cfg(feature = "alloc")]
#[inline(always)]
pub fn serialize_to_vec<F, T>(value: T, output: &mut alloc::vec::Vec<u8>) -> usize
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    match serialize_into::<F, T, _>(value, VecBuffer::new(output)) {
        Ok(size) => size,
        Err(never) => match never {},
    }
}

#[inline(always)]
fn serialized_sizes<F, T>(value: T) -> Sizes
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    let mut sizes = Sizes::ZERO;
    match Serialize::<F>::serialize(value, &mut sizes, DryBuffer) {
        Ok(()) => sizes,
        Err(never) => match never {},
    }
}

#[inline(always)]
pub fn serialized_size<F, T>(value: T) -> usize
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    let reference_size = reference_size::<F>();
    let sizes = serialized_sizes::<F, T>(value);
    sizes.heap + sizes.stack + reference_size
}

#[inline(always)]
#[track_caller]
fn check_stack<F, T>(stack: usize)
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    if let Some(max_size) = F::MAX_STACK_SIZE {
        assert!(
            stack <= max_size,
            "Incorrect `<{} as Serialize<{}>>` implementation. `stack` size is `{}` but must be at most `{}`",
            type_name::<T>(),
            type_name::<F>(),
            stack,
            max_size,
        )
    };
}

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

    match (F::MAX_STACK_SIZE, F::EXACT_SIZE) {
        (Some(0), _) => {
            // do nothing
        }
        (Some(_), true) => {
            buffer.write_stack(heap, stack, &address.to_le_bytes())?;
        }
        _ => {
            buffer.write_stack(
                heap,
                stack + size_of::<FixedUsize>(),
                &address.to_le_bytes(),
            )?;
            buffer.write_stack(heap, stack, &size.to_le_bytes())?;
        }
    }
    Ok(())
}

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
        buffer.write_stack(sizes.heap, sizes.stack, &[0; size_of::<FixedUsize>()])?;
        sizes.stack += size_of::<FixedUsize>();
    }

    let old_stack = sizes.stack;
    <T as Serialize<F>>::serialize(value, sizes, buffer.reborrow())?;

    match (last, F::MAX_STACK_SIZE) {
        (true, None) => {}
        (true, Some(max_stack)) => {
            debug_assert!(sizes.stack - old_stack <= max_stack);
        }
        (false, None) => {
            let size = FixedUsize::truncate_unchecked(sizes.stack - old_stack);
            let res = buffer.write_stack(
                sizes.heap,
                old_stack - size_of::<FixedUsize>(),
                &size.to_le_bytes(),
            );
            if res.is_err() {
                unreachable!("Successfully written before");
            };
        }
        (false, Some(max_stack)) => {
            debug_assert!(sizes.stack - old_stack <= max_stack);
            sizes.stack = old_stack + max_stack;
        }
    }

    Ok(())
}

/// Write a field with exact size.
#[inline(always)]
pub fn write_exact_size_field<F, T, B>(
    value: T,
    sizes: &mut Sizes,
    mut buffer: B,
) -> Result<(), B::Error>
where
    F: Formula + ?Sized,
    T: Serialize<F>,
    B: Buffer,
{
    debug_assert!(F::EXACT_SIZE);
    let old_stack = sizes.stack;
    <T as Serialize<F>>::serialize(value, sizes, buffer.reborrow())?;
    debug_assert_eq!(old_stack + F::MAX_STACK_SIZE.unwrap(), sizes.stack);
    Ok(())
}

/// Write bytes to the stack.
#[inline(always)]
pub fn write_bytes<B>(bytes: &[u8], sizes: &mut Sizes, mut buffer: B) -> Result<(), B::Error>
where
    B: Buffer,
{
    buffer.write_stack(sizes.heap, sizes.stack, bytes)?;
    sizes.stack += bytes.len();
    Ok(())
}

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

/// Write to the heap and reference to the stack.
#[inline(always)]
pub fn write_ref<F, T, B>(value: T, sizes: &mut Sizes, mut buffer: B) -> Result<(), B::Error>
where
    F: Formula + ?Sized,
    T: Serialize<F>,
    B: Buffer,
{
    let reference_size = reference_size::<F>();

    // Can we get promised sizes?
    let promised = <T as Serialize<F>>::size_hint(&value);

    let stack = match promised {
        None => write_ref_slow(value, sizes, buffer.reborrow())?,
        Some(promised) => {
            match buffer.reserve_heap(sizes.heap, sizes.stack, promised.heap + promised.stack)? {
                None => write_ref_slow(value, sizes, buffer.reborrow())?,
                Some(reserved) => {
                    let mut reserved_sizes = Sizes {
                        heap: sizes.heap,
                        stack: 0,
                    };
                    <T as Serialize<F>>::serialize(value, &mut reserved_sizes, reserved).unwrap();

                    debug_assert_eq!(reserved_sizes.heap, sizes.heap + promised.heap);
                    debug_assert_eq!(reserved_sizes.stack, promised.stack);

                    sizes.heap = reserved_sizes.heap + reserved_sizes.stack;
                    reserved_sizes.stack
                }
            }
        }
    };
    write_reference::<F, B>(stack, sizes.heap, sizes.heap, sizes.stack, buffer)?;
    sizes.stack += reference_size;
    Ok(())
}

/// Writes slices of elements.
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
    /// Serialize next element into the slice.
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
    pub fn finish(self) -> Result<(), B::Error> {
        if let Some(0) = <F as Formula>::MAX_STACK_SIZE {
            debug_assert!(<F as Formula>::HEAPLESS);
            write_field::<FixedUsize, _, _>(self.count, self.sizes, self.buffer.reborrow(), true)?;
        }
        Ok(())
    }
}

/// Writes iterator into slice formula.
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
                assert!(serialize::<F, T>(item, &mut []).is_ok());
                acc + 1
            })
        } else {
            iter.count()
        };
        write_field::<FixedUsize, _, _>(count, sizes, buffer, true)
    } else {
        iter.try_for_each(|elem| write_field::<F, _, _>(elem, sizes, buffer.reborrow(), false))?;
        Ok(())
    }
}

/// Returns a writer for slice formula.
/// It can be used to serialize elements one by one.
/// `SliceWriter::finish` must be called to finish the serialization.
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

/// Fast sizes for formula if it is known at compile time.
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
