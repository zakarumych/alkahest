use core::{
    any::type_name,
    convert::Infallible,
    mem::{replace, size_of},
};

use crate::{
    formula::{Formula, NonRefFormula},
    size::FixedUsize,
};

pub trait SerializeOwned<F: NonRefFormula + ?Sized> {
    /// Serializes `self` into given serializer.
    fn serialize_owned<S>(self, serializer: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}

/// Trait for types that can be serialized
/// into raw bytes with specified `F: `[`NonRefFormula`].
///
/// Implementations *must* write data according to the formula.
/// Doing otherwise may result in errors during deserialization.
/// Where errors may be both failures to deserialize and
/// incorrect deserialized values.
pub trait Serialize<F: NonRefFormula + ?Sized>: SerializeOwned<F> {
    /// Serializes `self` into given serializer.
    fn serialize<S>(&self, serializer: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}

impl<F, T> Serialize<F> for &T
where
    F: NonRefFormula + ?Sized,
    for<'s> &'s T: SerializeOwned<F>,
{
    #[cfg_attr(feature = "inline-more", inline(always))]
    fn serialize<S>(&self, serializer: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        <&T as SerializeOwned<F>>::serialize_owned(self, serializer)
    }
}

impl<F, T> SerializeOwned<F> for &T
where
    F: NonRefFormula + ?Sized,
    T: Serialize<F>,
{
    #[cfg_attr(feature = "inline-more", inline(always))]
    fn serialize_owned<S>(self, serializer: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        <T as Serialize<F>>::serialize(self, serializer)
    }
}

/// Instances of this trait are provided to `SerializeOwned::serialize` method.
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
/// `SerializeOwned::serialize` implementation *must* call `Serializer::finish` or diverge.
pub trait Serializer {
    type Ok;
    type Error;

    /// Writes raw bytes into serializer.
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Self::Error>;

    /// Writes a value with specific formula into serializer.
    fn write_value<F, T>(&mut self, value: T) -> Result<(), Self::Error>
    where
        F: Formula + ?Sized,
        T: SerializeOwned<F::NonRef>;

    /// Writes a value with specific formula into serializer.
    /// It avoids padding the value with zeros to `F::MAX_SIZE`.
    /// Instead creates indirection and consumes few bytes to store
    /// address and size of serialized value.
    ///
    /// This method is used for any `Ref` formula.
    ///
    /// User should prefer wrapping their formulas with `Ref` instead
    /// of using this method manually to avoid potential mismatch in
    /// serialization and deserialization.
    fn write_ref<F, T>(&mut self, value: T) -> Result<(), Self::Error>
    where
        F: Formula + ?Sized,
        T: SerializeOwned<F::NonRef>;

    /// Writes padding bytes into serializer.
    /// Padding it automatically calculated.
    /// Only array serialization should use this method.
    fn write_pad(&mut self) -> Result<(), Self::Error>;

    /// Finish serialization.
    fn finish(self) -> Result<Self::Ok, Self::Error>;
}

struct IntoDrySerializer;

impl From<IntoDrySerializer> for DrySerializer {
    #[cfg_attr(feature = "inline-more", inline(always))]
    fn from(_: IntoDrySerializer) -> Self {
        DrySerializer::new()
    }
}

/// This type helps implementing padding in serializers.
/// Handles error case when unsized formula is not a tail.
enum Pad {
    Unsized {
        #[cfg(debug_assertions)]
        serialize: &'static str,
        #[cfg(debug_assertions)]
        formula: &'static str,
    },
    Sized(usize),
}

impl Pad {
    #[cfg_attr(feature = "inline-more", inline(always))]
    fn take(&mut self) -> usize {
        match self {
            #[cfg(not(debug_assertions))]
            Pad::Unsized { .. } => {
                panic!("Unsized formula should be the last one. Use `Ref` to break the chain.");
            }
            #[cfg(debug_assertions)]
            Pad::Unsized { serialize, formula } => {
                panic!(
                    "Unsized formula should be the last one. Use `Ref` to break the chain.
                    Unsized serialized here <{} as SerializeOwned<{}>",
                    serialize, formula
                );
            }
            Pad::Sized(pad) => replace(pad, 0),
        }
    }
}

struct DrySerializer {
    heap: usize,
    stack: usize,
    pad: Pad,
}

impl DrySerializer {
    #[cfg_attr(feature = "inline-more", inline(always))]
    #[must_use]
    const fn new() -> Self {
        Self {
            heap: 0,
            stack: 0,
            pad: Pad::Sized(0),
        }
    }
}

impl Serializer for DrySerializer {
    type Ok = (usize, usize);
    type Error = Infallible;

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        self.write_pad()?;
        self.stack += bytes.len();
        Ok(())
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn write_value<F, T>(&mut self, value: T) -> Result<(), Self::Error>
    where
        F: Formula + ?Sized,
        T: SerializeOwned<F::NonRef>,
    {
        self.write_pad()?;
        let (heap, stack) = serialized_sizes::<F, T>(value);
        find_pad::<F, T>(stack, &mut self.pad);
        self.heap += heap;
        self.stack += stack;
        Ok(())
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn write_ref<F, T>(&mut self, value: T) -> Result<(), Self::Error>
    where
        F: Formula + ?Sized,
        T: SerializeOwned<F::NonRef>,
    {
        self.write_pad()?;
        let (heap, stack) = serialized_sizes::<F, T>(value);
        check_stack::<F, T>(stack);
        self.heap += heap + stack;
        self.stack += size_of::<[FixedUsize; 2]>();
        Ok(())
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn write_pad(&mut self) -> Result<(), Infallible> {
        self.stack += self.pad.take();
        Ok(())
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn finish(self) -> Result<(usize, usize), Infallible> {
        Ok((self.heap, self.stack))
    }
}

struct IntoSerializer<'ser> {
    output: &'ser mut [u8],
    heap: usize,
}

impl<'ser> From<IntoSerializer<'ser>> for FailFastSerializer<'ser> {
    #[cfg_attr(feature = "inline-more", inline(always))]
    fn from(into: IntoSerializer<'ser>) -> Self {
        FailFastSerializer::new(into.heap, into.output)
    }
}

impl<'ser> From<IntoSerializer<'ser>> for ExactSizeSerializer<'ser> {
    #[cfg_attr(feature = "inline-more", inline(always))]
    fn from(into: IntoSerializer<'ser>) -> Self {
        ExactSizeSerializer::new(into.heap, into.output)
    }
}

/// Implementation of `Serializer` that returns error from all methods
/// when output buffer is exhausted.
/// This allows quickly return to the caller without traversing whole value.
/// But it will not provide exact buffer size required for serialization.
#[must_use]
struct FailFastSerializer<'ser> {
    /// Output buffer sub-slice usable for serialization.
    output: &'ser mut [u8],

    // size of the heap
    heap: usize,

    // start of the stack
    stack: usize,

    /// Padding to insert into stack before next value.
    pad: Pad,
}

impl<'ser> FailFastSerializer<'ser> {
    #[cfg_attr(feature = "inline-more", inline(always))]
    #[must_use]
    fn new(heap: usize, output: &'ser mut [u8]) -> Self {
        FailFastSerializer {
            heap,
            stack: 0,
            pad: Pad::Sized(0),
            output,
        }
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn sub_value<F, T>(&mut self, value: T) -> Result<(usize, usize), ()>
    where
        F: Formula + ?Sized,
        T: SerializeOwned<F::NonRef>,
    {
        let at = self.output.len() - self.stack;

        F::serialize::<T, FailFastSerializer>(
            value,
            IntoSerializer {
                output: &mut self.output[..at],
                heap: self.heap,
            },
        )
    }
}

impl<'ser> Serializer for FailFastSerializer<'ser> {
    type Ok = (usize, usize);
    type Error = ();

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), ()> {
        self.write_pad()?;
        if self.output.len() - self.stack - self.heap < bytes.len() {
            return Err(());
        }
        let at = self.output.len() - self.stack - bytes.len();
        self.output[at..].copy_from_slice(bytes);
        self.stack += bytes.len();
        Ok(())
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn write_value<F, T>(&mut self, value: T) -> Result<(), ()>
    where
        F: Formula + ?Sized,
        T: SerializeOwned<F::NonRef>,
    {
        self.write_pad()?;
        let (heap, stack) = self.sub_value::<F, T>(value)?;

        find_pad::<F, T>(stack, &mut self.pad);
        self.heap = heap;
        self.stack += stack;
        Ok(())
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn write_ref<F, T>(&mut self, value: T) -> Result<(), ()>
    where
        F: Formula + ?Sized,
        T: SerializeOwned<F::NonRef>,
    {
        self.write_pad()?;
        let (heap, stack) = self.sub_value::<F, T>(value)?;

        check_stack::<F, T>(stack);
        let end = self.output.len() - self.stack;
        let start = end - stack;
        self.output.copy_within(start..end, heap);
        self.heap = heap + stack;

        let address = FixedUsize::truncate_unchecked(self.heap);
        let size = FixedUsize::truncate_unchecked(stack);

        self.write_value::<[FixedUsize; 2], _>([address, size])
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn write_pad(&mut self) -> Result<(), ()> {
        let pad = self.pad.take();
        if self.output.len() - self.stack - self.heap < pad {
            return Err(());
        }
        self.stack += pad;
        Ok(())
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn finish(self) -> Result<(usize, usize), ()> {
        Ok((self.heap, self.stack))
    }
}

/// Wraps output buffer and provides methods for serializing data.
/// Implementors of `SerializeOwned` trait may use this type.
#[must_use]
struct ExactSizeSerializer<'ser> {
    /// Output buffer sub-slice usable for serialization.
    output: Option<&'ser mut [u8]>,

    // size of the heap
    heap: usize,

    // start of the stack
    stack: usize,

    /// Padding to insert into stack before next value.
    pad: Pad,
}

impl<'ser> ExactSizeSerializer<'ser> {
    #[cfg_attr(feature = "inline-more", inline(always))]
    #[must_use]
    fn new(heap: usize, output: &'ser mut [u8]) -> Self {
        ExactSizeSerializer {
            heap,
            stack: 0,
            pad: Pad::Sized(0),
            output: Some(output),
        }
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn sub_value<F, T>(&mut self, value: T) -> (usize, usize)
    where
        F: Formula + ?Sized,
        T: SerializeOwned<F::NonRef>,
    {
        match &mut self.output {
            None => match F::serialize::<T, DrySerializer>(value, IntoDrySerializer) {
                Err(never) => match never {},
                Ok((heap, stack)) => (heap, stack),
            },
            Some(output) => {
                let at = output.len() - self.stack;
                match F::serialize::<T, ExactSizeSerializer>(
                    value,
                    IntoSerializer {
                        output: &mut output[..at],
                        heap: self.heap,
                    },
                ) {
                    Err(sizes) => {
                        self.output = None;
                        sizes
                    }
                    Ok(sizes) => sizes,
                }
            }
        }
    }
}

impl<'ser> Serializer for ExactSizeSerializer<'ser> {
    type Ok = (usize, usize);
    type Error = (usize, usize);

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), (usize, usize)> {
        self.write_pad()?;
        if let Some(output) = &mut self.output {
            if output.len() - self.stack - self.heap < bytes.len() {
                self.output = None;
            } else {
                let at = output.len() - self.stack - bytes.len();
                output[at..].copy_from_slice(bytes);
            }
        }
        self.stack += bytes.len();
        Ok(())
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn write_value<F, T>(&mut self, value: T) -> Result<(), (usize, usize)>
    where
        F: Formula + ?Sized,
        T: SerializeOwned<F::NonRef>,
    {
        self.write_pad()?;
        let (heap, stack) = self.sub_value::<F, T>(value);
        find_pad::<F, T>(stack, &mut self.pad);

        self.heap = heap;
        self.stack += stack;
        Ok(())
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn write_ref<F, T>(&mut self, value: T) -> Result<(), (usize, usize)>
    where
        F: Formula + ?Sized,
        T: SerializeOwned<F::NonRef>,
    {
        self.write_pad()?;
        let (heap, stack) = self.sub_value::<F, T>(value);
        check_stack::<F, T>(stack);

        if let Some(output) = &mut self.output {
            let end = output.len() - self.stack;
            let start = end - stack;
            if start != heap {
                output.copy_within(start..end, heap);
            }
        }

        self.heap = heap + stack;

        let address = FixedUsize::truncate_unchecked(self.heap);
        let size = FixedUsize::truncate_unchecked(stack);

        self.write_value::<[FixedUsize; 2], _>([address, size])
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn write_pad(&mut self) -> Result<(), (usize, usize)> {
        let pad = self.pad.take();
        if let Some(output) = &mut self.output {
            if output.len() - self.stack - self.heap < pad {
                self.output = None;
            }
        }
        self.stack += pad;
        Ok(())
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn finish(self) -> Result<(usize, usize), (usize, usize)> {
        if self.output.is_none() {
            Err((self.heap, self.stack))
        } else {
            Ok((self.heap, self.stack))
        }
    }
}

#[inline]
pub fn serialize<F, T>(value: T, output: &mut [u8]) -> Result<usize, ()>
where
    F: Formula + ?Sized,
    T: SerializeOwned<F::NonRef>,
{
    if output.len() < HEADER_SIZE {
        return Err(());
    }

    let mut ser = FailFastSerializer::new(HEADER_SIZE, output);
    ser.write_value::<F, T>(value)?;
    let (heap, stack) = ser.finish()?;
    output.copy_within(output.len() - stack.., heap);

    let address = FixedUsize::truncate_unchecked(heap + stack);
    let size = FixedUsize::truncate_unchecked(stack);
    let mut ser = FailFastSerializer::new(0, &mut output[..HEADER_SIZE]);
    ser.write_value::<[FixedUsize; 2], _>([address, size])
        .unwrap();

    Ok(heap + stack)
}

#[inline]
pub fn serialize_or_size<F, T>(value: T, output: &mut [u8]) -> Result<usize, usize>
where
    F: Formula + ?Sized,
    T: SerializeOwned<F::NonRef>,
{
    if output.len() < HEADER_SIZE {
        return Err(serialized_size::<F, T>(value));
    }

    let mut ser = ExactSizeSerializer::new(HEADER_SIZE, output);
    ser.write_value::<F, T>(value).unwrap();
    let (heap, stack) = match ser.finish() {
        Err((heap, stack)) => return Err(heap + stack),
        Ok(sizes) => sizes,
    };
    output.copy_within(output.len() - stack.., heap);

    let address = FixedUsize::truncate_unchecked(heap + stack);
    let size = FixedUsize::truncate_unchecked(stack);
    let mut ser = FailFastSerializer::new(0, &mut output[..HEADER_SIZE]);
    ser.write_value::<[FixedUsize; 2], _>([address, size])
        .unwrap();

    Ok(heap + stack)
}

fn serialized_sizes<F, T>(value: T) -> (usize, usize)
where
    F: Formula + ?Sized,
    T: SerializeOwned<F::NonRef>,
{
    match F::serialize::<T, DrySerializer>(value, IntoDrySerializer) {
        Ok((heap, stack)) => (heap, stack),
        Err(never) => match never {},
    }
}

pub fn serialized_size<F, T>(value: T) -> usize
where
    F: Formula + ?Sized,
    T: SerializeOwned<F::NonRef>,
{
    let (heap, stack) = serialized_sizes::<F, T>(value);
    heap + stack + HEADER_SIZE
}

#[cfg_attr(feature = "inline-more", inline(always))]
#[track_caller]
fn check_stack<F, T>(stack: usize)
where
    F: Formula + ?Sized,
    T: SerializeOwned<F::NonRef>,
{
    if let Some(max_size) = F::MAX_SIZE {
        assert!(
            stack <= max_size,
            "Incorrect `<{} as SerializeOwned<{}>>` implementation. `stack` size is `{}` but must be at most `{}`",
            type_name::<T>(),
            type_name::<F>(),
            stack,
            max_size,
        )
    };
}

#[cfg_attr(feature = "inline-more", inline(always))]
#[track_caller]
fn find_pad<F, T>(stack: usize, pad: &mut Pad)
where
    F: Formula + ?Sized,
    T: SerializeOwned<F::NonRef>,
{
    check_stack::<F, T>(stack);

    match pad {
        Pad::Sized(slot @ 0) => match F::MAX_SIZE {
            Some(max_size) => *slot = max_size - stack,
            None => {
                *pad = Pad::Unsized {
                    #[cfg(debug_assertions)]
                    serialize: type_name::<T>(),
                    #[cfg(debug_assertions)]
                    formula: type_name::<F>(),
                }
            }
        },
        _ => unreachable!(),
    }
}

const FIELD_SIZE: usize = size_of::<FixedUsize>();
const HEADER_SIZE: usize = FIELD_SIZE * 2;
