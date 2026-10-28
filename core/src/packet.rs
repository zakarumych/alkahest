use crate::{
    Deserialize, DeserializeError, Element, Formula, Serialize, SizeBound, Sizes,
    buffer::{
        Buffer, BufferExhausted, CheckedFixedBuffer, DryBuffer, MaybeFixedBuffer, OptimisticBuffer,
    },
    deserialize::{deserialize_at_version, deserialize_in_place_at_version, read_usize},
    element::{heap_size_versioned, stack_size_versioned},
    serialize::{
        BufferSizeRequired, ComplexSerializer, TrivialSerializer, size_hint_versioned,
        trivial_size, write_usize,
    },
};

#[inline]
const fn total<E, const SIZE_BYTES: usize, const VERSION: u32>() -> Option<usize>
where
    E: Element<VERSION> + ?Sized,
{
    match (
        stack_size_versioned::<E, SIZE_BYTES, VERSION>(),
        heap_size_versioned::<E, SIZE_BYTES, VERSION>(),
    ) {
        (SizeBound::Exact(stack), SizeBound::Exact(heap)) => Some(stack + heap),
        (SizeBound::Exact(stack), SizeBound::Bounded(0)) => Some(stack),
        (SizeBound::Bounded(0), SizeBound::Exact(heap)) => Some(heap),
        (SizeBound::Bounded(0), SizeBound::Bounded(0)) => Some(0),

        _ => None,
    }
}

/// Packs value into buffer.
/// Returns total number of bytes written.
/// The buffer type controls bytes writing and failing strategy.
///
/// Unlike [`serialize_into`] adds length prefix to the output unless formula has exact total size known in advance.
/// Length prefix is encoded as unsigned integer with `SIZE_BYTES` bytes in little-endian format.
/// Enabling calling [`unpack`] without slicing input data to the exact length returned by serialization function.
/// This allows more convenient use with byte streams where end of input is not known in advance.
///
/// [`serialize_into`]: crate::serialize::serialize_into

#[inline(always)]
pub fn pack_into<F, T, B, const SIZE_BYTES: usize>(value: &T, buffer: B) -> Result<usize, B::Error>
where
    F: Formula + ?Sized,
    T: Serialize<F>,
    B: Buffer,
{
    pack_into_at_version::<F, T, B, SIZE_BYTES, 0>(value, buffer)
}
/// Version-selecting form of [`pack_into`]. Uses `VERSION` without compatibility checks or a version tag.
#[inline(always)]
pub fn pack_into_at_version<F, T, B, const SIZE_BYTES: usize, const VERSION: u32>(
    value: &T,
    mut buffer: B,
) -> Result<usize, B::Error>
where
    F: Formula<VERSION> + ?Sized,
    T: Serialize<F, VERSION>,
    B: Buffer,
{
    const {
        assert!(F::INHABITED);
    }

    if let Some(trivial_size) = const { trivial_size::<F, SIZE_BYTES, VERSION>() } {
        let reserved = simple_try!(buffer.reserved_heap(0, 0, trivial_size));

        if let Some(reserved) = reserved {
            if let Err(err) = <T as Serialize<F, VERSION>>::serialize(
                value,
                TrivialSerializer::<SIZE_BYTES>::new(reserved),
            ) {
                match err {}
            }
        }

        return Ok(trivial_size);
    }

    let total = const { total::<F, SIZE_BYTES, VERSION>() };

    if total.is_none() {
        // Pre-reserve space for length prefix if total size is not known in advance.
        let _ = simple_try!(buffer.reserved_heap(0, 0, SIZE_BYTES));
    }

    // Initialize sizes with first `SIZE_BYTES` bytes reserved for length prefix
    // unless total size is known in advance.
    let mut sizes = Sizes {
        heap: total.map_or(SIZE_BYTES, |_| 0),
        stack: 0,
    };

    match size_hint_versioned::<F, T, SIZE_BYTES, VERSION>(&value) {
        None => {
            {
                let serializer =
                    ComplexSerializer::<_, SIZE_BYTES>::new(&mut sizes, buffer.reborrow());

                simple_try!(<T as Serialize<F, VERSION>>::serialize(value, serializer));
            }

            buffer.move_to_heap(sizes.heap, sizes.stack, sizes.stack);
        }
        Some(promised) => {
            // Reserive heap space to avoid serializing to stack and moving to heap.
            let old_heap = sizes.heap;
            let reserved = simple_try!(buffer.reserved_heap(old_heap, 0, promised.total()));

            if let Some(reserved) = reserved {
                let serializer = ComplexSerializer::<_, SIZE_BYTES>::new(&mut sizes, reserved);
                if let Err(err) = <T as Serialize<F, VERSION>>::serialize(value, serializer) {
                    match err {}
                }

                debug_assert_eq!(
                    sizes.stack, promised.stack,
                    "Serialization used different amount of stack than promised by `Serialize::size_hint`"
                );
                debug_assert_eq!(
                    sizes.heap,
                    old_heap + promised.heap,
                    "Serialization used different amount of heap than promised by `Serialize::size_hint`"
                );
            } else {
                sizes += promised;
            }

            // No need to move to heap, as exact size was reserved,
            // so no gap between stack and heap is possible.
        }
    }

    let actual = sizes.total();

    match total {
        None => {
            let reserved = match buffer.reserved_heap(0, 0, SIZE_BYTES) {
                Ok(reserved) => reserved,
                Err(_err) => {
                    unreachable!("Failed to reserve space for length prefix that was pre-reserved");
                }
            };

            if let Some(reserved) = reserved {
                write_usize::<_, SIZE_BYTES>(actual, 0, reserved);
            }
        }
        Some(total) => {
            assert_eq!(
                sizes.total(),
                total,
                "Formula's exact total size does not match actual serialized size"
            );
        }
    }

    Ok(sizes.total())
}

/// Packs value into bytes slice.
/// Returns the number of bytes written.
/// Fails if the buffer is too small.
///
/// To retrieve the number of bytes required to serialize the value,
/// use [`pack_size`] or [`pack_or_size`].
///
/// Unlike [`serialize`] adds length prefix to the output unless formula has exact total size known in advance.
/// Length prefix is encoded as unsigned integer with `SIZE_BYTES` bytes in little-endian format.
/// Enabling calling [`unpack`] without slicing input data to the exact length returned by serialization function.
/// This allows more convenient use with byte streams where end of input is not known in advance.
///
/// # Errors
///
/// Returns [`BufferExhausted`] if the buffer is too small.
///
/// [`serialize`]: crate::serialize::serialize
#[inline(always)]
pub fn pack<F, T, const SIZE_BYTES: usize>(
    value: &T,
    output: &mut [u8],
) -> Result<usize, BufferExhausted>
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    pack_at_version::<F, T, SIZE_BYTES, 0>(value, output)
}
/// Version-selecting form of [`pack`]. Uses `VERSION` without compatibility checks or a version tag.
#[inline(always)]
pub fn pack_at_version<F, T, const SIZE_BYTES: usize, const VERSION: u32>(
    value: &T,
    output: &mut [u8],
) -> Result<usize, BufferExhausted>
where
    F: Formula<VERSION> + ?Sized,
    T: Serialize<F, VERSION>,
{
    pack_into_at_version::<F, T, _, SIZE_BYTES, VERSION>(value, CheckedFixedBuffer::new(output))
}

/// Slightly faster version of [`pack`].
/// Panics if the buffer is too small instead of returning an error.
///
/// Use instead of using [`pack`] with immediate [`unwrap`](Result::unwrap).
///
/// Unlike [`serialize_unchecked`] adds length prefix to the output unless formula has exact total size known in advance.
/// Length prefix is encoded as unsigned integer with `SIZE_BYTES` bytes in little-endian format.
/// Enabling calling [`unpack`] without slicing input data to the exact length returned by serialization function.
/// This allows more convenient use with byte streams where end of input is not known in advance.
///
/// [`serialize_unchecked`]: crate::serialize::serialize_unchecked
#[inline(always)]
pub fn pack_unchecked<F, T, const SIZE_BYTES: usize>(value: &T, output: &mut [u8]) -> usize
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    pack_unchecked_at_version::<F, T, SIZE_BYTES, 0>(value, output)
}
/// Version-selecting form of [`pack_unchecked`]. Uses `VERSION` without compatibility checks or a version tag.
#[inline(always)]
pub fn pack_unchecked_at_version<F, T, const SIZE_BYTES: usize, const VERSION: u32>(
    value: &T,
    output: &mut [u8],
) -> usize
where
    F: Formula<VERSION> + ?Sized,
    T: Serialize<F, VERSION>,
{
    match pack_into_at_version::<F, T, _, SIZE_BYTES, VERSION>(value, OptimisticBuffer::new(output))
    {
        Ok(size) => size,
        Err(never) => match never {},
    }
}

/// Returns the number of bytes required to pack the value.
/// Note that value is consumed.
///
/// Use when value is `Copy` or can be cheaply replicated to allocate
/// the buffer for serialization in advance.
/// Or to find out required size after [`pack`] fails.
///
/// Unlike [`serialized_size`] adds length prefix to the calculations unless formula has exact total size known in advance.
/// Length prefix is encoded as unsigned integer with `SIZE_BYTES` bytes in little-endian format.
/// Enabling calling [`unpack`] without slicing input data to the exact length returned by serialization function.
/// This allows more convenient use with byte streams where end of input is not known in advance.
///
/// [`serialized_size`]: crate::serialize::serialized_size
#[inline(always)]
pub fn pack_size<F, T, const SIZE_BYTES: usize>(value: &T) -> usize
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    pack_size_at_version::<F, T, SIZE_BYTES, 0>(value)
}
/// Version-selecting form of [`pack_size`]. Uses `VERSION` without compatibility checks or a version tag.
#[inline(always)]
pub fn pack_size_at_version<F, T, const SIZE_BYTES: usize, const VERSION: u32>(value: &T) -> usize
where
    F: Formula<VERSION> + ?Sized,
    T: Serialize<F, VERSION>,
{
    match pack_into_at_version::<F, T, _, SIZE_BYTES, VERSION>(value, DryBuffer) {
        Ok(size) => size,
        Err(never) => match never {},
    }
}

/// Packs value into bytes slice.
/// Returns the number of bytes written.
///
/// If the buffer is too small, returns error that contains
/// the exact number of bytes required.
///
/// Use [`pack`] if this information is not needed.
///
/// # Errors
///
/// Returns [`BufferSizeRequired`] error if the buffer is too small.
/// Error contains the exact number of bytes required.
///
/// Unlike [`serialize_or_size`] adds length prefix to the output unless formula has exact total size known in advance.
/// Length prefix is encoded as unsigned integer with `SIZE_BYTES` bytes in little-endian format.
/// Enabling calling [`unpack`] without slicing input data to the exact length returned by serialization function.
/// This allows more convenient use with byte streams where end of input is not known in advance.
///
/// [`serialize_or_size`]: crate::serialize::serialize_or_size
#[inline(always)]
pub fn pack_or_size<F, T, const SIZE_BYTES: usize>(
    value: &T,
    output: &mut [u8],
) -> Result<usize, BufferSizeRequired>
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    pack_or_size_at_version::<F, T, SIZE_BYTES, 0>(value, output)
}
/// Version-selecting form of [`pack_or_size`]. Uses `VERSION` without compatibility checks or a version tag.
#[inline(always)]
pub fn pack_or_size_at_version<F, T, const SIZE_BYTES: usize, const VERSION: u32>(
    value: &T,
    output: &mut [u8],
) -> Result<usize, BufferSizeRequired>
where
    F: Formula<VERSION> + ?Sized,
    T: Serialize<F, VERSION>,
{
    let mut exhausted = false;
    let result = pack_into_at_version::<F, T, _, SIZE_BYTES, VERSION>(
        value,
        MaybeFixedBuffer::new(output, &mut exhausted),
    );
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

/// Packs value into byte vector.
/// Returns the number of bytes written.
///
/// Grows the vector if needed.
/// Infallible except for allocation errors.
///
/// Use pre-allocated vector when possible to avoid reallocations.
///
/// Unlike [`serialize_to_vec`] adds length prefix to the output unless formula has exact total size known in advance.
/// Length prefix is encoded as unsigned integer with `SIZE_BYTES` bytes in little-endian format.
/// Enabling calling [`unpack`] without slicing input data to the exact length returned by serialization function.
/// This allows more convenient use with byte streams where end of input is not known in advance.
///
/// [`serialize_to_vec`]: crate::serialize::serialize_to_vec
#[cfg(feature = "alloc")]
#[inline(always)]
pub fn pack_to_vec<F, T, const SIZE_BYTES: usize>(
    value: &T,
    output: &mut alloc::vec::Vec<u8>,
) -> usize
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    pack_to_vec_at_version::<F, T, SIZE_BYTES, 0>(value, output)
}
/// Version-selecting form of [`pack_to_vec`]. Uses `VERSION` without compatibility checks or a version tag.
#[cfg(feature = "alloc")]
#[inline(always)]
pub fn pack_to_vec_at_version<F, T, const SIZE_BYTES: usize, const VERSION: u32>(
    value: &T,
    output: &mut alloc::vec::Vec<u8>,
) -> usize
where
    F: Formula<VERSION> + ?Sized,
    T: Serialize<F, VERSION>,
{
    use crate::buffer::VecBuffer;

    match pack_into_at_version::<F, T, _, SIZE_BYTES, VERSION>(value, VecBuffer::new(output)) {
        Ok(size) => size,
        Err(never) => match never {},
    }
}

/// Returns the number of bytes of the packed value in the input.
#[inline(always)]
pub fn read_pack_size<F, const SIZE_BYTES: usize>(input: &[u8]) -> Result<usize, DeserializeError>
where
    F: Formula + ?Sized,
{
    read_pack_size_at_version::<F, SIZE_BYTES, 0>(input)
}
/// Version-selecting form of [`read_pack_size`]. Uses `VERSION` without compatibility checks or a version tag.
#[inline(always)]
pub fn read_pack_size_at_version<F, const SIZE_BYTES: usize, const VERSION: u32>(
    input: &[u8],
) -> Result<usize, DeserializeError>
where
    F: Formula<VERSION> + ?Sized,
{
    let total = total::<F, SIZE_BYTES, VERSION>();

    match total {
        Some(total) => Ok(total),
        None => match input.first_chunk::<SIZE_BYTES>() {
            None => Err(DeserializeError::OutOfBounds(SIZE_BYTES)),
            Some(bytes) => read_usize::<SIZE_BYTES>(bytes),
        },
    }
}

/// Deserializes value from the input.
/// Returns deserialized value.
/// Unlike [`deserialize`] this function allows input to be longer than the length returned by packing function.
/// If input is shorter than the length returned by packing function,
/// this function will return [`DeserializeError::OutOfBounds`] error with the least number of bytes required to deserialize the value.
///
/// This allows more convenient use with byte streams where end of input is not known in advance.
///
/// # Errors
///
/// Returns [`DeserializeError`] if deserialization fails.

#[inline(always)]
pub fn unpack<'de, F, T, const SIZE_BYTES: usize>(
    input: &'de [u8],
) -> Result<(T, usize), DeserializeError>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F>,
{
    unpack_at_version::<F, T, SIZE_BYTES, 0>(input)
}
/// Version-selecting form of [`unpack`]. Uses `VERSION` without compatibility checks or a version tag.
#[inline(always)]
pub fn unpack_at_version<'de, F, T, const SIZE_BYTES: usize, const VERSION: u32>(
    input: &'de [u8],
) -> Result<(T, usize), DeserializeError>
where
    F: Formula<VERSION> + ?Sized,
    T: Deserialize<'de, F, VERSION>,
{
    let total = simple_try!(read_pack_size_at_version::<F, SIZE_BYTES, VERSION>(input));
    if input.len() < total {
        return Err(DeserializeError::OutOfBounds(total));
    }
    let value = simple_try!(deserialize_at_version::<F, T, SIZE_BYTES, VERSION>(
        &input[..total]
    ));
    Ok((value, total))
}

/// Deserializes value from the input.
/// Updates value in-place.
/// Unlike [`deserialize`] this function allows input to be longer than the length returned by packing function.
/// If input is shorter than the length returned by packing function,
/// this function will return [`DeserializeError::OutOfBounds`] error with the least number of bytes required to deserialize the value.
///
/// This allows more convenient use with byte streams where end of input is not known in advance.
///
/// # Errors
///
/// Returns [`DeserializeError`] if deserialization fails.
#[inline(always)]
pub fn unpack_in_place<'de, F, T, const SIZE_BYTES: usize>(
    place: &mut T,
    input: &'de [u8],
) -> Result<usize, DeserializeError>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F> + ?Sized,
{
    unpack_in_place_at_version::<F, T, SIZE_BYTES, 0>(place, input)
}
/// Version-selecting form of [`unpack_in_place`]. Uses `VERSION` without compatibility checks or a version tag.
#[inline(always)]
pub fn unpack_in_place_at_version<'de, F, T, const SIZE_BYTES: usize, const VERSION: u32>(
    place: &mut T,
    input: &'de [u8],
) -> Result<usize, DeserializeError>
where
    F: Formula<VERSION> + ?Sized,
    T: Deserialize<'de, F, VERSION> + ?Sized,
{
    let total = simple_try!(read_pack_size_at_version::<F, SIZE_BYTES, VERSION>(input));
    if input.len() < total {
        return Err(DeserializeError::OutOfBounds(total));
    }
    simple_try!(
        deserialize_in_place_at_version::<F, T, SIZE_BYTES, VERSION>(place, &input[..total])
    );
    Ok(total)
}

macro_rules! fixed_size_module {
    ($(#[$meta:meta])* $vis:vis mod $module:ident { $size_bytes:literal }) => {
        $(#[$meta])*
         $vis mod $module {
            use super::*;

            /// Packs value into bytes slice.
            /// Returns the number of bytes written.
            /// Fails if the buffer is too small.
            ///
            /// To retrieve the number of bytes required to serialize the value,
            /// use [`pack_size`] or [`pack_or_size`].
            ///
            /// Unlike [`serialize`] adds length prefix to the output unless formula has exact total size known in advance.
            /// Length prefix is encoded as unsigned integer with `SIZE_BYTES` bytes in little-endian format.
            /// Enabling calling [`unpack`] without slicing input data to the exact length returned by serialization function.
            /// This allows more convenient use with byte streams where end of input is not known in advance.
            ///
            /// # Errors
            ///
            /// Returns [`BufferExhausted`] if the buffer is too small.
            ///
            /// [`serialize`]: crate::serialize::serialize
            #[inline(always)]
pub fn pack<F, T>(
                value: &T,
                output: &mut [u8],
            ) -> Result<usize, BufferExhausted>
            where
                F: Formula + ?Sized,
                T: Serialize<F>,
            {
    pack_at_version::<F, T, 0>(value, output)
}
            /// Version-selecting form of [`pack`]. Uses `VERSION` without compatibility checks or a version tag.
            #[inline(always)]
            pub fn pack_at_version<F, T, const VERSION: u32>(
                value: &T,
                output: &mut [u8],
            ) -> Result<usize, BufferExhausted>
            where
                F: Formula<VERSION> + ?Sized,
                T: Serialize<F, VERSION>,
            {
                super::pack_at_version::<F, T, $size_bytes, VERSION>(value, output)
            }

            /// Slightly faster version of [`pack`].
            /// Panics if the buffer is too small instead of returning an error.
            ///
            /// Use instead of using [`pack`] with immediate [`unwrap`](Result::unwrap).
            ///
            /// Unlike [`serialize_unchecked`] adds length prefix to the output unless formula has exact total size known in advance.
            /// Length prefix is encoded as unsigned integer with `SIZE_BYTES` bytes in little-endian format.
            /// Enabling calling [`unpack`] without slicing input data to the exact length returned by serialization function.
            /// This allows more convenient use with byte streams where end of input is not known in advance.
            ///
            /// [`serialize_unchecked`]: crate::serialize::serialize_unchecked
            #[inline(always)]
pub fn pack_unchecked<F, T>(value: &T, output: &mut [u8]) -> usize
            where
                F: Formula + ?Sized,
                T: Serialize<F>,
            {
    pack_unchecked_at_version::<F, T, 0>(value, output)
}
            /// Version-selecting form of [`pack_unchecked`]. Uses `VERSION` without compatibility checks or a version tag.
            #[inline(always)]
            pub fn pack_unchecked_at_version<F, T, const VERSION: u32>(value: &T, output: &mut [u8]) -> usize
            where
                F: Formula<VERSION> + ?Sized,
                T: Serialize<F, VERSION>,
            {
                super::pack_unchecked_at_version::<F, T, $size_bytes, VERSION>(value, output)
            }

            /// Returns the number of bytes required to pack the value.
            /// Note that value is consumed.
            ///
            /// Use when value is `Copy` or can be cheaply replicated to allocate
            /// the buffer for serialization in advance.
            /// Or to find out required size after [`pack`] fails.
            ///
            /// Unlike [`serialized_size`] adds length prefix to the calculations unless formula has exact total size known in advance.
            /// Length prefix is encoded as unsigned integer with `SIZE_BYTES` bytes in little-endian format.
            /// Enabling calling [`unpack`] without slicing input data to the exact length returned by serialization function.
            /// This allows more convenient use with byte streams where end of input is not known in advance.
            ///
            /// [`serialized_size`]: crate::serialize::serialized_size
            #[inline(always)]
pub fn pack_size<F, T>(value: &T) -> usize
            where
                F: Formula + ?Sized,
                T: Serialize<F>,
            {
    pack_size_at_version::<F, T, 0>(value)
}
            /// Version-selecting form of [`pack_size`]. Uses `VERSION` without compatibility checks or a version tag.
            #[inline(always)]
            pub fn pack_size_at_version<F, T, const VERSION: u32>(value: &T) -> usize
            where
                F: Formula<VERSION> + ?Sized,
                T: Serialize<F, VERSION>,
            {
                super::pack_size_at_version::<F, T, $size_bytes, VERSION>(value)
            }

            /// Packs value into bytes slice.
            /// Returns the number of bytes written.
            ///
            /// If the buffer is too small, returns error that contains
            /// the exact number of bytes required.
            ///
            /// Use [`pack`] if this information is not needed.
            ///
            /// # Errors
            ///
            /// Returns [`BufferSizeRequired`] error if the buffer is too small.
            /// Error contains the exact number of bytes required.
            ///
            /// Unlike [`serialize_or_size`] adds length prefix to the output unless formula has exact total size known in advance.
            /// Length prefix is encoded as unsigned integer with `SIZE_BYTES` bytes in little-endian format.
            /// Enabling calling [`unpack`] without slicing input data to the exact length returned by serialization function.
            /// This allows more convenient use with byte streams where end of input is not known in advance.
            ///
            /// [`serialize_or_size`]: crate::serialize::serialize_or_size
            #[inline(always)]
pub fn pack_or_size<F, T>(
                value: &T,
                output: &mut [u8],
            ) -> Result<usize, BufferSizeRequired>
            where
                F: Formula + ?Sized,
                T: Serialize<F>,
            {
    pack_or_size_at_version::<F, T, 0>(value, output)
}
            /// Version-selecting form of [`pack_or_size`]. Uses `VERSION` without compatibility checks or a version tag.
            #[inline(always)]
            pub fn pack_or_size_at_version<F, T, const VERSION: u32>(
                value: &T,
                output: &mut [u8],
            ) -> Result<usize, BufferSizeRequired>
            where
                F: Formula<VERSION> + ?Sized,
                T: Serialize<F, VERSION>,
            {
                super::pack_or_size_at_version::<F, T, $size_bytes, VERSION>(value, output)
            }

            /// Packs value into byte vector.
            /// Returns the number of bytes written.
            ///
            /// Grows the vector if needed.
            /// Infallible except for allocation errors.
            ///
            /// Use pre-allocated vector when possible to avoid reallocations.
            ///
            /// Unlike [`serialize_to_vec`] adds length prefix to the output unless formula has exact total size known in advance.
            /// Length prefix is encoded as unsigned integer with `SIZE_BYTES` bytes in little-endian format.
            /// Enabling calling [`unpack`] without slicing input data to the exact length returned by serialization function.
            /// This allows more convenient use with byte streams where end of input is not known in advance.
            ///
            /// [`serialize_to_vec`]: crate::serialize::serialize_to_vec
            #[cfg(feature = "alloc")]
#[inline(always)]
pub fn pack_to_vec<F, T>(
                value: &T,
                output: &mut alloc::vec::Vec<u8>,
            ) -> usize
            where
                F: Formula + ?Sized,
                T: Serialize<F>,
            {
    pack_to_vec_at_version::<F, T, 0>(value, output)
}
            /// Version-selecting form of [`pack_to_vec`]. Uses `VERSION` without compatibility checks or a version tag.
            #[cfg(feature = "alloc")]
            #[inline(always)]
            pub fn pack_to_vec_at_version<F, T, const VERSION: u32>(
                value: &T,
                output: &mut alloc::vec::Vec<u8>,
            ) -> usize
            where
                F: Formula<VERSION> + ?Sized,
                T: Serialize<F, VERSION>,
            {
                super::pack_to_vec_at_version::<F, T, $size_bytes, VERSION>(value, output)
            }

            /// Returns the number of bytes of the packed value in the input.
            #[inline(always)]
pub fn read_pack_size<'de, F>(
                input: &[u8],
            ) -> Result<usize, DeserializeError>
            where
                F: Formula + ?Sized,
            {
    read_pack_size_at_version::<F, 0>(input)
}
            /// Version-selecting form of [`read_pack_size`]. Uses `VERSION` without compatibility checks or a version tag.
            #[inline(always)]
            pub fn read_pack_size_at_version<'de, F, const VERSION: u32>(
                input: &[u8],
            ) -> Result<usize, DeserializeError>
            where
                F: Formula<VERSION> + ?Sized,
            {
                super::read_pack_size_at_version::<F, $size_bytes, VERSION>(input)
            }

            /// Deserializes value from the input.
            /// Returns deserialized value.
            /// Unlike [`deserialize`] this function allows input to be longer than the length returned by packing function.
            /// If input is shorter than the length returned by packing function,
            /// this function will return [`DeserializeError::OutOfBounds`] error with the least number of bytes required to deserialize the value.
            ///
            /// This allows more convenient use with byte streams where end of input is not known in advance.
            ///
            /// # Errors
            ///
            /// Returns [`DeserializeError`] if deserialization fails.
            #[inline(always)]
pub fn unpack<'de, F, T>(input: &'de [u8]) -> Result<(T, usize), DeserializeError>
            where
                F: Formula + ?Sized,
                T: Deserialize<'de, F>,
            {
    unpack_at_version::<F, T, 0>(input)
}
            /// Version-selecting form of [`unpack`]. Uses `VERSION` without compatibility checks or a version tag.
            #[inline(always)]
            pub fn unpack_at_version<'de, F, T, const VERSION: u32>(input: &'de [u8]) -> Result<(T, usize), DeserializeError>
            where
                F: Formula<VERSION> + ?Sized,
                T: Deserialize<'de, F, VERSION>,
            {
                super::unpack_at_version::<F, T, $size_bytes, VERSION>(input)
            }

            /// Deserializes value from the input.
            /// Updates value in-place.
            /// Unlike [`deserialize`] this function allows input to be longer than the length returned by packing function.
            /// If input is shorter than the length returned by packing function,
            /// this function will return [`DeserializeError::OutOfBounds`] error with the least number of bytes required to deserialize the value.
            ///
            /// This allows more convenient use with byte streams where end of input is not known in advance.
            ///
            /// # Errors
            ///
            /// Returns [`DeserializeError`] if deserialization fails.
            #[inline(always)]
pub fn unpack_in_place<'de, F, T>(
                place: &mut T,
                input: &'de [u8],
            ) -> Result<usize, DeserializeError>
            where
                F: Formula + ?Sized,
                T: Deserialize<'de, F> + ?Sized,
            {
    unpack_in_place_at_version::<F, T, 0>(place, input)
}
            /// Version-selecting form of [`unpack_in_place`]. Uses `VERSION` without compatibility checks or a version tag.
            #[inline(always)]
            pub fn unpack_in_place_at_version<'de, F, T, const VERSION: u32>(
                place: &mut T,
                input: &'de [u8],
            ) -> Result<usize, DeserializeError>
            where
                F: Formula<VERSION> + ?Sized,
                T: Deserialize<'de, F, VERSION> + ?Sized,
            {
                super::unpack_in_place_at_version::<F, T, $size_bytes, VERSION>(place, input)
            }
        }
    };
}

fixed_size_module! {
    /// Deserialization functions for small data.
    ///
    /// They use only 1 byte to encode sizes and indirection,
    /// so max size is 255 bytes and max length of sequences is 255 elements,
    /// even if elements are zero-sized.
    pub mod small { 1 }
}

fixed_size_module! {
    /// Deserialization functions for medium data.
    ///
    /// They use only 2 bytes to encode sizes and indirection,
    /// so max size is 65535 bytes and max length of sequences is 65535 elements,
    /// even if elements are zero-sized.
    pub mod medium { 2 }
}

fixed_size_module! {
    /// Deserialization functions for large data.
    ///
    /// They use only 4 bytes to encode sizes and indirection,
    /// so max size is 4294967295 bytes and max length of sequences is 4294967295 elements,
    /// even if elements are zero-sized.
    pub mod large { 4 }
}

fixed_size_module! {
    /// Deserialization functions for huge data.
    ///
    /// They use 8 bytes to encode sizes and indirection,
    /// so max size is 18446744073709551615 bytes and max length of
    /// sequences is 18446744073709551615 elements, even if elements are zero-sized.
    pub mod huge { 8 }
}

fixed_size_module! {
    /// Deserialization functions for humongous data.
    ///
    /// They use 16 bytes to encode sizes and indirection,
    /// so max size is 340282366920938463463374607431768211455 bytes and max length of
    /// sequences is 340282366920938463463374607431768211455 elements, even if elements are zero-sized.
    pub mod humongous { 16 }
}
