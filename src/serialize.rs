use core::{any::type_name, mem::size_of};

use crate::{
    buffer::*,
    formula::{BareFormula, Formula},
    size::FixedUsize,
};

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
/// # use alkahest::*;
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
///     fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
///     where
///         Self: Sized,
///         S: Serializer,
///     {
///         let mut ser = ser.into();
///         ser.write_bytes(b"qwe")?;
///         ser.finish()
///     }
///
///     fn size_hint(&self) -> Option<usize> {
///         Some(3)
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
    /// Serializes `self` into given serializer.
    fn serialize<S>(self, serializer: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        Self: Sized,
        S: Serializer;

    /// Returns heap and stack sizes required to serialize `self`.
    /// This function may return `None` conservatively.
    ///
    /// When possible to do it fast implementations *should* override this method
    /// and provide more accurate sizes.
    ///
    /// Implementations *should not* override this method
    /// if going through `serialize` method is faster.
    ///
    /// Returning incorrect sizes may cause panic during implementation
    /// or broken data.
    // #[inline(always)]
    fn size_hint(&self) -> Option<usize>;
}

impl<'ser, F, T: ?Sized> Serialize<F> for &&'ser T
where
    F: BareFormula + ?Sized,
    &'ser T: Serialize<F>,
{
    fn serialize<S>(self, serializer: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        Self: Sized,
        S: Serializer,
    {
        <&'ser T as Serialize<F>>::serialize(self, serializer)
    }

    fn size_hint(&self) -> Option<usize> {
        <&'ser T as Serialize<F>>::size_hint(self)
    }
}

/// Instances of this trait are provided to `Serialize::serialize` method.
/// It should be used to perform the serialization process.
/// Primitives use `Serializer::write_bytes` to store bytes representation
/// of the value.
/// Arrays serialize each element using `Serializer::write_value`.
/// Tuples serialize each element using `Serializer::write_value`.
/// Structs *should* prefer to use `Serializer::write_value`
/// for each field.
/// Enums *should* serialize the discriminant
/// and then serialize the variant fields using `Serializer::write_value`.
/// `Ref` formula uses `Serializer::write_ref`.
pub trait Serializer {
    type Ok;
    type Error;

    /// Writes raw bytes into serializer.
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Self::Error>;

    /// Writes a value with specific formula into serializer.
    fn write_value<F, T>(&mut self, value: T) -> Result<(), Self::Error>
    where
        F: Formula + ?Sized,
        T: Serialize<F>;

    /// Writes a value with specific formula into serializer.
    fn write_last_value<F, T>(self, value: T) -> Result<Self::Ok, Self::Error>
    where
        F: Formula + ?Sized,
        T: Serialize<F>;

    /// Writes a value with specific formula into serializer.
    /// It avoids padding the value with zeros to `F::MAX_STACK_SIZE`.
    /// Instead creates indirection and consumes few bytes to store
    /// address and size of serialized value.
    ///
    /// This method is used for any `Ref` formula.
    ///
    /// User should prefer wrapping their formulas with `Ref` instead
    /// of using this method manually to avoid potential mismatch in
    /// serialization and deserialization.
    fn write_ref<F, T>(self, value: T) -> Result<Self::Ok, Self::Error>
    where
        F: Formula + ?Sized,
        T: Serialize<F>;

    /// Writes iterator into slice formula.
    fn write_slice<F, T>(&mut self, mut iter: impl Iterator<Item = T>) -> Result<(), Self::Error>
    where
        F: Formula + ?Sized,
        T: Serialize<F>,
    {
        if let Some(0) = <F as Formula>::MAX_STACK_SIZE {
            debug_assert!(<F as Formula>::HEAPLESS);
            self.write_value::<FixedUsize, _>(iter.count())
        } else {
            iter.try_for_each(|elem| self.write_value::<F, _>(elem))?;
            Ok(())
        }
    }

    // /// Writes padding bytes into serializer.
    // /// Padding it automatically calculated.
    // /// Only array serialization should use this method.
    // fn write_pad(&mut self) -> Result<(), Self::Error>;

    /// Finish serialization.
    fn finish(self) -> Result<Self::Ok, Self::Error>;
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

    let (actual_heap, actual_stack) = match promised {
        None => match buffer.sub(0) {
            None => {
                let (heap, stack) = serialized_sizes(value);
                (heap + reference_size, stack)
            }
            Some(sub) => <T as Serialize<F>>::serialize::<BufferedSerializer<_>>(
                value,
                IntoBufferedSerializer {
                    buffer: sub,
                    heap: reference_size,
                },
            )?,
        },
        Some(size) => match buffer.reserve_heap(reference_size, 0, size)? {
            None => {
                let (heap, stack) = serialized_sizes(value);
                (heap + reference_size, stack)
            }
            Some(sub) => <T as Serialize<F>>::serialize::<BufferedSerializer<_>>(
                value,
                IntoBufferedSerializer {
                    buffer: sub,
                    heap: reference_size,
                },
            )
            .unwrap(),
        },
    };
    check_stack::<F, T>(actual_stack);

    match promised {
        None => buffer.move_to_heap(actual_heap, actual_stack, actual_stack),
        Some(size) => debug_assert!(
            reference_size + size >= actual_heap + actual_stack,
            "<{} as Serialize<{}>>::size_hint() result is incorrect",
            type_name::<T>(),
            type_name::<F>()
        ),
    }

    if let Some(mut sub) = buffer.reserve_heap(0, 0, reference_size)? {
        write_reference::<F, _>(actual_stack, actual_heap + actual_stack, 0, 0, &mut sub).unwrap();
    }

    buffer.finish(actual_heap, actual_stack)?;
    Ok(actual_heap + actual_stack)
}

#[inline(always)]
pub fn serialize_or_size<F, T>(value: T, output: &mut [u8]) -> Result<usize, BufferSizeRequired>
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    serialize_into::<F, T, _>(value, MaybeFixedBuffer::new(output))
}

#[inline(always)]
pub fn serialize<F, T>(value: T, output: &mut [u8]) -> Result<usize, BufferExhausted>
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    serialize_into::<F, T, _>(value, FixedBuffer::new(output))
}

#[inline(always)]
fn serialized_sizes<F, T>(value: T) -> (usize, usize)
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    match Serialize::<F>::serialize::<BufferedSerializer<DryBuffer>>(
        value,
        IntoBufferedSerializer {
            buffer: DryBuffer,
            heap: 0,
        },
    ) {
        Ok((heap, stack)) => (heap, stack),
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
    let (heap, stack) = serialized_sizes::<F, T>(value);
    heap + stack + reference_size
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
pub fn header_size<F>() -> usize
where
    F: Formula + ?Sized,
{
    reference_size::<F>()
}

#[inline(always)]
pub const fn reference_size<F>() -> usize
where
    F: Formula + ?Sized,
{
    match (F::MAX_STACK_SIZE, F::EXACT_SIZE) {
        (Some(0), _) => 0,
        (Some(_), true) => size_of::<FixedUsize>(),
        _ => size_of::<[FixedUsize; 2]>(),
    }
}

#[inline(always)]
fn write_reference<F, B>(
    size: usize,
    address: usize,
    heap: usize,
    stack: usize,
    buffer: &mut B,
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

struct IntoBufferedSerializer<B> {
    buffer: B,
    heap: usize,
}

impl<B> Into<BufferedSerializer<B>> for IntoBufferedSerializer<B> {
    #[inline(always)]
    fn into(self) -> BufferedSerializer<B> {
        BufferedSerializer::new(self.heap, self.buffer)
    }
}

/// Implementation of `Serializer` that uses
/// `Buffer` to store serialized.
/// `Buffer` provides failing and growing strategy.
#[must_use]
struct BufferedSerializer<B> {
    /// Output buffer.
    buffer: B,

    // size of the heap
    heap: usize,

    // start of the stack
    stack: usize,
}

type DrySerializer = BufferedSerializer<DryBuffer>;

struct IntoDrySerializer;

impl From<IntoDrySerializer> for DrySerializer {
    fn from(_: IntoDrySerializer) -> Self {
        DrySerializer::new(0, DryBuffer)
    }
}

impl<B> BufferedSerializer<B> {
    #[must_use]
    #[inline(always)]
    fn new(heap: usize, buffer: B) -> Self {
        BufferedSerializer {
            heap,
            stack: 0,
            buffer,
        }
    }
}

impl<B> BufferedSerializer<B>
where
    B: Buffer,
{
    #[inline(always)]
    fn write_value<F, T>(&mut self, value: T, last: bool) -> Result<(), B::Error>
    where
        F: Formula + ?Sized,
        T: Serialize<F>,
    {
        if !last && F::MAX_STACK_SIZE.is_none() {
            self.buffer
                .write_stack(self.heap, self.stack, &[0; size_of::<FixedUsize>()])?;
            self.stack += size_of::<FixedUsize>();
        }

        let stack = match self.buffer.sub(self.stack) {
            None => {
                match <T as Serialize<F>>::serialize::<DrySerializer>(value, IntoDrySerializer) {
                    Err(never) => match never {},
                    Ok((heap, stack)) => {
                        self.heap = heap;
                        self.stack += stack;
                    }
                }
                return Ok(());
            }
            Some(sub) => {
                let sub = IntoBufferedSerializer {
                    buffer: sub,
                    heap: self.heap,
                };

                let (heap, stack) =
                    <T as Serialize<F>>::serialize::<BufferedSerializer<_>>(value, sub)?;
                self.heap = heap;
                stack
            }
        };

        match (last, F::MAX_STACK_SIZE) {
            (true, None) => {
                self.stack += stack;
            }
            (true, Some(max_stack)) => {
                debug_assert!(stack <= max_stack);
                self.stack += stack;
            }
            (false, None) => {
                let size = FixedUsize::truncate_unchecked(stack);
                let res = self.buffer.write_stack(
                    self.heap,
                    self.stack - size_of::<FixedUsize>(),
                    &size.to_le_bytes(),
                );
                if res.is_err() {
                    unreachable!();
                };
                self.stack += stack;
            }
            (false, Some(max_stack)) => {
                debug_assert!(stack <= max_stack);
                self.stack += max_stack;
            }
        }

        Ok(())
    }

    #[inline(always)]
    fn write_ref_slow<F, T>(&mut self, value: T) -> Result<usize, B::Error>
    where
        F: Formula + ?Sized,
        T: Serialize<F>,
    {
        let old_stack = self.stack;
        self.write_value(value, true)?;
        let stack = self.stack - old_stack;
        self.buffer.move_to_heap(self.heap, self.stack, stack);
        self.heap += stack;
        self.stack = old_stack;
        Ok(stack)
    }
}

impl<B> Serializer for BufferedSerializer<B>
where
    B: Buffer,
{
    type Ok = (usize, usize);
    type Error = B::Error;

    #[inline(always)]
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), B::Error> {
        self.buffer.write_stack(self.heap, self.stack, bytes)?;
        self.stack += bytes.len();
        Ok(())
    }

    #[inline(always)]
    fn write_value<F, T>(&mut self, value: T) -> Result<(), B::Error>
    where
        F: Formula + ?Sized,
        T: Serialize<F>,
    {
        self.write_value(value, false)
    }

    #[inline(always)]
    fn write_last_value<F, T>(mut self, value: T) -> Result<(usize, usize), B::Error>
    where
        F: Formula + ?Sized,
        T: Serialize<F>,
    {
        self.write_value(value, true)?;
        self.finish()
    }

    #[inline(always)]
    fn write_ref<F, T>(mut self, value: T) -> Result<(usize, usize), B::Error>
    where
        F: Formula + ?Sized,
        T: Serialize<F>,
    {
        let reference_size = reference_size::<F>();

        // Can we get promised sizes?
        let promised = <T as Serialize<F>>::size_hint(&value);

        let stack = match promised {
            None => self.write_ref_slow(value)?,
            Some(size) => match self.buffer.reserve_heap(self.heap, self.stack, size)? {
                None => self.write_ref_slow(value)?,
                Some(reserved) => {
                    let sub = IntoBufferedSerializer {
                        buffer: reserved,
                        heap: self.heap,
                    };

                    let Ok((heap, stack)) =
                            <T as Serialize<F>>::serialize::<BufferedSerializer<_>>(value, sub) else {
                                panic!("Failed to serialize a value into promised size");
                            };

                    self.heap += size;
                    assert_eq!(self.heap, heap + stack);
                    stack
                }
            },
        };

        // let stack = self.write_ref_slow(value)?;
        write_reference::<F, B>(stack, self.heap, self.heap, self.stack, &mut self.buffer)?;
        self.stack += reference_size;

        Ok((self.heap, self.stack))
    }

    #[inline(always)]
    fn finish(self) -> Result<(usize, usize), B::Error> {
        self.buffer.finish(self.heap, self.stack)?;
        Ok((self.heap, self.stack))
    }
}
