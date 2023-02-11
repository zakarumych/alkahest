use core::{
    any::type_name,
    convert::Infallible,
    fmt,
    mem::{replace, size_of},
};

use crate::{
    cold::{cold, err},
    formula::Formula,
    private::BareFormula,
    size::FixedUsize,
};

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
    /// Implementations *should not* override this method
    /// if going through `serialize` method is faster.
    ///
    /// Returning incorrect sizes may cause panic during implementation
    /// or broken data.
    // #[inline(always)]
    fn fast_sizes(&self) -> Option<usize>;
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

    fn fast_sizes(&self) -> Option<usize> {
        <&'ser T as Serialize<F>>::fast_sizes(self)
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

    // /// Writes padding bytes into serializer.
    // /// Padding it automatically calculated.
    // /// Only array serialization should use this method.
    // fn write_pad(&mut self) -> Result<(), Self::Error>;

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

// /// This type helps implementing padding in serializers.
// /// Handles error case when unsized formula is not a tail.
// enum Pad {
//     Tail {
//         #[cfg(debug_assertions)]
//         serialize: &'static str,
//         #[cfg(debug_assertions)]
//         formula: &'static str,
//     },
//     Sized(usize),
// }

// impl Pad {
//     #[inline(always)]
//     fn take(&mut self) -> usize {
//         match self {
//             #[cfg(not(debug_assertions))]
//             Pad::Tail { .. } => {
//                 panic!("Tail formula should be the last one.");
//             }
//             #[cfg(debug_assertions)]
//             Pad::Tail { serialize, formula } => {
//                 panic!(
//                     "Tail formula should be the last one.
//                     Tail serialized here <{} as Serialize<{}>",
//                     serialize, formula
//                 );
//             }
//             Pad::Sized(pad) => replace(pad, 0),
//         }
//     }
// }

struct DrySerializer {
    heap: usize,
    stack: usize,
    // pad: Pad,
}

impl DrySerializer {
    #[inline(always)]
    #[must_use]
    const fn new() -> Self {
        Self {
            heap: 0,
            stack: 0,
            // pad: Pad::Sized(0),
        }
    }

    #[inline(always)]
    fn write_value<F, T>(&mut self, value: T, last: bool)
    where
        F: Formula + ?Sized,
        T: Serialize<F>,
    {
        let (heap, stack) = serialized_sizes::<F, T>(value);

        match (last, F::MAX_STACK_SIZE) {
            (false, Some(max_stack)) => {
                debug_assert!(stack <= max_stack);
                self.stack += max_stack;
            }
            (false, None) => {
                self.stack += size_of::<FixedUsize>() + stack;
            }
            (true, Some(max_stack)) => {
                debug_assert!(stack <= max_stack);
                self.stack += stack;
            }
            (true, None) => {
                self.stack += stack;
            }
        }

        self.heap += heap;
    }
}

impl Serializer for DrySerializer {
    type Ok = (usize, usize);
    type Error = Infallible;

    #[inline(always)]
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Infallible> {
        self.stack += bytes.len();
        Ok(())
    }

    #[inline(always)]
    fn write_value<F, T>(&mut self, value: T) -> Result<(), Infallible>
    where
        F: Formula + ?Sized,
        T: Serialize<F>,
    {
        self.write_value::<F, T>(value, false);
        Ok(())
    }

    #[inline(always)]
    fn write_last_value<F, T>(mut self, value: T) -> Result<(usize, usize), Infallible>
    where
        F: Formula + ?Sized,
        T: Serialize<F>,
    {
        self.write_value::<F, T>(value, true);
        Ok((self.heap, self.stack))
    }

    #[inline(always)]
    fn write_ref<F, T>(mut self, value: T) -> Result<(usize, usize), Infallible>
    where
        F: Formula + ?Sized,
        T: Serialize<F>,
    {
        let (heap, stack) = serialized_sizes::<F, T>(value);
        check_stack::<F, T>(stack);
        self.heap += heap + stack;
        self.stack += size_of::<[FixedUsize; 2]>();
        Ok((self.heap, self.stack))
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
}

impl<'ser> FailFastSerializer<'ser> {
    #[must_use]
    #[inline(always)]
    fn new(heap: usize, output: &'ser mut [u8]) -> Self {
        FailFastSerializer {
            heap,
            stack: 0,
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

    #[inline(always)]
    fn write_value<F, T>(&mut self, value: T, last: bool) -> Result<(), ()>
    where
        F: Formula + ?Sized,
        T: Serialize<F>,
    {
        if !last && F::MAX_STACK_SIZE.is_none() {
            if self.output.len() - self.heap - self.stack < size_of::<FixedUsize>() {
                return err(());
            }
            self.stack += size_of::<FixedUsize>();
        }

        let (heap, stack) =
            <T as Serialize<F>>::serialize::<FailFastSerializer>(value, self.sub())?;

        match (last, F::MAX_STACK_SIZE) {
            (true, None) => {
                self.stack += stack;
            }
            (true, Some(max_stack)) => {
                debug_assert!(stack <= max_stack);
                self.stack += stack;
            }
            (false, None) => {
                let at = self.output.len() - self.stack;
                let size = FixedUsize::truncate_unchecked(stack);
                self.output[at..][..size_of::<FixedUsize>()].copy_from_slice(&size.to_le_bytes());
                self.stack += stack;
            }
            (false, Some(max_stack)) => {
                debug_assert!(stack <= max_stack);
                self.stack += max_stack;
            }
        }

        self.heap = heap;
        Ok(())
    }
}

impl<'ser> Serializer for FailFastSerializer<'ser> {
    type Ok = (usize, usize);
    type Error = ();

    #[inline(always)]
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), ()> {
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
        self.write_value(value, false)
    }

    #[inline(always)]
    fn write_last_value<F, T>(mut self, value: T) -> Result<(usize, usize), ()>
    where
        F: Formula + ?Sized,
        T: Serialize<F>,
    {
        self.write_value(value, true)?;
        self.finish()
    }

    #[inline(always)]
    fn write_ref<F, T>(mut self, value: T) -> Result<(usize, usize), ()>
    where
        F: Formula + ?Sized,
        T: Serialize<F>,
    {
        if self.output.len() - self.heap - self.stack < size_of::<[FixedUsize; 2]>() {
            return err(());
        }

        // Can we get promised sizes?
        let promised = <T as Serialize<F>>::fast_sizes(&value);

        let ref_start = self.output.len() - self.stack - size_of::<[FixedUsize; 2]>();
        let at = match promised {
            None => ref_start,
            Some(size) => {
                // How slow this is?
                if ref_start - self.heap < size {
                    return err(());
                }

                self.heap + size
            }
        };

        if self.output.len() - self.stack < at {
            return err(());
        }

        let (actual_heap, actual_stack) = <T as Serialize<F>>::serialize::<FailFastSerializer>(
            value,
            IntoSerializer {
                output: &mut self.output[..at],
                heap: self.heap,
            },
        )?;

        if let Some(size) = promised {
            debug_assert_eq!(
                self.heap + size,
                actual_heap + actual_stack,
                "<{} as Serialize<{}>>::fast_size() result is inaccurate",
                type_name::<T>(),
                type_name::<F>()
            );
        } else {
            check_stack::<F, T>(actual_stack);

            // let end = self.output.len() - self.stack;
            to_heap(&mut self.output[..at], actual_heap, actual_stack);
        }

        self.heap = actual_heap + actual_stack;
        // let address = FixedUsize::truncate_unchecked(self.heap);
        // let size = FixedUsize::truncate_unchecked(actual_stack);
        // self.write_value::<[FixedUsize; 2], _>([address, size], true)

        self.stack += size_of::<[FixedUsize; 2]>();
        let ref_slice = &mut self.output[ref_start..][..size_of::<[FixedUsize; 2]>()];
        let (size_slice, address_slice) = ref_slice.split_at_mut(size_of::<FixedUsize>());
        address_slice.copy_from_slice(&FixedUsize::truncate_unchecked(self.heap).to_le_bytes());
        size_slice.copy_from_slice(&FixedUsize::truncate_unchecked(actual_stack).to_le_bytes());
        Ok((self.heap, self.stack))
    }

    // #[inline(always)]
    // fn write_ref<F, T>(&mut self, value: T) -> Result<(), ()>
    // where
    //     F: Formula + ?Sized,
    //     T: Serialize<F>,
    // {
    //     let promised = <T as Serialize<F>>::fast_sizes(&value);
    //     self.write_pad()?;

    //     let at = self.output.len() - self.stack;

    //     let (heap, stack) = <T as Serialize<F>>::serialize::<FailFastSerializer>(
    //         value,
    //         IntoSerializer {
    //             output: &mut self.output[..at],
    //             heap: self.heap,
    //         },
    //     )?;

    //     if let Some(size) = promised {
    //         assert_eq!(self.heap + size, heap + stack);
    //     }

    //     check_stack::<F, T>(stack);

    //     let end = self.output.len() - self.stack;
    //     to_heap(&mut self.output[..end], heap, stack);

    //     self.heap = heap + stack;
    //     let address = FixedUsize::truncate_unchecked(self.heap);
    //     let size = FixedUsize::truncate_unchecked(stack);

    //     self.write_value::<[FixedUsize; 2], _>([address, size])
    // }

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
}

impl<'ser> ExactSizeSerializer<'ser> {
    #[must_use]
    #[inline(always)]
    fn new(heap: usize, output: &'ser mut [u8]) -> Self {
        ExactSizeSerializer {
            heap,
            stack: 0,
            output: Some(output),
        }
    }

    // #[inline(always)]
    // fn sub_value<F, T>(&mut self, value: T) -> (usize, usize)
    // where
    //     F: Formula + ?Sized,
    //     T: Serialize<F>,
    // {
    //     match &mut self.output {
    //         None => {
    //             cold();
    //             match <T as Serialize<F>>::serialize::<DrySerializer>(value, IntoDrySerializer) {
    //                 Err(never) => match never {},
    //                 Ok((heap, stack)) => (heap, stack),
    //             }
    //         }
    //         Some(output) => {
    //             let at = output.len() - self.stack;
    //             match <T as Serialize<F>>::serialize::<ExactSizeSerializer>(
    //                 value,
    //                 IntoSerializer {
    //                     output: &mut output[..at],
    //                     heap: self.heap,
    //                 },
    //             ) {
    //                 Err(sizes) => {
    //                     self.output = None;
    //                     sizes
    //                 }
    //                 Ok(sizes) => sizes,
    //             }
    //         }
    //     }
    // }

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

    #[inline(always)]
    fn write_value<F, T>(&mut self, value: T, last: bool) -> Result<(), (usize, usize)>
    where
        F: Formula + ?Sized,
        T: Serialize<F>,
    {
        if !last && F::MAX_STACK_SIZE.is_none() {
            if let Some(output) = &self.output {
                if output.len() - self.heap - self.stack < size_of::<FixedUsize>() {
                    self.output = None;
                }
            }
            self.stack += size_of::<FixedUsize>();
        }

        let (heap, stack) = self.sub_value::<F, T>(value);

        match (last, F::MAX_STACK_SIZE) {
            (true, None) => {
                self.stack += stack;
            }
            (true, Some(max_stack)) => {
                debug_assert!(stack <= max_stack);
                self.stack += stack;
            }
            (false, None) => {
                if let Some(output) = &mut self.output {
                    let at = output.len() - self.stack;
                    let size = FixedUsize::truncate_unchecked(stack);
                    output[at..][..size_of::<FixedUsize>()].copy_from_slice(&size.to_le_bytes());
                }
                self.stack += stack;
            }
            (false, Some(max_stack)) => {
                debug_assert!(stack <= max_stack);
                self.stack += max_stack;
            }
        }

        self.heap = heap;
        self.stack += stack;
        Ok(())
    }
}

impl<'ser> Serializer for ExactSizeSerializer<'ser> {
    type Ok = (usize, usize);
    type Error = (usize, usize);

    #[inline(always)]
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), (usize, usize)> {
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
        self.write_value::<F, T>(value, false)
    }

    #[inline(always)]
    fn write_last_value<F, T>(mut self, value: T) -> Result<(usize, usize), (usize, usize)>
    where
        F: Formula + ?Sized,
        T: Serialize<F>,
    {
        self.write_value::<F, T>(value, false)?;
        self.finish()
    }

    #[inline(always)]
    fn write_ref<F, T>(mut self, value: T) -> Result<(usize, usize), (usize, usize)>
    where
        F: Formula + ?Sized,
        T: Serialize<F>,
    {
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

        self.write_value::<[FixedUsize; 2], _>([address, size], false)?;
        self.finish()
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

/// Error that may occur during serialization,
/// if buffer is too small to fit serialized data.
///
/// This type does not contain the size of the buffer required to fit serialized data.
/// To get the size use `serialize_or_size` function that returns `Result<usize, BufferSizeRequired>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BufferExhausted;

impl fmt::Display for BufferExhausted {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "buffer exhausted")
    }
}

#[inline]
pub fn serialize<F, T>(value: T, output: &mut [u8]) -> Result<usize, BufferExhausted>
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    if output.len() < HEADER_SIZE {
        return err(BufferExhausted);
    }

    // Can we get promised sizes?
    let promised = <T as Serialize<F>>::fast_sizes(&value);

    let at = match promised {
        None => output.len(),
        Some(size) => HEADER_SIZE + size,
    };

    let (actual_heap, actual_stack) = <T as Serialize<F>>::serialize::<FailFastSerializer>(
        value,
        IntoSerializer {
            output: &mut output[..at],
            heap: HEADER_SIZE,
        },
    )
    .map_err(|()| BufferExhausted)?;

    if let Some(size) = promised {
        debug_assert_eq!(
            HEADER_SIZE + size,
            actual_heap + actual_stack,
            "<{} as Serialize<{}>>::fast_size() result is inaccurate",
            type_name::<T>(),
            type_name::<F>()
        );
    } else {
        check_stack::<F, T>(actual_stack);
        to_heap(output, actual_heap, actual_stack);
    }

    let address = FixedUsize::truncate_unchecked(actual_heap + actual_stack);
    let size = FixedUsize::truncate_unchecked(actual_stack);

    let mut ser = FailFastSerializer::new(0, &mut output[..HEADER_SIZE]);
    ser.write_value::<[FixedUsize; 2], _>([address, size], false)
        .unwrap();

    Ok(actual_heap + actual_stack)
}

// #[inline]
// pub fn serialize<F, T>(value: T, output: &mut [u8]) -> Result<usize, BufferExhausted>
// where
//     F: Formula + ?Sized,
//     T: Serialize<F>,
// {
//     if output.len() < HEADER_SIZE {
//         return err(BufferExhausted);
//     }

//     let (heap, stack) = <T as Serialize<F>>::serialize::<FailFastSerializer>(
//         value,
//         IntoSerializer {
//             output: output,
//             heap: HEADER_SIZE,
//         },
//     )
//     .map_err(|()| BufferExhausted)?;

//     check_stack::<F, T>(stack);
//     to_heap(output, heap, stack);

//     let address = FixedUsize::truncate_unchecked(heap + stack);
//     let size = FixedUsize::truncate_unchecked(stack);

//     let mut ser = FailFastSerializer::new(0, &mut output[..HEADER_SIZE]);
//     ser.write_value::<[FixedUsize; 2], _>([address, size])
//         .unwrap();

//     Ok(heap + stack)
// }

/// Error that may occur during serialization,
/// if buffer is too small to fit serialized data.
///
/// Contains the size of the buffer required to fit serialized data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct BufferSizeRequired {
    pub required: usize,
}

impl fmt::Display for BufferSizeRequired {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "buffer size required: {}", self.required)
    }
}

#[inline]
pub fn serialize_or_size<F, T>(value: T, output: &mut [u8]) -> Result<usize, BufferSizeRequired>
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    if output.len() < HEADER_SIZE {
        return err(BufferSizeRequired {
            required: serialized_size::<F, _>(value),
        });
    }

    let mut ser = ExactSizeSerializer::new(HEADER_SIZE, output);
    ser.write_value::<F, _>(value, true).unwrap();
    let (heap, stack) = match ser.finish() {
        Err((heap, stack)) => {
            return err(BufferSizeRequired {
                required: heap + stack,
            });
        }
        Ok(sizes) => sizes,
    };

    to_heap(output, heap, stack);

    let address = FixedUsize::truncate_unchecked(heap + stack);
    let size = FixedUsize::truncate_unchecked(stack);
    let mut ser = FailFastSerializer::new(0, &mut output[..HEADER_SIZE]);
    ser.write_value::<[FixedUsize; 2], _>([address, size], false)
        .unwrap();

    Ok(heap + stack)
}

#[inline(always)]
fn serialized_sizes<F, T>(value: T) -> (usize, usize)
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    match Serialize::<F>::serialize::<DrySerializer>(value, IntoDrySerializer) {
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

const FIELD_SIZE: usize = size_of::<FixedUsize>();
const HEADER_SIZE: usize = FIELD_SIZE * 2;

/// Moves stack bytes to the heap
#[inline(always)]
fn to_heap(output: &mut [u8], heap: usize, stack: usize) {
    let len = output.len();
    if len == heap + stack {
        return;
    }
    // if len - stack >= heap + stack {
    //     let (head, tail) = output.split_at_mut(len - stack);
    //     head[heap..][..stack].copy_from_slice(tail);
    // } else {
    output.copy_within(len - stack.., heap);
    // }
}
