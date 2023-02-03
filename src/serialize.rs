use core::mem::size_of;

use crate::{formula::UnsizedFormula, size::FixedUsize, Formula};

/// Trait for types that can be serialized
/// into raw bytes with specified `F: `[`Formula`].
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
pub struct Serializer<'de> {
    /// Offset of the output buffer from the "start".
    offset: usize,

    /// Output buffer sub-slice usable for serialization.
    output: &'de mut [u8],

    // size of the heap
    heap: usize,

    // start of the stack
    stack: usize,
}

impl<'de> Serializer<'de> {
    pub fn new(offset: usize, output: &'de mut [u8]) -> Self {
        Self {
            offset,
            heap: 0,
            stack: output.len(),
            output,
        }
    }

    #[inline(always)]
    fn written(&self) -> usize {
        self.heap + (self.output.len() - self.stack)
    }

    /// Serialize a value according to specified formula.
    /// Value is written to the output buffer associated with this serializer.
    /// Serializer takes care of moving data around if necessary.
    #[inline]
    pub fn serialize_unsized<F, T>(&mut self, value: T) -> Result<(), usize>
    where
        F: UnsizedFormula + ?Sized,
        T: Serialize<F>,
    {
        match value.serialize(
            self.offset + self.heap,
            &mut self.output[self.heap..self.stack],
        ) {
            Ok((heap, stack)) => {
                self.stack = self.heap + stack;
                self.heap += heap;
                Ok(())
            }
            Err(size) => Err(size + self.written()),
        }
    }

    /// Serialize a value according to specified formula.
    /// Value is written to the output buffer associated with this serializer.
    /// Serializer takes care of moving data around if necessary.
    #[inline]
    pub fn serialize_sized<F, T>(&mut self, value: T) -> Result<(), usize>
    where
        F: Formula + ?Sized,
        T: Serialize<F>,
    {
        match value.serialize(
            self.offset + self.heap,
            &mut self.output[self.heap..self.stack],
        ) {
            Ok((heap, stack)) => {
                debug_assert!(
                    stack <= F::SIZE,
                    "Incorrect `Serialize` implementation consumes more than `Formula::SIZE` bytes"
                );
                self.stack = self.heap + stack.max(F::SIZE); // Padding.
                self.heap += heap;
                Ok(())
            }
            Err(size) => Err(size + self.written()),
        }
    }

    /// Wastes `size` bytes on stack.
    #[inline]
    pub fn waste(&mut self, size: usize) -> Result<(), usize> {
        if self.output.len() - self.stack < size {
            return Err(size + self.written());
        }

        self.stack -= size;
        Ok(())
    }

    /// Serialize a value according to specified formula.
    /// Value is written to the output buffer associated with this serializer.
    /// Serializer takes care of moving data around if necessary.
    #[inline]
    pub fn serialize_self<T>(&mut self, value: T) -> Result<(), usize>
    where
        T: Formula + Serialize<T>,
    {
        self.serialize_sized::<T, T>(value)
    }

    /// Moves "stack" to "heap".
    /// Returns address of the value moved from "stack" and its size.
    #[inline(always)]
    #[must_use]
    pub fn flush(&mut self) -> (usize, usize) {
        let meta_size = self.output.len() - self.stack;
        self.output.copy_within(self.stack.., self.heap);
        self.heap += meta_size;
        self.stack = self.output.len();
        (self.heap + self.offset, meta_size)
    }

    /// Ends writing to the output buffer.
    /// Returns heap size and start of the stack.
    #[inline(always)]
    #[must_use]
    pub fn finish(self) -> (usize, usize) {
        (self.heap, self.stack)
    }
}

pub fn serialize<F, T>(value: T, output: &mut [u8]) -> Result<usize, usize>
where
    F: UnsizedFormula + ?Sized,
    T: Serialize<F>,
{
    if output.len() < HEADER_SIZE {
        return Err(HEADER_SIZE + value.size());
    }

    let mut ser = Serializer::new(HEADER_SIZE, &mut output[HEADER_SIZE..]);

    ser.serialize_unsized::<F, T>(value)
        .map_err(|size| size + HEADER_SIZE)?;
    let (address, size) = ser.flush();

    output[..FIELD_SIZE].copy_from_slice(&FixedUsize::truncate_unchecked(address).to_le_bytes());
    output[FIELD_SIZE..][..FIELD_SIZE]
        .copy_from_slice(&FixedUsize::truncate_unchecked(size).to_le_bytes());

    // Nothing is written beyond address of top-level value.
    Ok(address)
}

pub fn serialized_size<F, T>(value: T) -> usize
where
    F: UnsizedFormula + ?Sized,
    T: Serialize<F>,
{
    <T as Serialize<F>>::size(value) + HEADER_SIZE
}

const FIELD_SIZE: usize = size_of::<FixedUsize>();
const HEADER_SIZE: usize = FIELD_SIZE * 2;
