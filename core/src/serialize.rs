use core::{convert::Infallible, fmt, ops};

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

pub(crate) struct ComplexSerializer<'a, B: Buffer, const SIZE_BYTES: usize> {
    sizes: &'a mut Sizes,
    buffer: B,

    // Number of bytes of padding to add before next element.
    // It is set when writing direct elements with actual size less than formula's max stack size.
    pad_next: usize,
}

impl<'a, B, const SIZE_BYTES: usize> ComplexSerializer<'a, B, SIZE_BYTES>
where
    B: Buffer,
{
    #[inline(always)]
    fn new(sizes: &'a mut Sizes, buffer: B) -> Self {
        ComplexSerializer {
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

impl<'a, B, const SIZE_BYTES: usize> Serializer for ComplexSerializer<'a, B, SIZE_BYTES>
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
        simple_try!(self.buffer.reserve(
            self.sizes.heap,
            self.sizes.stack,
            self.pad_next + bytes.len(),
        ));

        self.sizes.stack += self.pad_next;
        self.pad_next = 0;

        self.buffer.write_stack(self.sizes.stack, bytes);

        self.sizes.stack += bytes.len();

        Ok(())
    }

    /// Specialized method to write usize value in `SIZE_BYTES` bytes.
    #[inline(always)]
    fn write_usize(&mut self, value: usize) -> Result<(), Self::Error> {
        simple_try!(self.buffer.reserve(
            self.sizes.heap,
            self.sizes.stack,
            self.pad_next + SIZE_BYTES,
        ));

        self.sizes.stack += self.pad_next;
        self.pad_next = 0;

        write_usize::<_, SIZE_BYTES>(value, self.sizes.stack, self.buffer.reborrow());

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
        let _is_zero = is_zero::<F, SIZE_BYTES>();

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

        match const { trivial_size::<F, SIZE_BYTES>() } {
            Some(size) => {
                // Switch to trivial layout serialization.

                let reserved = simple_try!(self.buffer.reserved(
                    self.sizes.heap,
                    self.sizes.stack,
                    size + self.pad_next,
                ));

                if let Some(reserved) = reserved {
                    let at = reserved.len() - self.sizes.stack - size - self.pad_next;
                    let tail = &mut reserved[at..][..size];

                    if let Err(err) = <T as Serialize<F>>::serialize(
                        value,
                        TrivialSerializer::<SIZE_BYTES>::new(tail),
                    ) {
                        match err {}
                    }
                }

                self.sizes.stack += size + self.pad_next;
                self.pad_next = 0;

                return Ok(());
            }
            _ => {}
        }

        let old_sizes;

        if !B::RESERVED_IS_SELF
            && let Some(sizes) = const { exact_sizes::<F, T, SIZE_BYTES>() }
        {
            // If size is known - serialize with reserved buffer, unless it's the same kind of buffer.

            let reserved = simple_try!(self.buffer.reserved(
                self.sizes.heap,
                self.sizes.stack,
                sizes.stack + sizes.heap + self.pad_next,
            ));

            self.sizes.stack += self.pad_next;
            self.pad_next = 0;

            old_sizes = *self.sizes;

            if let Some(reserved) = reserved {
                let serializer = ComplexSerializer::<_, SIZE_BYTES>::new(&mut self.sizes, reserved);
                if let Err(err) = <T as Serialize<F>>::serialize(value, serializer) {
                    match err {}
                }
            } else {
                let serializer = ComplexSerializer::<_, SIZE_BYTES>::new(self.sizes, DryBuffer);
                if let Err(err) = <T as Serialize<F>>::serialize(value, serializer) {
                    match err {}
                }
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
                ComplexSerializer::<_, SIZE_BYTES>::new(self.sizes, self.buffer.reborrow())
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

        if matches!(
            const { (stack_size::<E, SIZE_BYTES>(), heap_size::<E, SIZE_BYTES>()) },
            (SizeBound::Exact(0), SizeBound::Exact(0))
        ) {
            return Ok(());
        }

        // Can we get size hint for the value?
        match size_hint::<E, T, SIZE_BYTES>(&value) {
            None => {
                // Size hint is unobtainable, serialize to heap through stack and move to heap.
                simple_try!(self.write_to_heap::<E, T>(value));
            }
            Some(promised) => {
                // Reserive heap space to avoid serializing to stack and moving to heap.
                let reserved = simple_try!(self.buffer.reserved_heap(
                    self.sizes.heap,
                    self.sizes.stack,
                    promised.total(),
                ));

                let mut sizes = Sizes {
                    heap: self.sizes.heap,
                    stack: 0,
                };

                if let Some(reserved) = reserved {
                    let mut serializer =
                        ComplexSerializer::<_, SIZE_BYTES>::new(&mut sizes, reserved);
                    if let Err(err) = E::serialize(value, &mut serializer) {
                        match err {}
                    }
                } else {
                    let mut serializer =
                        ComplexSerializer::<_, SIZE_BYTES>::new(&mut sizes, DryBuffer);
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

pub(crate) struct TrivialSerializer<'a, const SIZE_BYTES: usize> {
    bytes: &'a mut [u8],
}

impl<'a, const SIZE_BYTES: usize> TrivialSerializer<'a, SIZE_BYTES> {
    fn new(bytes: &'a mut [u8]) -> Self {
        TrivialSerializer { bytes }
    }
}

impl<'a, const SIZE_BYTES: usize> Serializer for TrivialSerializer<'a, SIZE_BYTES> {
    type Error = Infallible;

    /// Write raw bytes to the buffer.
    ///
    /// Use in [`Serialize::serialize`](Serialize::serialize) implementation.
    ///
    /// # Errors
    ///
    /// Returns error if buffer write fails.
    #[inline(always)]
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Infallible> {
        let (head, tail) = core::mem::take(&mut self.bytes).split_at_mut(bytes.len());
        self.bytes = tail;
        head.copy_from_slice(bytes);
        Ok(())
    }

    /// Specialized method to write usize value in `SIZE_BYTES` bytes.
    #[inline(always)]
    fn write_usize(&mut self, value: usize) -> Result<(), Infallible> {
        let (head, tail) = core::mem::take(&mut self.bytes).split_at_mut(SIZE_BYTES);
        self.bytes = tail;
        write_usize_trivial::<SIZE_BYTES>(value, head);
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
    fn write_direct<F, T>(&mut self, value: &T) -> Result<(), Infallible>
    where
        F: Formula + ?Sized,
        T: Serialize<F> + ?Sized,
    {
        let _is_zero = is_zero::<F, SIZE_BYTES>();

        #[cfg(not(debug_assertions))]
        if _is_zero {
            // No need to serialize zero-sized value.
            // In release builds we simply skip serialization.
            return Ok(());
        }

        match const { (stack_size::<F, SIZE_BYTES>(), heap_size::<F, SIZE_BYTES>()) } {
            (
                SizeBound::Exact(stack_size) | SizeBound::Bounded(stack_size),
                SizeBound::Exact(0),
            ) => {
                // Switch to trivial layout serialization.
                let (head, tail) = core::mem::take(&mut self.bytes).split_at_mut(stack_size);
                self.bytes = tail;

                if let Err(err) = <T as Serialize<F>>::serialize(
                    value,
                    TrivialSerializer::<SIZE_BYTES>::new(head),
                ) {
                    match err {}
                }

                Ok(())
            }
            _ => {
                unreachable!()
            }
        }
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
    fn write_indirect<E, T>(&mut self, _value: &T) -> Result<(), Infallible>
    where
        E: Element + ?Sized,
        T: Serialize<E::Formula> + ?Sized,
    {
        unreachable!()
    }

    #[inline(always)]
    fn reserve_usize(&mut self) -> Result<usize, Infallible> {
        unreachable!()
    }

    #[inline(always)]
    fn write_reserved_usize(&mut self, _address: usize, _value: usize) {
        unreachable!()
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

/// Specialized method to write usize value in `SIZE_BYTES` bytes.
#[inline(always)]
pub fn write_usize_trivial<const SIZE_BYTES: usize>(value: usize, bytes: &mut [u8]) {
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
            bytes.copy_from_slice(&value.to_le_bytes()[..SIZE_BYTES]);
        }
        () if SIZE_BYTES > LEN => {
            bytes[..LEN].copy_from_slice(&value.to_le_bytes());
        }
        () => {
            // SIZE_BYTES == LEN
            bytes.copy_from_slice(&value.to_le_bytes());
        }
    }
}

#[inline(always)]
pub const fn exact_sizes<
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

pub(crate) const fn is_zero<E: Element + ?Sized, const SIZE_BYTES: usize>() -> bool {
    if let SizeBound::Exact(0) = stack_size::<E, SIZE_BYTES>() {
        assert!(matches!(heap_size::<E, SIZE_BYTES>(), SizeBound::Exact(0)));
        true
    } else {
        false
    }
}

pub(crate) const fn trivial_size<E: Element + ?Sized, const SIZE_BYTES: usize>() -> Option<usize> {
    match (stack_size::<E, SIZE_BYTES>(), heap_size::<E, SIZE_BYTES>()) {
        (SizeBound::Exact(size), SizeBound::Exact(0)) => Some(size),
        _ => None,
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
    ComplexSerializer::<B, SIZE_BYTES>::new(sizes, buffer)
}

#[inline(always)]
pub fn make_trivial_serializer<'a, const SIZE_BYTES: usize>(
    bytes: &'a mut [u8],
) -> impl Serializer<Error = Infallible> + use<'a, SIZE_BYTES> {
    TrivialSerializer::<SIZE_BYTES>::new(bytes)
}

/// Serializes value into buffer.
/// Returns total number of bytes written and size of the root value.
/// The buffer type controls bytes writing and failing strategy.
pub fn serialize_into<F, T, B, const SIZE_BYTES: usize>(
    value: &T,
    mut buffer: B,
) -> Result<usize, B::Error>
where
    F: Formula + ?Sized,
    T: Serialize<F>,
    B: Buffer,
{
    const {
        assert!(F::INHABITED);
    }

    if let Some(trivial_size) = const { trivial_size::<F, SIZE_BYTES>() } {
        let reserved = simple_try!(buffer.reserved_heap(0, 0, trivial_size));

        if let Some(reserved) = reserved {
            if let Err(err) = <T as Serialize<F>>::serialize(
                value,
                make_trivial_serializer::<SIZE_BYTES>(&mut reserved[..trivial_size]),
            ) {
                match err {}
            }
        }

        return Ok(trivial_size);
    }

    match size_hint::<F, T, SIZE_BYTES>(&value) {
        None => {
            let mut sizes = Sizes { heap: 0, stack: 0 };
            {
                let serializer = make_serializer::<_, SIZE_BYTES>(buffer.reborrow(), &mut sizes);
                simple_try!(<T as Serialize<F>>::serialize(value, serializer));
            }

            buffer.move_to_heap(sizes.heap, sizes.stack, sizes.stack);

            Ok(sizes.total())
        }
        Some(promised) => {
            // Reserive heap space to avoid serializing to stack and moving to heap.
            let reserved = simple_try!(buffer.reserved_heap(0, 0, promised.total()));

            let mut sizes = Sizes { heap: 0, stack: 0 };

            if let Some(reserved) = reserved {
                let serializer = make_serializer::<_, SIZE_BYTES>(reserved, &mut sizes);
                if let Err(err) = <T as Serialize<F>>::serialize(value, serializer) {
                    match err {}
                }
            } else {
                let serializer = make_serializer::<_, SIZE_BYTES>(DryBuffer, &mut sizes);
                if let Err(err) = <T as Serialize<F>>::serialize(value, serializer) {
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
pub fn serialize<F, T, const SIZE_BYTES: usize>(
    value: &T,
    output: &mut [u8],
) -> Result<usize, BufferExhausted>
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
#[inline(always)]
pub fn serialize_unchecked<F, T, const SIZE_BYTES: usize>(value: &T, output: &mut [u8]) -> usize
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    match serialize_into::<F, T, _, SIZE_BYTES>(value, output) {
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
pub fn serialized_size<F, T, const SIZE_BYTES: usize>(value: &T) -> usize
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    match serialize_into::<F, T, _, SIZE_BYTES>(value, DryBuffer) {
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
pub fn serialize_or_size<F, T, const SIZE_BYTES: usize>(
    value: &T,
    output: &mut [u8],
) -> Result<usize, BufferSizeRequired>
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    let mut exhausted = false;
    let result =
        serialize_into::<F, T, _, SIZE_BYTES>(value, MaybeFixedBuffer::new(output, &mut exhausted));
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
pub fn serialize_to_vec<F, T, const SIZE_BYTES: usize>(
    value: &T,
    output: &mut alloc::vec::Vec<u8>,
) -> usize
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    use crate::buffer::VecBuffer;

    match serialize_into::<F, T, _, SIZE_BYTES>(value, VecBuffer::new(output)) {
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
            pub fn serialize<F, T>(value: &T, output: &mut [u8]) -> Result<usize, BufferExhausted>
            where
                F: Formula + ?Sized,
                T: Serialize<F>,
            {
                super::serialize::<F, T, $size_bytes>(value, output)
            }

            /// Slightly faster version of [`serialize`].
            /// Panics if the buffer is too small instead of returning an error.
            ///
            /// Use instead of using [`serialize`] with immediate [`unwrap`](Result::unwrap).
            #[inline(always)]
            pub fn serialize_unchecked<F, T>(value: &T, output: &mut [u8]) -> usize
            where
                F: Formula + ?Sized,
                T: Serialize<F>,
            {
                super::serialize_unchecked::<F, T, $size_bytes>(value, output)
            }

            /// Returns the number of bytes required to serialize the value.
            /// Note that value is consumed.
            ///
            /// Use when value is `Copy` or can be cheaply replicated to allocate
            /// the buffer for serialization in advance.
            /// Or to find out required size after [`serialize`] fails.
            #[inline(always)]
            pub fn serialized_size<F, T>(value: &T) -> usize
            where
                F: Formula + ?Sized,
                T: Serialize<F>,
            {
                super::serialized_size::<F, T, $size_bytes>(value)
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
            pub fn serialize_or_size<F, T>(
                value: &T,
                output: &mut [u8],
            ) -> Result<usize, BufferSizeRequired>
            where
                F: Formula + ?Sized,
                T: Serialize<F>,
            {
                super::serialize_or_size::<F, T, $size_bytes>(value, output)
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
            pub fn serialize_to_vec<F, T>(value: &T, output: &mut alloc::vec::Vec<u8>) -> usize
            where
                F: Formula + ?Sized,
                T: Serialize<F>,
            {
                super::serialize_to_vec::<F, T, $size_bytes>(value, output)
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
