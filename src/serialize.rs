use core::mem::size_of;

use crate::{
    formula::{FormulaAlias, UnsizedFormula},
    size::FixedUsize,
};

/// Trait for types that can be serialized
/// into raw bytes with specified `S: `[`Formula`].
///
/// Implementations *must* write data according to the formula.
/// Doing otherwise may result in errors during deserialization.
/// Where errors may be both failures to deserialize and
/// incorrect deserialized values.
pub trait Serialize<T: UnsizedFormula + ?Sized> {
    /// Serializes value into output buffer.
    /// Writes data to "heap" and "stack".
    /// "Heap" grows from the beginning of the buffer.
    /// "Stack" grows back from the end of the buffer.
    ///
    /// If serialized successfully `Ok((heap, stack))` is returned.
    /// Where `heap` is size in bytes written to the "heap".
    /// And `stack` is the new start of the "stack".
    fn serialize(self, offset: usize, output: &mut [u8]) -> Result<(usize, usize), usize>;

    /// Returns size of buffer in bytes required to serialize the value.
    ///
    /// Implementations *should* override this method whenever faster
    /// implementation is possible.
    #[inline]
    fn size(self) -> usize
    where
        Self: Sized,
    {
        match self.serialize(0, &mut []) {
            Ok(_) => 0,
            Err(size) => size,
        }
    }
}

/// Wraps output buffer and provides methods for serializing data.
/// Implementors of `Serialize` trait may use this type.
#[must_use]
pub struct Serializer<'a> {
    /// Offset of the output buffer from the "start".
    offset: usize,

    /// Output buffer sub-slice usable for serialization.
    output: &'a mut [u8],

    // size of the payload
    payload: usize,

    // start of the metadata
    metadata: usize,
}

impl<'a> Serializer<'a> {
    pub fn new(offset: usize, output: &'a mut [u8]) -> Self {
        Self {
            offset,
            payload: 0,
            metadata: output.len(),
            output,
        }
    }

    #[inline(always)]
    fn written(&self) -> usize {
        self.payload + (self.output.len() - self.metadata)
    }

    /// Serialize a value according to specified formula.
    /// Value is written to the output buffer associated with this serializer.
    /// Serializer takes care of moving data around if necessary.
    #[inline]
    pub fn serialize_value<S, T>(&mut self, value: T) -> Result<(), usize>
    where
        S: UnsizedFormula + ?Sized,
        T: Serialize<S>,
    {
        match value.serialize(
            self.offset + self.payload,
            &mut self.output[self.payload..self.metadata],
        ) {
            Ok((payload, metadata)) => {
                self.metadata = self.payload + metadata;
                self.payload += payload;
                Ok(())
            }
            Err(size) => Err(size + self.written()),
        }
    }

    /// Serialize a value according to specified formula.
    /// Value is written to the output buffer associated with this serializer.
    /// Serializer takes care of moving data around if necessary.
    #[inline]
    pub fn serialize_self<T>(&mut self, value: T) -> Result<(), usize>
    where
        T: UnsizedFormula + Serialize<T>,
    {
        self.serialize_value::<T, T>(value)
    }

    /// Returns offset of the output buffer from the "start".
    #[inline(always)]
    #[must_use]
    pub fn offset(&self) -> usize {
        self.offset + self.payload
    }

    /// Moves "stack" to "heap".
    /// Returns address of the value moved from "stack" and its size.
    #[inline(always)]
    #[must_use]
    pub fn flush(&mut self) -> (usize, usize) {
        let meta_size = self.output.len() - self.metadata;
        self.output.copy_within(self.metadata.., self.payload);
        self.payload += meta_size;
        self.metadata = self.output.len();
        (self.payload + self.offset, meta_size)
    }

    /// Ends writing to the output buffer.
    /// Returns payload size and start of the metadata.
    #[inline(always)]
    #[must_use]
    pub fn finish(self) -> (usize, usize) {
        (self.payload, self.metadata)
    }
}

pub fn serialize<S, T>(value: T, output: &mut [u8]) -> Result<usize, usize>
where
    S: UnsizedFormula + ?Sized,
    T: Serialize<S>,
{
    if output.len() < HEADER_SIZE {
        return Err(HEADER_SIZE + value.size());
    }

    let mut ser = Serializer::new(HEADER_SIZE, &mut output[HEADER_SIZE..]);

    ser.serialize_value::<S, T>(value)
        .map_err(|size| size + HEADER_SIZE)?;
    let (address, size) = ser.flush();

    output[..FIELD_SIZE].copy_from_slice(&FixedUsize::truncated(address).to_le_bytes());
    output[FIELD_SIZE..][..FIELD_SIZE].copy_from_slice(&FixedUsize::truncated(size).to_le_bytes());

    // Nothing is written beyond address of top-level value.
    Ok(address)
}

pub fn serialized_size<S, T>(value: T) -> usize
where
    S: UnsizedFormula + ?Sized,
    T: Serialize<S>,
{
    <T as Serialize<S>>::size(value) + HEADER_SIZE
}

const FIELD_SIZE: usize = size_of::<FixedUsize>();
const HEADER_SIZE: usize = FIELD_SIZE * 2;

impl<T, S, A> Serialize<S> for T
where
    S: FormulaAlias<Alias = A>,
    A: UnsizedFormula,
    T: Serialize<A>,
{
    fn serialize(self, offset: usize, output: &mut [u8]) -> Result<(usize, usize), usize> {
        <T as Serialize<A>>::serialize(self, offset, output)
    }

    fn size(self) -> usize {
        <T as Serialize<A>>::size(self)
    }
}
