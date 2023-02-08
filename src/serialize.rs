use core::{
    any::type_name,
    convert::Infallible,
    mem::{replace, size_of},
};

use crate::{cold, err, formula::Formula, size::FixedUsize};

/// Trait for types that can be serialized
/// into raw bytes with specified `F: `[`Formula`].
///
/// Implementations *must* write data according to the formula.
/// Doing otherwise may result in errors during deserialization.
/// Where errors may be both failures to deserialize and
/// incorrect deserialized values.
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
    /// Implemtations *should not* override this method
    /// if going through `serialize` method is faster.
    ///
    /// Returning incorrect sizes may cause panic during implementation
    /// or broken data.
    #[inline(always)]
    fn fast_sizes(&self) -> Option<(usize, usize)> {
        if F::EXACT_SIZE {
            if let Some(size) = F::MAX_STACK_SIZE {
                return Some((0, size));
            }
        }
        None
    }

    /// Serializes `self` into given serializer.
    /// Uses `fast_sizes` to provide sizes to the serializer.
    ///
    /// This method can be overriden
    /// to prov
    #[inline(always)]
    fn serialize_with_sizes<I, S>(
        self,
        f: impl FnOnce(Option<(usize, usize)>) -> I,
    ) -> Result<S::Ok, S::Error>
    where
        Self: Sized,
        I: Into<S>,
        S: Serializer,
    {
        let ser = f(self.fast_sizes());
        self.serialize(ser)
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
    /// It avoids padding the value with zeros to `F::MAX_STACK_SIZE`.
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
        T: Serialize<F>;

    /// Writes padding bytes into serializer.
    /// Padding it automatically calculated.
    /// Only array serialization should use this method.
    fn write_pad(&mut self) -> Result<(), Self::Error>;

    /// Finish serialization.
    fn finish(self) -> Result<Self::Ok, Self::Error>;
}

struct IntoDrySerializer;

impl From<IntoDrySerializer> for DrySerializer {
    #[inline(always)]
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
    #[inline(always)]
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
                    Unsized serialized here <{} as Serialize<{}>",
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
    #[inline(always)]
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

    #[inline(always)]
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Infallible> {
        self.write_pad()?;
        self.stack += bytes.len();
        Ok(())
    }

    #[inline(always)]
    fn write_value<F, T>(&mut self, value: T) -> Result<(), Infallible>
    where
        F: Formula + ?Sized,
        T: Serialize<F>,
    {
        self.write_pad()?;
        let (heap, stack) = serialized_sizes::<F, T>(value);
        find_pad::<F, T>(stack, &mut self.pad);
        self.heap += heap;
        self.stack += stack;
        Ok(())
    }

    #[inline(always)]
    fn write_ref<F, T>(&mut self, value: T) -> Result<(), Infallible>
    where
        F: Formula + ?Sized,
        T: Serialize<F>,
    {
        self.write_pad()?;
        let (heap, stack) = serialized_sizes::<F, T>(value);
        check_stack::<F, T>(stack);
        self.heap += heap + stack;
        self.stack += size_of::<[FixedUsize; 2]>();
        Ok(())
    }

    #[inline(always)]
    fn write_pad(&mut self) -> Result<(), Infallible> {
        self.stack += self.pad.take();
        Ok(())
    }

    #[inline(always)]
    fn finish(self) -> Result<(usize, usize), Infallible> {
        Ok((self.heap, self.stack))
    }
}

struct IntoSerializer<'ser> {
    output: &'ser mut [u8],
    heap: usize,
}

impl<'ser> From<IntoSerializer<'ser>> for FailFastSerializer<'ser> {
    #[inline(always)]
    fn from(into: IntoSerializer<'ser>) -> Self {
        FailFastSerializer::new(into.heap, into.output)
    }
}

impl<'ser> From<IntoSerializer<'ser>> for ExactSizeSerializer<'ser> {
    #[inline(always)]
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
    #[must_use]
    #[inline(always)]
    fn new(heap: usize, output: &'ser mut [u8]) -> Self {
        FailFastSerializer {
            heap,
            stack: 0,
            pad: Pad::Sized(0),
            output,
        }
    }

    #[inline(always)]
    fn sub(&mut self) -> IntoSerializer {
        let at = self.output.len() - self.stack;
        IntoSerializer {
            output: &mut self.output[..at],
            heap: self.heap,
        }
    }
}

impl<'ser> Serializer for FailFastSerializer<'ser> {
    type Ok = (usize, usize);
    type Error = ();

    #[inline(always)]
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), ()> {
        self.write_pad()?;
        if self.output.len() - self.stack - self.heap < bytes.len() {
            return err(());
        }
        let at = self.output.len() - self.stack - bytes.len();
        self.output[at..].copy_from_slice(bytes);
        self.stack += bytes.len();
        Ok(())
    }

    #[inline(always)]
    fn write_value<F, T>(&mut self, value: T) -> Result<(), ()>
    where
        F: Formula + ?Sized,
        T: Serialize<F>,
    {
        self.write_pad()?;
        let (heap, stack) =
            <T as Serialize<F>>::serialize::<FailFastSerializer>(value, self.sub())?;

        find_pad::<F, T>(stack, &mut self.pad);
        self.heap = heap;
        self.stack += stack;
        Ok(())
    }

    #[inline(always)]
    fn write_ref<F, T>(&mut self, value: T) -> Result<(), ()>
    where
        F: Formula + ?Sized,
        T: Serialize<F>,
    {
        self.write_pad()?;
        let size;

        if let Some((heap, stack)) = <T as Serialize<F>>::fast_sizes(&value) {
            if self.output.len() - self.stack - self.heap < stack {
                return err(());
            }

            if self.output.len() - self.stack - self.heap - stack < heap {
                return err(());
            }

            let at = self.heap + heap + stack;
            let ser = IntoSerializer {
                output: &mut self.output[..at],
                heap: self.heap,
            };

            let (actual_heap, actual_stack) =
                <T as Serialize<F>>::serialize::<FailFastSerializer>(value, ser)?;

            debug_assert_eq!(self.heap + heap, actual_heap);
            debug_assert_eq!(stack, actual_stack);

            self.heap += heap + stack;
            size = stack;
        } else {
            let (heap, stack) =
                <T as Serialize<F>>::serialize::<FailFastSerializer>(value, self.sub())?;

            check_stack::<F, T>(stack);

            let end = self.output.len() - self.stack;
            to_heap(&mut self.output[..end], heap, stack);

            self.heap = heap + stack;
            size = stack;
        }

        let address = FixedUsize::truncate_unchecked(self.heap);
        let size = FixedUsize::truncate_unchecked(size);

        self.write_value::<[FixedUsize; 2], _>([address, size])
    }

    #[inline(always)]
    fn write_pad(&mut self) -> Result<(), ()> {
        let pad = self.pad.take();
        if self.output.len() - self.stack - self.heap < pad {
            return err(());
        }
        self.stack += pad;
        Ok(())
    }

    #[inline(always)]
    fn finish(self) -> Result<(usize, usize), ()> {
        Ok((self.heap, self.stack))
    }
}

/// Wraps output buffer and provides methods for serializing data.
/// Implementors of `Serialize` trait may use this type.
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
    #[must_use]
    #[inline(always)]
    fn new(heap: usize, output: &'ser mut [u8]) -> Self {
        ExactSizeSerializer {
            heap,
            stack: 0,
            pad: Pad::Sized(0),
            output: Some(output),
        }
    }

    #[inline(always)]
    fn sub_value<F, T>(&mut self, value: T) -> (usize, usize)
    where
        F: Formula + ?Sized,
        T: Serialize<F>,
    {
        match &mut self.output {
            None => {
                cold();
                match <T as Serialize<F>>::serialize::<DrySerializer>(value, IntoDrySerializer) {
                    Err(never) => match never {},
                    Ok((heap, stack)) => (heap, stack),
                }
            }
            Some(output) => {
                let at = output.len() - self.stack;
                match <T as Serialize<F>>::serialize::<ExactSizeSerializer>(
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

    #[inline(always)]
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), (usize, usize)> {
        self.write_pad()?;
        match &mut self.output {
            None => cold(),
            Some(output) => {
                if output.len() - self.stack - self.heap < bytes.len() {
                    cold();
                    self.output = None;
                } else {
                    let at = output.len() - self.stack - bytes.len();
                    output[at..].copy_from_slice(bytes);
                }
            }
        }
        self.stack += bytes.len();
        Ok(())
    }

    #[inline(always)]
    fn write_value<F, T>(&mut self, value: T) -> Result<(), (usize, usize)>
    where
        F: Formula + ?Sized,
        T: Serialize<F>,
    {
        self.write_pad()?;
        let (heap, stack) = self.sub_value::<F, T>(value);
        find_pad::<F, T>(stack, &mut self.pad);

        self.heap = heap;
        self.stack += stack;
        Ok(())
    }

    #[inline(always)]
    fn write_ref<F, T>(&mut self, value: T) -> Result<(), (usize, usize)>
    where
        F: Formula + ?Sized,
        T: Serialize<F>,
    {
        self.write_pad()?;
        let (heap, stack) = self.sub_value::<F, T>(value);
        check_stack::<F, T>(stack);

        match &mut self.output {
            None => cold(),
            Some(output) => {
                let end = output.len() - self.stack;
                to_heap(&mut output[..end], heap, stack);
            }
        }

        self.heap = heap + stack;

        let address = FixedUsize::truncate_unchecked(self.heap);
        let size = FixedUsize::truncate_unchecked(stack);

        self.write_value::<[FixedUsize; 2], _>([address, size])
    }

    #[inline(always)]
    fn write_pad(&mut self) -> Result<(), (usize, usize)> {
        let pad = self.pad.take();
        match &mut self.output {
            None => cold(),
            Some(output) => {
                if output.len() - self.stack - self.heap < pad {
                    cold();
                    self.output = None;
                }
            }
        }
        self.stack += pad;
        Ok(())
    }

    #[inline(always)]
    fn finish(self) -> Result<(usize, usize), (usize, usize)> {
        if self.output.is_none() {
            err((self.heap, self.stack))
        } else {
            Ok((self.heap, self.stack))
        }
    }
}

#[inline(always)]
pub fn serialize<F, T>(value: T, output: &mut [u8]) -> Result<usize, ()>
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    if output.len() < HEADER_SIZE {
        return err(());
    }

    let mut ser = FailFastSerializer::new(HEADER_SIZE, output);
    ser.write_value::<F, T>(value)?;
    let (heap, stack) = ser.finish()?;

    to_heap(output, heap, stack);

    let address = FixedUsize::truncate_unchecked(heap + stack);
    let size = FixedUsize::truncate_unchecked(stack);
    let mut ser = FailFastSerializer::new(0, &mut output[..HEADER_SIZE]);
    ser.write_value::<[FixedUsize; 2], _>([address, size])
        .unwrap();

    Ok(heap + stack)
}

#[inline(always)]
pub fn serialize_or_size<F, T>(value: T, output: &mut [u8]) -> Result<usize, usize>
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    if output.len() < HEADER_SIZE {
        return err(serialized_size::<F, T>(value));
    }

    let mut ser = ExactSizeSerializer::new(HEADER_SIZE, output);
    ser.write_value::<F, T>(value).unwrap();
    let (heap, stack) = match ser.finish() {
        Err((heap, stack)) => {
            return err(heap + stack);
        }
        Ok(sizes) => sizes,
    };

    to_heap(output, heap, stack);

    let address = FixedUsize::truncate_unchecked(heap + stack);
    let size = FixedUsize::truncate_unchecked(stack);
    let mut ser = FailFastSerializer::new(0, &mut output[..HEADER_SIZE]);
    ser.write_value::<[FixedUsize; 2], _>([address, size])
        .unwrap();

    Ok(heap + stack)
}

#[inline(always)]
fn serialized_sizes<F, T>(value: T) -> (usize, usize)
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    match <T as Serialize<F>>::serialize::<DrySerializer>(value, IntoDrySerializer) {
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
    let (heap, stack) = serialized_sizes::<F, T>(value);
    heap + stack + HEADER_SIZE
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

#[track_caller]
#[inline(always)]
fn find_pad<F, T>(stack: usize, pad: &mut Pad)
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    check_stack::<F, T>(stack);

    match pad {
        Pad::Sized(slot @ 0) => match F::MAX_STACK_SIZE {
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

/// Moves stack bytes to the heap
#[inline(always)]
fn to_heap(output: &mut [u8], heap: usize, stack: usize) {
    let len = output.len();
    // if len == heap + stack {
    //     return;
    // }
    // if len - stack >= heap + stack {
    //     let (head, tail) = output.split_at_mut(len - stack);
    //     head[heap..][..stack].copy_from_slice(tail);
    // } else {
    output.copy_within(len - stack.., heap);
    // }
}
