use crate::{
    Deserialize, DeserializeError, Element, Serialize, SizeBound, Sizes,
    advanced::make_serializer,
    buffer::{Buffer, BufferExhausted, CheckedFixedBuffer, DryBuffer, MaybeFixedBuffer},
    deserialize::{deserialize, deserialize_in_place, read_usize},
    element::{heap_size, stack_size},
    serialize::{BufferSizeRequired, write_usize},
};

#[inline]
const fn total<E, const SIZE_BYTES: usize>() -> Option<usize>
where
    E: Element + ?Sized,
{
    match (stack_size::<E, SIZE_BYTES>(), heap_size::<E, SIZE_BYTES>()) {
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
pub fn pack_into<E, T, B, const SIZE_BYTES: usize>(
    value: &T,
    mut buffer: B,
) -> Result<usize, B::Error>
where
    E: Element + ?Sized,
    T: Serialize<E::Formula>,
    B: Buffer,
{
    let total = total::<E, SIZE_BYTES>();

    if total.is_none() {
        // Pre-reserve space for length prefix if total size is not known in advance.
        let _ = simple_try!(buffer.reserve_heap(0, 0, SIZE_BYTES));
    }

    // Initialize sizes with first `SIZE_BYTES` bytes reserved for length prefix
    // unless total size is known in advance.
    let mut sizes = Sizes {
        heap: total.map_or(SIZE_BYTES, |_| 0),
        stack: 0,
    };

    {
        let mut serializer = make_serializer::<_, SIZE_BYTES>(buffer.reborrow(), &mut sizes);
        simple_try!(E::serialize(value, &mut serializer));
    }

    // Move stack to the heap to make serialized data contiguous.
    buffer.move_to_heap(sizes.heap, sizes.stack, sizes.stack);

    let actual = sizes.total();

    match total {
        None => {
            let reserved = match buffer.reserve_heap(0, 0, SIZE_BYTES) {
                Ok(reserved) => reserved,
                Err(_err) => {
                    unreachable!("Failed to reserve space for length prefix that was pre-reserved");
                }
            };

            write_usize::<_, SIZE_BYTES>(actual, 0, reserved);
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
#[inline]
pub fn pack<E, T, const SIZE_BYTES: usize>(
    value: &T,
    output: &mut [u8],
) -> Result<usize, BufferExhausted>
where
    E: Element + ?Sized,
    T: Serialize<E::Formula>,
{
    pack_into::<E, T, _, SIZE_BYTES>(value, CheckedFixedBuffer::new(output))
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
#[inline]
pub fn pack_unchecked<E, T, const SIZE_BYTES: usize>(value: &T, output: &mut [u8]) -> usize
where
    E: Element + ?Sized,
    T: Serialize<E::Formula>,
{
    match pack_into::<E, T, _, SIZE_BYTES>(value, output) {
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
#[inline]
pub fn pack_size<E, T, const SIZE_BYTES: usize>(value: &T) -> usize
where
    E: Element + ?Sized,
    T: Serialize<E::Formula>,
{
    match pack_into::<E, T, _, SIZE_BYTES>(value, DryBuffer) {
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
#[inline]
pub fn pack_or_size<E, T, const SIZE_BYTES: usize>(
    value: &T,
    output: &mut [u8],
) -> Result<usize, BufferSizeRequired>
where
    E: Element + ?Sized,
    T: Serialize<E::Formula>,
{
    let mut exhausted = false;
    let result =
        pack_into::<E, T, _, SIZE_BYTES>(value, MaybeFixedBuffer::new(output, &mut exhausted));
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
#[inline]
pub fn pack_to_vec<E, T, const SIZE_BYTES: usize>(
    value: &T,
    output: &mut alloc::vec::Vec<u8>,
) -> usize
where
    E: Element + ?Sized,
    T: Serialize<E::Formula>,
{
    use crate::buffer::VecBuffer;

    match pack_into::<E, T, _, SIZE_BYTES>(value, VecBuffer::new(output)) {
        Ok(size) => size,
        Err(never) => match never {},
    }
}

/// Returns the number of bytes of the packed value in the input.
#[inline]
pub fn read_pack_size<E, const SIZE_BYTES: usize>(input: &[u8]) -> Result<usize, DeserializeError>
where
    E: Element + ?Sized,
{
    let total = total::<E, SIZE_BYTES>();

    match total {
        None => match input.first_chunk::<SIZE_BYTES>() {
            None => Err(DeserializeError::OutOfBounds(SIZE_BYTES)),
            Some(bytes) => read_usize::<SIZE_BYTES>(bytes),
        },
        Some(total) => Ok(total),
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
pub fn unpack<'de, E, T, const SIZE_BYTES: usize>(
    input: &'de [u8],
) -> Result<(T, usize), DeserializeError>
where
    E: Element + ?Sized,
    T: Deserialize<'de, E::Formula>,
{
    let total = simple_try!(read_pack_size::<E, SIZE_BYTES>(input));
    if input.len() < total {
        return Err(DeserializeError::OutOfBounds(total));
    }
    let value = simple_try!(deserialize::<E, T, SIZE_BYTES>(&input[..total]));
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
#[inline]
pub fn unpack_in_place<'de, E, T, const SIZE_BYTES: usize>(
    place: &mut T,
    input: &'de [u8],
) -> Result<usize, DeserializeError>
where
    E: Element + ?Sized,
    T: Deserialize<'de, E::Formula> + ?Sized,
{
    let total = simple_try!(read_pack_size::<E, SIZE_BYTES>(input));
    if input.len() < total {
        return Err(DeserializeError::OutOfBounds(total));
    }
    simple_try!(deserialize_in_place::<E, T, SIZE_BYTES>(
        place,
        &input[..total]
    ));
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
            #[inline]
            pub fn pack<E, T>(
                value: &T,
                output: &mut [u8],
            ) -> Result<usize, BufferExhausted>
            where
                E: Element + ?Sized,
                T: Serialize<E::Formula>,
            {
                super::pack::<E, T, $size_bytes>(value, output)
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
            #[inline]
            pub fn pack_unchecked<E, T>(value: &T, output: &mut [u8]) -> usize
            where
                E: Element + ?Sized,
                T: Serialize<E::Formula>,
            {
                super::pack_unchecked::<E, T, $size_bytes>(value, output)
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
            #[inline]
            pub fn pack_size<E, T>(value: &T) -> usize
            where
                E: Element + ?Sized,
                T: Serialize<E::Formula>,
            {
                super::pack_size::<E, T, $size_bytes>(value)
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
            #[inline]
            pub fn pack_or_size<E, T>(
                value: &T,
                output: &mut [u8],
            ) -> Result<usize, BufferSizeRequired>
            where
                E: Element + ?Sized,
                T: Serialize<E::Formula>,
            {
                super::pack_or_size::<E, T, $size_bytes>(value, output)
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
            #[inline]
            pub fn pack_to_vec<E, T>(
                value: &T,
                output: &mut alloc::vec::Vec<u8>,
            ) -> usize
            where
                E: Element + ?Sized,
                T: Serialize<E::Formula>,
            {
                super::pack_to_vec::<E, T, $size_bytes>(value, output)
            }

            /// Returns the number of bytes of the packed value in the input.
            #[inline]
            pub fn read_pack_size<'de, E>(
                input: &[u8],
            ) -> Result<usize, DeserializeError>
            where
                E: Element + ?Sized,
            {
                super::read_pack_size::<E, $size_bytes>(input)
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
            #[inline]
            pub fn unpack<'de, E, T>(input: &'de [u8]) -> Result<(T, usize), DeserializeError>
            where
                E: Element + ?Sized,
                T: Deserialize<'de, E::Formula>,
            {
                super::unpack::<E, T, $size_bytes>(input)
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
            #[inline]
            pub fn unpack_in_place<'de, E, T>(
                place: &mut T,
                input: &'de [u8],
            ) -> Result<usize, DeserializeError>
            where
                E: Element + ?Sized,
                T: Deserialize<'de, E::Formula> + ?Sized,
            {
                super::unpack_in_place::<E, T, $size_bytes>(place, input)
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
