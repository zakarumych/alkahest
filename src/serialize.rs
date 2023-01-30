use crate::schema::Schema;

/// Trait for types that can be serialized
/// into raw bytes with specified `S: `[`Schema`].
pub trait Serialize<T: Schema + ?Sized> {
    /// Serializes value into output buffer.
    /// Writes metadata at the end of the buffer.
    /// Writes payload at the beginning of the buffer.
    ///
    /// If serialized successfully `Ok((payload, metadata))` is returned.
    /// Where `payload` is size of the payload in bytes written to the beginning of the buffer.
    /// And `metadata` is start of the metadata in bytes written to the end of the buffer.
    ///
    /// Implementations *must* write data according to the schema.
    /// Doing otherwise may result in errors during deserialization.
    /// Where errors may be both failures to deserialize and
    /// incorrect deserialized values.
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
            Ok((_payload, _metadata)) => 0,
            Err(size) => size,
        }
    }
}

/// Wraps output buffer and provides methods for serializing data.
/// Implementors of `Serialize` trait may use this type.
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
    pub fn written(&self) -> usize {
        self.payload + (self.output.len() - self.metadata)
    }

    /// Serialize a value according to specified schema.
    /// Value is written to the output buffer associated with this serializer.
    /// Serializer takes care of moving data around if necessary.
    #[inline]
    pub fn put<S, T>(&mut self, value: T) -> Result<(), usize>
    where
        S: Schema + ?Sized,
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
            Err(size) => Err(size),
        }
    }

    /// Returns offset of the output buffer from the "start".
    /// Metadata should use this offset to point to the payload location.
    #[inline(always)]
    pub fn offset(&self) -> usize {
        self.offset + self.payload
    }

    /// Flushes metadata to the end of the payload.
    #[inline(always)]
    pub fn flush(&mut self) {
        self.output.copy_within(self.metadata.., self.payload);
        self.payload += self.output.len() - self.metadata;
        self.metadata = self.output.len();
    }

    /// Ends writing to the output buffer.
    /// Returns payload size and start of the metadata.
    #[inline(always)]
    pub fn finish(self) -> (usize, usize) {
        (self.payload, self.metadata)
    }
}

pub fn serialize<S, T>(value: T, output: &mut [u8]) -> Result<usize, usize>
where
    S: Schema + ?Sized,
    T: Serialize<S>,
{
    let mut serializer = Serializer::new(0, output);
    serializer.put(value)?;
    serializer.flush();
    Ok(serializer.finish().0)
}
