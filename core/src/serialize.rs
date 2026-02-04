use core::{fmt, ops};

use crate::{
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
    #[inline]
    pub const fn with_heap(heap: usize) -> Self {
        Sizes { heap, stack: 0 }
    }

    /// Create new `Sizes` with specified stack size.
    #[must_use]
    #[inline]
    pub const fn with_stack(stack: usize) -> Self {
        Sizes { heap: 0, stack }
    }

    /// Adds to the heap size.
    #[inline]
    pub fn add_heap(&mut self, heap: usize) {
        self.heap += heap;
    }

    /// Adds to the stack size.
    #[inline]
    pub fn add_stack(&mut self, stack: usize) {
        self.stack += stack;
    }

    /// Moves stack size to heap size.
    #[inline]
    pub fn to_heap(&mut self, until: usize) -> usize {
        let len = self.stack - until;
        self.heap += len;
        self.stack = until;
        len
    }

    /// Returns total size.
    #[inline]
    pub fn total(&self) -> usize {
        self.heap + self.stack
    }
}

impl ops::Add for Sizes {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self {
            heap: self.heap + rhs.heap,
            stack: self.stack + rhs.stack,
        }
    }
}

impl ops::AddAssign for Sizes {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.heap += rhs.heap;
        self.stack += rhs.stack;
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
    /// `heap` specifies the size of the buffer's heap occupied prior to this call.
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
    ///
    /// This function won't be called if `F` has both [`Formula::EXACT_SIZE`] and [`Formula::HEAPLESS`] set to `true`,
    /// as size must be obtainable from [`Formula::max_stack_size`] in that case.
    fn size_hint<const SIZE_BYTES: u8>(&self) -> Option<Sizes> {
        None
    }
}

/// Returns size hint for serializing value according to formula `F`.
///
/// Avoids calling [`Serialize::size_hint`] for exact-sized, heapless formulas.
///
/// Should be used by composite [`Serialize`] implementations to implement their own [`Serialize::size_hint`].
#[inline]
pub fn size_hint<F: Formula + ?Sized, T: Serialize<F> + ?Sized, const SIZE_BYTES: u8>(
    value: &T,
) -> Option<Sizes> {
    match (stack_size::<F, SIZE_BYTES>(), heap_size::<F, SIZE_BYTES>()) {
        (SizeBound::Exact(stack_size), SizeBound::Exact(heap_size)) => Some(Sizes {
            heap: heap_size,
            stack: stack_size,
        }),
        _ => value.size_hint::<SIZE_BYTES>(),
    }
}

/// Serialize value into buffer.
/// Returns total number of bytes written and size of the root value.
/// The buffer type controls bytes writing and failing strategy.
#[inline]
pub fn serialize_into<F, T, B, const SIZE_BYTES: u8>(
    value: &T,
    buffer: B,
) -> Result<Sizes, B::Error>
where
    F: Formula + ?Sized,
    T: Serialize<F>,
    B: Buffer,
{
    let mut sizes = Sizes { heap: 0, stack: 0 };

    let mut serializer = SerialzierImpl::<B, SIZE_BYTES>::new(&mut sizes, buffer);

    serializer.write_indirect(value)?;
    Ok(sizes)
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
#[inline]
pub fn serialize<F, T, const SIZE_BYTES: u8>(
    value: &T,
    output: &mut [u8],
) -> Result<Sizes, BufferExhausted>
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    serialize_into::<F, T, _, SIZE_BYTES>(value, CheckedFixedBuffer::new(output))
}

/// Slightly faster version of [`serialize`].
/// Panics if the buffer is too small instead of returning an error.
///
/// Use instead of using [`serialize`] with immediate [`unwrap`](Result::unwrap).
#[inline]
pub fn serialize_unchecked<F, T, const SIZE_BYTES: u8>(value: &T, output: &mut [u8]) -> Sizes
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    match serialize_into::<F, T, _, SIZE_BYTES>(value, output) {
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
#[inline]
pub fn serialize_or_size<F, T, const SIZE_BYTES: u8>(
    value: &T,
    output: &mut [u8],
) -> Result<Sizes, BufferSizeRequired>
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    let mut exhausted = false;
    let result =
        serialize_into::<F, T, _, SIZE_BYTES>(value, MaybeFixedBuffer::new(output, &mut exhausted));
    let sizes = match result {
        Ok(sizes) => sizes,
        Err(never) => match never {},
    };
    if exhausted {
        Err(BufferSizeRequired {
            required: sizes.total(),
        })
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
#[inline]
pub fn serialize_to_vec<F, T, const SIZE_BYTES: u8>(
    value: &T,
    output: &mut alloc::vec::Vec<u8>,
) -> Sizes
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    use crate::buffer::VecBuffer;

    match serialize_into::<F, T, _, SIZE_BYTES>(value, VecBuffer::new(output)) {
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
#[inline]
pub fn serialized_sizes<F, T, const SIZE_BYTES: u8>(value: &T) -> Sizes
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    match serialize_into::<F, T, _, SIZE_BYTES>(value, DryBuffer) {
        Ok(sizes) => sizes,
        Err(never) => match never {},
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
    fn write_indirect<F, T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        F: Formula + ?Sized,
        T: Serialize<F> + ?Sized;
}

struct SerialzierImpl<'a, B: Buffer, const SIZE_BYTES: u8> {
    sizes: &'a mut Sizes,
    buffer: B,

    // Number of bytes of padding to adde before next element.
    // It is set when writing direct elements with actual size less than formula's max stack size.
    pad_next: usize,
}

impl<'a, B, const SIZE_BYTES: u8> SerialzierImpl<'a, B, SIZE_BYTES>
where
    B: Buffer,
{
    #[inline]
    fn new(sizes: &'a mut Sizes, buffer: B) -> Self {
        SerialzierImpl {
            sizes,
            buffer,
            pad_next: 0,
        }
    }

    fn reserved<'b>(
        sizes: &'b mut Sizes,
        buffer: B::ReservedHeap<'b>,
    ) -> SerialzierImpl<'b, B::ReservedHeap<'b>, SIZE_BYTES> {
        SerialzierImpl::new(sizes, buffer)
    }

    fn reborrow(&mut self) -> SerialzierImpl<'_, B::Reborrow<'_>, SIZE_BYTES> {
        SerialzierImpl::new(self.sizes, self.buffer.reborrow())
    }

    #[inline]
    fn write_to_heap<F, T>(&mut self, value: &T) -> Result<(), B::Error>
    where
        F: Formula + ?Sized,
        T: Serialize<F> + ?Sized,
    {
        let old_stack = self.sizes.stack;
        self.write_direct::<F, T>(value)?;
        let len = self.sizes.to_heap(old_stack);
        self.buffer
            .move_to_heap(self.sizes.heap - len, self.sizes.stack + len, len);
        Ok(())
    }

    #[inline]
    fn write_padding(&mut self) -> Result<(), B::Error> {
        if self.pad_next > 0 {
            self.buffer
                .pad_stack(self.sizes.heap, self.sizes.stack, self.pad_next)?;
            self.sizes.stack += self.pad_next;
            self.pad_next = 0;
        }
        Ok(())
    }
}

impl<'a, B, const SIZE_BYTES: u8> Serializer for SerialzierImpl<'a, B, SIZE_BYTES>
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
    #[inline]
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        self.write_padding()?;
        self.buffer
            .write_stack(self.sizes.heap, self.sizes.stack, bytes)?;
        self.sizes.stack += bytes.len();
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
        const {
            assert!(F::INHABITED);
        }

        self.write_padding()?;

        if let SizeBound::Exact(0) | SizeBound::Bounded(0) = stack_size::<F, SIZE_BYTES>() {
            debug_assert!(matches!(
                heap_size::<F, SIZE_BYTES>(),
                SizeBound::Exact(0) | SizeBound::Bounded(0)
            ));

            // No need to serialize zero-sized value.
            // In release builds we simply skip serialization.
            #[cfg(not(debug_assertions))]
            return Ok(());
        }

        let old_stack = self.sizes.stack;

        <T as Serialize<F>>::serialize(value, self.reborrow())?;

        let actual_size = self.sizes.stack - old_stack;

        match stack_size::<F, SIZE_BYTES>() {
            SizeBound::Unbounded => {}
            SizeBound::Bounded(max_stack) => {
                debug_assert!(actual_size <= max_stack);
                self.pad_next = old_stack + max_stack - self.sizes.stack;
            }
            SizeBound::Exact(exact_stack) => {
                // This branch can be chosen at compile time,
                // so we simply avoid simple calculation of the branch above.
                debug_assert_eq!(actual_size, exact_stack);
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
    fn write_indirect<F, T>(&mut self, value: &T) -> Result<(), B::Error>
    where
        F: Formula + ?Sized,
        T: Serialize<F> + ?Sized,
    {
        const {
            assert!(F::INHABITED);
        }

        // Can we get size hint for the value?
        match size_hint::<F, T, SIZE_BYTES>(&value) {
            None => {
                // Size hint is unobtainable, serialize to heap through stack and move to heap.
                self.write_to_heap::<F, T>(value)?;
            }
            Some(promised) => {
                // Reserive heap space to avoid serializing to stack and moving to heap.
                let reserved = self.buffer.reserve_heap(
                    self.sizes.heap,
                    self.sizes.stack,
                    promised.total(),
                )?;

                let mut sizes = Sizes {
                    heap: self.sizes.heap,
                    stack: 0,
                };

                let serializer = Self::reserved(&mut sizes, reserved);

                <T as Serialize<F>>::serialize(value, serializer).expect("Reserved enough space");

                debug_assert_eq!(
                    sizes.heap,
                    self.sizes.heap + promised.heap,
                    "Serialization used more heap than promised by `Serialize::size_hint`"
                );
                debug_assert_eq!(
                    sizes.stack, promised.stack,
                    "Serialization used more stack than promised by `Serialize::size_hint`"
                );

                // Flush reserved stack to heap.
                self.sizes.heap += sizes.stack;
            }
        }

        self.write_padding()?;

        let address = self.sizes.heap;
        self.write_usize(address)?;

        Ok(())
    }

    /// Specialized method to write usize value in `SIZE_BYTES` bytes.
    fn write_usize(&mut self, value: usize) -> Result<(), Self::Error> {
        let max_size: usize = 1 << (SIZE_BYTES * 8);
        assert!(
            value < max_size,
            "Value too large to fit in SIZE_BYTES bytes ({SIZE_BYTES})"
        );

        let bytes = value.to_le_bytes();

        self.write_bytes(&bytes[..usize::from(SIZE_BYTES)])
    }
}
