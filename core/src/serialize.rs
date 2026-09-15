use core::{fmt, ops};

use crate::{
    Element,
    buffer::{Buffer, BufferExhausted, CheckedFixedBuffer, DryBuffer, MaybeFixedBuffer},
    element::{heap_size, stack_size},
    formula::{Formula, SizeBound},
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
        Sizes {
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

impl ops::Sub for Sizes {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        Sizes {
            heap: self.heap - rhs.heap,
            stack: self.stack - rhs.stack,
        }
    }
}

/// Trait for types that can be serialized with specified `F: `[`Formula`].
///
/// Implementations *must* ensure that serialized data conforms to formula layout.
///
/// # Examples
///
/// ```
/// # use alkahest::{*, advanced::*};
///
/// struct ThreeBytes;
///
/// impl Formula for ThreeBytes {
///     const EXACT_SIZE: bool = true;
///     const HEAPLESS: bool = true;
///     fn max_stack_size(_size_bytes: u8) -> Option<usize> { Some(3) }
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
pub trait Serialize<F: ?Sized> {
    /// Serializes `self` into the given buffer.
    ///
    /// # Errors
    ///
    /// Returns error if buffer write fails.
    fn serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: Serializer;

    /// Returns heap and stack sizes required to serialize `self` according to formula `F`.
    ///
    /// Returns both stack and heap sizes required if any.
    ///
    /// This function may conservatively return `None` and it won't affect final serialization layout.
    /// However if sizes are known ahead of time, returning them may improve serialization performance.
    ///
    /// Returning incorrect sizes may lead to corrupted serialization or panics.
    #[inline(always)]
    fn size_hint<const SIZE_BYTES: usize>(&self) -> Option<Sizes> {
        None
    }
}

impl<'a, F, T> Serialize<F> for &'a T
where
    F: ?Sized,
    T: Serialize<F> + ?Sized,
{
    #[inline(always)]
    fn serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        <T as Serialize<F>>::serialize(&**self, serializer)
    }

    #[inline(always)]
    fn size_hint<const SIZE_BYTES: usize>(&self) -> Option<Sizes> {
        <T as Serialize<F>>::size_hint::<SIZE_BYTES>(&**self)
    }
}

pub trait Serializer {
    type Error;

    /// Serializes a slice of raw bytes.
    ///
    /// This is a low-level method used in [`Serialize::serialize`](Serialize::serialize) implementation.
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Self::Error>;

    /// Specializes a `usize` value.
    ///
    /// This is a low-level method used in [`Serialize::serialize`](Serialize::serialize) implementation.
    /// It is used when sizes and addresses are serialized.
    ///
    /// If size or address can't fit into defined number of bytes, implementation should return an error.
    fn write_usize(&mut self, value: usize) -> Result<(), Self::Error>;

    /// Serializes an element.
    ///
    /// This is higher-level method used in [`Serialize::serialize`](Serialize::serialize) implementation of composite types.
    /// It is used when serializing fields of records, tuples, or elements of slices.
    ///
    /// Unlike `write_indirect`, this method serializes the value directly into the "stack" space.
    fn write_direct<F, T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        F: Formula + ?Sized,
        T: Serialize<F> + ?Sized;

    /// Serializes an element.
    ///
    /// This is higher-level method used in [`Serialize::serialize`](Serialize::serialize) implementation of composite types.
    /// It is used when serializing fields of records, tuples, or elements of slices.
    ///
    /// Unlike `write_direct`, this method serializes the value into "heap" and writes only an address to the "stack" space.
    fn write_indirect<E, T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        E: Element + ?Sized,
        T: Serialize<E::Formula> + ?Sized;

    /// Reserve space for one `usize` and return its address.
    fn reserve_usize(&mut self) -> Result<usize, Self::Error>;

    /// Write one `usize` to address previously obtained via [`Serializer::reserve_usize`]
    fn write_reserved_usize(&mut self, address: usize, value: usize);
}

pub(crate) struct SerialzierImpl<'a, B: Buffer, const SIZE_BYTES: usize> {
    sizes: &'a mut Sizes,
    buffer: B,

    // Number of bytes of padding to add before next element.
    // It is set when writing direct elements with actual size less than formula's max stack size.
    pad_next: usize,
}

impl<'a, B, const SIZE_BYTES: usize> SerialzierImpl<'a, B, SIZE_BYTES>
where
    B: Buffer,
{
    #[inline(always)]
    fn new(sizes: &'a mut Sizes, buffer: B) -> Self {
        SerialzierImpl {
            sizes,
            buffer,
            pad_next: 0,
        }
    }

    #[inline(never)] // This is sad-path, so we put it on separate function to avoid bloating the main serialization logic.
    fn write_to_heap<E, T>(&mut self, value: &T) -> Result<(), B::Error>
    where
        E: Element + ?Sized,
        T: Serialize<E::Formula> + ?Sized,
    {
        let old_stack = self.sizes.stack;
        debug_assert_eq!(self.pad_next, 0);

        simple_try!(E::serialize(value, self));

        let len = self.sizes.stack - old_stack;

        self.buffer
            .move_to_heap(self.sizes.heap, self.sizes.stack, len);

        self.sizes.heap += len;
        self.sizes.stack = old_stack;
        self.pad_next = 0;

        Ok(())
    }
}

impl<'a, B, const SIZE_BYTES: usize> Serializer for SerialzierImpl<'a, B, SIZE_BYTES>
where
    B: Buffer,
{
    type Error = B::Error;

    /// Write raw bytes to the buffer.
    ///
    /// Use in [`Serialize::serialize`](Serialize::serialize) implementation.
    ///
    /// # Errors
    ///
    /// Returns error if buffer write fails.
    #[inline(always)]
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        let mut reserved = simple_try!(self.buffer.reserve(
            self.sizes.heap,
            self.sizes.stack,
            self.pad_next + bytes.len(),
        ));

        self.sizes.stack += self.pad_next;
        self.pad_next = 0;

        reserved.write_stack(self.sizes.stack, bytes);

        self.sizes.stack += bytes.len();

        Ok(())
    }

    /// Specialized method to write usize value in `SIZE_BYTES` bytes.
    #[inline(always)]
    fn write_usize(&mut self, value: usize) -> Result<(), Self::Error> {
        let reserved = simple_try!(self.buffer.reserve(
            self.sizes.heap,
            self.sizes.stack,
            self.pad_next + SIZE_BYTES,
        ));

        self.sizes.stack += self.pad_next;
        self.pad_next = 0;

        write_usize::<_, SIZE_BYTES>(value, self.sizes.stack, reserved);

        self.sizes.stack += SIZE_BYTES;

        Ok(())
    }

    /// Writes field value into the buffer.
    ///
    /// Use in [`Serialize::serialize`](Serialize::serialize) implementation.
    ///
    /// # Errors
    ///
    /// Returns error if buffer write fails.
    #[inline]
    fn write_direct<F, T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        F: Formula + ?Sized,
        T: Serialize<F> + ?Sized,
    {
        assert!(F::INHABITED);

        let _is_zero = const {
            if let SizeBound::Exact(0) | SizeBound::Bounded(0) = stack_size::<F, SIZE_BYTES>() {
                debug_assert!(matches!(
                    heap_size::<F, SIZE_BYTES>(),
                    SizeBound::Exact(0) | SizeBound::Bounded(0)
                ));

                true
            } else {
                false
            }
        };

        #[cfg(not(debug_assertions))]
        if _is_zero {
            simple_try!(
                self.buffer
                    .reserve(self.sizes.heap, self.sizes.stack, self.pad_next)
            );

            self.sizes.stack += self.pad_next;
            self.pad_next = 0;

            // No need to serialize zero-sized value.
            // In release builds we simply skip serialization.
            return Ok(());
        }

        let old_sizes;

        if !B::RESERVED_IS_SELF
            && let Some(sizes) = const { const_size_hint::<F, T, SIZE_BYTES>() }
        {
            // If size is known reserve.

            let reserved = simple_try!(self.buffer.reserve(
                self.sizes.heap,
                self.sizes.stack,
                sizes.stack + sizes.heap + self.pad_next,
            ));

            self.sizes.stack += self.pad_next;
            self.pad_next = 0;

            old_sizes = *self.sizes;

            let serializer = SerialzierImpl::<_, SIZE_BYTES>::new(&mut self.sizes, reserved);
            if let Err(err) = <T as Serialize<F>>::serialize(value, serializer) {
                match err {}
            }
        } else {
            simple_try!(
                self.buffer
                    .reserve(self.sizes.heap, self.sizes.stack, self.pad_next)
            );
            self.sizes.stack += self.pad_next;
            self.pad_next = 0;

            old_sizes = *self.sizes;

            simple_try!(<T as Serialize<F>>::serialize(
                value,
                SerialzierImpl::<_, SIZE_BYTES>::new(self.sizes, self.buffer.reborrow())
            ));
        }

        let actual_sizes = *self.sizes - old_sizes;

        match const { stack_size::<F, SIZE_BYTES>() } {
            SizeBound::Unbounded => {
                // It is impossible to write padding for unbounded element.
                // Thus we fence it with too large padding that will cause next write to fail.
                self.pad_next = usize::MAX;
            }
            SizeBound::Bounded(max_stack) => {
                debug_assert!(
                    actual_sizes.stack <= max_stack,
                    "{} <= {}",
                    actual_sizes.stack,
                    max_stack
                );
                self.pad_next = max_stack - actual_sizes.stack;
            }
            SizeBound::Exact(exact_stack) => {
                // This branch can be chosen at compile time,
                // so we simply avoid simple calculation of the branch above.
                debug_assert_eq!(actual_sizes.stack, exact_stack);
            }
        }

        match const { heap_size::<F, SIZE_BYTES>() } {
            SizeBound::Unbounded => {}
            SizeBound::Bounded(max_heap) => {
                debug_assert!(actual_sizes.heap <= max_heap);
            }
            SizeBound::Exact(exact_heap) => {
                // This branch can be chosen at compile time,
                // so we simply avoid simple calculation of the branch above.
                debug_assert_eq!(actual_sizes.heap, exact_heap);
            }
        }

        Ok(())
    }

    /// Write value to the buffer as a reference,
    /// placing value into the heap and reference into the stack.
    ///
    /// Use in [`Serialize::serialize`](Serialize::serialize) implementation.
    ///
    /// # Errors
    ///
    /// Returns error if buffer write fails.
    #[inline]
    fn write_indirect<E, T>(&mut self, value: &T) -> Result<(), B::Error>
    where
        E: Element + ?Sized,
        T: Serialize<E::Formula> + ?Sized,
    {
        assert!(E::INHABITED);

        // Can we get size hint for the value?
        match size_hint::<E, T, SIZE_BYTES>(&value) {
            None => {
                // Size hint is unobtainable, serialize to heap through stack and move to heap.
                simple_try!(self.write_to_heap::<E, T>(value));
            }
            Some(promised) => {
                // Reserive heap space to avoid serializing to stack and moving to heap.
                let reserved = simple_try!(self.buffer.reserve_heap(
                    self.sizes.heap,
                    self.sizes.stack,
                    promised.total(),
                ));

                let mut sizes = Sizes {
                    heap: self.sizes.heap,
                    stack: 0,
                };

                {
                    let mut serializer = SerialzierImpl::<_, SIZE_BYTES>::new(&mut sizes, reserved);
                    if let Err(err) = E::serialize(value, &mut serializer) {
                        match err {}
                    }
                }

                debug_assert_eq!(
                    sizes.heap,
                    self.sizes.heap + promised.heap,
                    "Serialization used more heap than promised by `Serialize::size_hint`"
                );
                debug_assert_eq!(
                    sizes.stack, promised.stack,
                    "Serialization used more stack than promised by `Serialize::size_hint`"
                );

                // Flush reserved memory to heap.
                self.sizes.heap = sizes.total();
            }
        }

        let address = self.sizes.heap;
        self.write_usize(address)
    }

    #[inline(always)]
    fn reserve_usize(&mut self) -> Result<usize, Self::Error> {
        simple_try!(self.buffer.reserve(
            self.sizes.heap,
            self.sizes.stack,
            self.pad_next + SIZE_BYTES,
        ));

        self.sizes.stack += self.pad_next;
        self.pad_next = 0;

        let reserved = self.sizes.stack;

        self.sizes.stack += SIZE_BYTES;

        Ok(reserved)
    }

    #[inline(always)]
    fn write_reserved_usize(&mut self, address: usize, value: usize) {
        write_usize::<_, SIZE_BYTES>(value, address, self.buffer.reborrow())
    }
}

/// Specialized method to write usize value in `SIZE_BYTES` bytes.
#[inline(always)]
pub fn write_usize<B, const SIZE_BYTES: usize>(value: usize, stack: usize, mut buffer: B)
where
    B: Buffer,
{
    const {
        assert!(SIZE_BYTES > 0 && SIZE_BYTES <= 16);
    }

    const LEN: usize = size_of::<usize>();

    match () {
        () if SIZE_BYTES < LEN => {
            let max_size = 1usize << (SIZE_BYTES * 8);
            assert!(
                value < max_size,
                "Value too large to fit in SIZE_BYTES bytes ({SIZE_BYTES})"
            );
            let bytes = value.to_le_bytes();
            buffer.write_stack(stack, &bytes[..SIZE_BYTES])
        }
        () if SIZE_BYTES > LEN => {
            let mut bytes = [0u8; SIZE_BYTES];
            bytes[..LEN].copy_from_slice(&value.to_le_bytes());
            buffer.write_stack(stack, &bytes)
        }
        () => {
            // SIZE_BYTES == LEN
            buffer.write_stack(stack, &value.to_le_bytes())
        }
    }
}

#[inline(always)]
pub const fn const_size_hint<
    E: Element + ?Sized,
    T: Serialize<E::Formula> + ?Sized,
    const SIZE_BYTES: usize,
>() -> Option<Sizes> {
    match const { (stack_size::<E, SIZE_BYTES>(), heap_size::<E, SIZE_BYTES>()) } {
        (SizeBound::Exact(stack), SizeBound::Exact(heap)) => Some(Sizes { stack, heap }),
        _ => None,
    }
}

#[inline(always)]
pub fn size_hint<
    E: Element + ?Sized,
    T: Serialize<E::Formula> + ?Sized,
    const SIZE_BYTES: usize,
>(
    value: &T,
) -> Option<Sizes> {
    match const { (stack_size::<E, SIZE_BYTES>(), heap_size::<E, SIZE_BYTES>()) } {
        (SizeBound::Exact(stack), SizeBound::Exact(heap)) => Some(Sizes { stack, heap }),
        _ => E::size_hint::<T, SIZE_BYTES>(value),
    }
}

#[inline(always)]
pub fn size_hint_padded<
    E: Element + ?Sized,
    T: Serialize<E::Formula> + ?Sized,
    const SIZE_BYTES: usize,
>(
    value: &T,
) -> Option<Sizes> {
    match const { (stack_size::<E, SIZE_BYTES>(), heap_size::<E, SIZE_BYTES>()) } {
        (SizeBound::Bounded(stack) | SizeBound::Exact(stack), SizeBound::Exact(heap)) => {
            Some(Sizes { stack, heap })
        }
        (SizeBound::Bounded(stack) | SizeBound::Exact(stack), _) => Some(Sizes {
            stack,
            heap: simple_some!(E::size_hint::<T, SIZE_BYTES>(value)).heap,
        }),
        (SizeBound::Unbounded, _) => E::size_hint::<T, SIZE_BYTES>(value),
    }
}

#[inline(always)]
pub fn make_serializer<'a, B, const SIZE_BYTES: usize>(
    buffer: B,
    sizes: &'a mut Sizes,
) -> impl Serializer<Error = B::Error> + use<'a, B, SIZE_BYTES>
where
    B: Buffer,
{
    SerialzierImpl::<B, SIZE_BYTES>::new(sizes, buffer)
}

/// Serializes value into buffer.
/// Returns total number of bytes written and size of the root value.
/// The buffer type controls bytes writing and failing strategy.
pub fn serialize_into<E, T, B, const SIZE_BYTES: usize>(
    value: &T,
    mut buffer: B,
) -> Result<usize, B::Error>
where
    E: Element + ?Sized,
    T: Serialize<E::Formula>,
    B: Buffer,
{
    const {
        assert!(E::INHABITED);
    }

    match size_hint::<E, T, SIZE_BYTES>(&value) {
        None => {
            let mut sizes = Sizes { heap: 0, stack: 0 };
            {
                let mut serializer =
                    make_serializer::<_, SIZE_BYTES>(buffer.reborrow(), &mut sizes);

                simple_try!(E::serialize(value, &mut serializer));
            }

            buffer.move_to_heap(sizes.heap, sizes.stack, sizes.stack);

            Ok(sizes.total())
        }
        Some(promised) => {
            // Reserive heap space to avoid serializing to stack and moving to heap.
            let reserved = simple_try!(buffer.reserve_heap(0, 0, promised.total()));

            let mut sizes = Sizes { heap: 0, stack: 0 };

            {
                let mut serializer = make_serializer::<_, SIZE_BYTES>(reserved, &mut sizes);
                if let Err(err) = E::serialize(value, &mut serializer) {
                    match err {}
                }
            }

            debug_assert_eq!(
                sizes.stack, promised.stack,
                "Serialization used different amount of stack than promised by `Serialize::size_hint`"
            );
            debug_assert_eq!(
                sizes.heap, promised.heap,
                "Serialization used different amount of heap than promised by `Serialize::size_hint`"
            );

            // No need to move to heap, as exact size was reserved,
            // so no gap between stack and heap is possible.

            Ok(sizes.total())
        }
    }
}

/// Serializes value into bytes slice.
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
pub fn serialize<E, T, const SIZE_BYTES: usize>(
    value: &T,
    output: &mut [u8],
) -> Result<usize, BufferExhausted>
where
    E: Element + ?Sized,
    T: Serialize<E::Formula>,
{
    serialize_into::<E, T, _, SIZE_BYTES>(value, CheckedFixedBuffer::new(output))
}

/// Slightly faster version of [`serialize`].
/// Panics if the buffer is too small instead of returning an error.
///
/// Use instead of using [`serialize`] with immediate [`unwrap`](Result::unwrap).
#[inline(always)]
pub fn serialize_unchecked<E, T, const SIZE_BYTES: usize>(value: &T, output: &mut [u8]) -> usize
where
    E: Element + ?Sized,
    T: Serialize<E::Formula>,
{
    match serialize_into::<E, T, _, SIZE_BYTES>(value, output) {
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
pub fn serialized_size<E, T, const SIZE_BYTES: usize>(value: &T) -> usize
where
    E: Element + ?Sized,
    T: Serialize<E::Formula>,
{
    match serialize_into::<E, T, _, SIZE_BYTES>(value, DryBuffer) {
        Ok(size) => size,
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

/// Serializes value into bytes slice.
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
pub fn serialize_or_size<E, T, const SIZE_BYTES: usize>(
    value: &T,
    output: &mut [u8],
) -> Result<usize, BufferSizeRequired>
where
    E: Element + ?Sized,
    T: Serialize<E::Formula>,
{
    let mut exhausted = false;
    let result =
        serialize_into::<E, T, _, SIZE_BYTES>(value, MaybeFixedBuffer::new(output, &mut exhausted));
    let size = match result {
        Ok(size) => size,
        Err(never) => match never {},
    };
    if exhausted {
        Err(BufferSizeRequired { required: size })
    } else {
        Ok(size)
    }
}

/// Serializes value into byte vector.
/// Returns the number of bytes written.
///
/// Grows the vector if needed.
/// Infallible except for allocation errors.
///
/// Use pre-allocated vector when possible to avoid reallocations.
#[cfg(feature = "alloc")]
#[inline(always)]
pub fn serialize_to_vec<E, T, const SIZE_BYTES: usize>(
    value: &T,
    output: &mut alloc::vec::Vec<u8>,
) -> usize
where
    E: Element + ?Sized,
    T: Serialize<E::Formula>,
{
    use crate::buffer::VecBuffer;

    match serialize_into::<E, T, _, SIZE_BYTES>(value, VecBuffer::new(output)) {
        Ok(size) => size,
        Err(never) => match never {},
    }
}

macro_rules! fixed_size_module {
    ($(#[$meta:meta])* $vis:vis mod $module:ident { $size_bytes:literal }) => {
        $(#[$meta])*
         $vis mod $module {
            use super::*;

            /// Serializes value into bytes slice.
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
            pub fn serialize<E, T>(value: &T, output: &mut [u8]) -> Result<usize, BufferExhausted>
            where
                E: Element + ?Sized,
                T: Serialize<E::Formula>,
            {
                super::serialize::<E, T, $size_bytes>(value, output)
            }

            /// Slightly faster version of [`serialize`].
            /// Panics if the buffer is too small instead of returning an error.
            ///
            /// Use instead of using [`serialize`] with immediate [`unwrap`](Result::unwrap).
            #[inline(always)]
            pub fn serialize_unchecked<E, T>(value: &T, output: &mut [u8]) -> usize
            where
                E: Element + ?Sized,
                T: Serialize<E::Formula>,
            {
                super::serialize_unchecked::<E, T, $size_bytes>(value, output)
            }

            /// Returns the number of bytes required to serialize the value.
            /// Note that value is consumed.
            ///
            /// Use when value is `Copy` or can be cheaply replicated to allocate
            /// the buffer for serialization in advance.
            /// Or to find out required size after [`serialize`] fails.
            #[inline(always)]
            pub fn serialized_size<E, T>(value: &T) -> usize
            where
                E: Element + ?Sized,
                T: Serialize<E::Formula>,
            {
                super::serialized_size::<E, T, $size_bytes>(value)
            }

            /// Serializes value into bytes slice.
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
            pub fn serialize_or_size<E, T>(
                value: &T,
                output: &mut [u8],
            ) -> Result<usize, BufferSizeRequired>
            where
                E: Element + ?Sized,
                T: Serialize<E::Formula>,
            {
                super::serialize_or_size::<E, T, $size_bytes>(value, output)
            }

            /// Serializes value into byte vector.
            /// Returns the number of bytes written.
            ///
            /// Grows the vector if needed.
            /// Infallible except for allocation errors.
            ///
            /// Use pre-allocated vector when possible to avoid reallocations.
            #[cfg(feature = "alloc")]
            #[inline(always)]
            pub fn serialize_to_vec<E, T>(value: &T, output: &mut alloc::vec::Vec<u8>) -> usize
            where
                E: Element + ?Sized,
                T: Serialize<E::Formula>,
            {
                super::serialize_to_vec::<E, T, $size_bytes>(value, output)
            }
        }
    };
}

fixed_size_module! {
    /// Serialization functions for small data.
    ///
    /// They use only 1 byte to encode sizes and indirection,
    /// so max size is 255 bytes and max length of sequences is 255 elements,
    /// even if elements are zero-sized.
    pub mod small { 1 }
}

fixed_size_module! {
    /// Serialization functions for medium data.
    ///
    /// They use only 2 bytes to encode sizes and indirection,
    /// so max size is 65535 bytes and max length of sequences is 65535 elements,
    /// even if elements are zero-sized.
    pub mod medium { 2 }
}

fixed_size_module! {
    /// Serialization functions for large data.
    ///
    /// They use only 4 bytes to encode sizes and indirection,
    /// so max size is 4294967295 bytes and max length of sequences is 4294967295 elements,
    /// even if elements are zero-sized.
    pub mod large { 4 }
}

fixed_size_module! {
    /// Serialization functions for huge data.
    ///
    /// They use 8 bytes to encode sizes and indirection,
    /// so max size is 18446744073709551615 bytes and max length of
    /// sequences is 18446744073709551615 elements, even if elements are zero-sized.
    pub mod huge { 8 }
}

fixed_size_module! {
    /// Serialization functions for humongous data.
    ///
    /// They use 16 bytes to encode sizes and indirection,
    /// so max size is 340282366920938463463374607431768211455 bytes and max length of
    /// sequences is 340282366920938463463374607431768211455 elements, even if elements are zero-sized.
    pub mod humongous { 16 }
}
