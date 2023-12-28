use crate::{
    advanced::FixedUsizeType,
    buffer::{Buffer, BufferExhausted, CheckedFixedBuffer, DryBuffer, VecBuffer},
    deserialize::{read_reference, Deserialize, DeserializeError, Deserializer},
    formula::{reference_size, Formula},
    serialize::{write_ref, write_reference, Serialize, Sizes},
    size::SIZE_STACK,
};

/// Returns the number of bytes required to write packet with the value.
/// Note that value is consumed.
///
/// Use when value is `Copy` or can be cheaply replicated to allocate
/// the buffer for serialization in advance.
/// Or to find out required size after [`write_packet`] fails.
#[inline]
pub fn packet_size<F, T>(value: T) -> usize
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    match write_packet_into(value, DryBuffer) {
        Ok(size) => size,
        Err(never) => match never {},
    }
}

/// Writes packet with the value into buffer.
/// The buffer type controls bytes writing and failing strategy.
#[inline(always)]
pub fn write_packet_into<F, T, B>(value: T, mut buffer: B) -> Result<usize, B::Error>
where
    F: Formula + ?Sized,
    T: Serialize<F>,
    B: Buffer,
{
    let reference_size = reference_size::<F>();
    buffer.reserve_heap(0, 0, reference_size)?;

    let mut sizes = Sizes {
        heap: reference_size,
        stack: 0,
    };

    let size = write_ref(value, &mut sizes, buffer.reborrow())?;

    match buffer.reserve_heap(0, 0, reference_size)? {
        [] => {}
        reserved => {
            write_reference::<F, _>(size, sizes.heap, 0, 0, reserved).unwrap();
        }
    }

    Ok(sizes.heap)
}

/// Writes packet with the value into bytes slice.
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
pub fn write_packet<F, T>(value: T, output: &mut [u8]) -> Result<usize, BufferExhausted>
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    write_packet_into::<F, T, _>(value, CheckedFixedBuffer::new(output))
}

/// Writes packet with the value into bytes slice.
/// Slightly faster version of [`write_packet`].
/// Panics if the buffer is too small instead of returning an error.
///
/// Use instead of using [`write_packet`] with immediate [`unwrap`](Result::unwrap).
#[inline(always)]
pub fn write_packet_unchecked<F, T>(value: T, output: &mut [u8]) -> usize
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    match write_packet_into::<F, T, _>(value, output) {
        Ok(size) => size,
        Err(never) => match never {},
    }
}

/// Writes packet with the value into byte vector.
/// Returns the number of bytes written.
///
/// Grows the vector if needed.
/// Infallible except for allocation errors.
///
/// Use pre-allocated vector when possible to avoid reallocations.
#[cfg(feature = "alloc")]
#[inline(always)]
pub fn write_packet_to_vec<F, T>(value: T, output: &mut alloc::vec::Vec<u8>) -> usize
where
    F: Formula + ?Sized,
    T: Serialize<F>,
{
    match write_packet_into::<F, T, _>(value, VecBuffer::new(output)) {
        Ok(size) => size,
        Err(never) => match never {},
    }
}

/// Reads size of the packet with value from the input.
/// Returns `None` if the input is too short to determine the size.
///
/// # Panics
///
/// This function may panic if the value size is too big to fit `usize`.
#[must_use]
#[inline]
pub fn read_packet_size<F>(input: &[u8]) -> Option<usize>
where
    F: Formula + ?Sized,
{
    match F::MAX_STACK_SIZE {
        Some(0) => Some(0),
        _ => {
            if input.len() < SIZE_STACK {
                None
            } else {
                let mut bytes = [0u8; SIZE_STACK];
                bytes.copy_from_slice(&input[..SIZE_STACK]);
                let address = FixedUsizeType::from_le_bytes(bytes)
                    .try_into()
                    .expect("Value size can't fit `usize`");
                Some(address)
            }
        }
    }
}

/// Reads packet with value from the input.
/// Returns deserialized value and number of bytes consumed.
///
/// # Errors
///
/// Returns `DeserializeError` if deserialization fails.
#[inline]
pub fn read_packet<'de, F, T>(input: &'de [u8]) -> Result<(T, usize), DeserializeError>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F>,
{
    let reference_size = reference_size::<F>();

    if input.len() < reference_size {
        return Err(DeserializeError::OutOfBounds);
    }

    let (address, size) = read_reference::<F>(input, input.len() - reference_size);

    if size > address {
        return Err(DeserializeError::WrongAddress);
    }

    if address > input.len() {
        return Err(DeserializeError::OutOfBounds);
    }

    let de = Deserializer::new_unchecked(size, &input[..address]);
    let value = <T as Deserialize<'de, F>>::deserialize(de)?;

    Ok((value, address))
}

/// Reads packet with value from the input.
/// Updates the value in-place.
/// Returns number of bytes consumed.
///
/// # Errors
///
/// Returns `DeserializeError` if deserialization fails.
#[inline]
pub fn read_packet_in_place<'de, F, T>(
    place: &mut T,
    input: &'de [u8],
) -> Result<usize, DeserializeError>
where
    F: Formula + ?Sized,
    T: Deserialize<'de, F> + ?Sized,
{
    let reference_size = reference_size::<F>();

    if input.len() < reference_size {
        return Err(DeserializeError::OutOfBounds);
    }

    let (address, size) = read_reference::<F>(input, input.len() - reference_size);

    if size > address {
        return Err(DeserializeError::WrongAddress);
    }

    if address > input.len() {
        return Err(DeserializeError::OutOfBounds);
    }

    let de = Deserializer::new_unchecked(size, &input[..address]);
    <T as Deserialize<'de, F>>::deserialize_in_place(place, de)?;

    Ok(address)
}
