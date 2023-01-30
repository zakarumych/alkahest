use crate::schema::Schema;

/// Trait for types that can be serialized
/// into raw bytes with specified `S: `[`Schema`].
pub trait Serialize<T: Schema + ?Sized> {
    /// Serializes value into output buffer.
    /// Writes metadata at the end of the buffer.
    /// Writes payload at the beginning of the buffer.
    ///
    /// If serialized successfully `Ok((payload, metadata))` is returned
    /// with payload size and offset of the metadata written.
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
            Ok((payload, metadata)) => {
                debug_assert_eq!(
                    metadata, 0,
                    "`Serialize::serialize` returned `Ok((_, metadata))` where `metadata` offset exceeds buffer size"
                );
                debug_assert_eq!(
                    payload, 0,
                    "`Serialize::serialize` returned `Ok((payload, _))` where `payload` size exceeds buffer size"
                );
                0
            }
            Err(size) => size,
        }
    }
}

/// Serializes data into bytes slice.
/// Returns number of bytes written.
///
/// # Panics
///
/// Panics if value doesn't fit into bytes.
///
/// # Examples
///
/// ```
/// use alkahest::{Schema, Serialize, serialize, Seq};
///
/// #[derive(Schema)]
/// struct MySchema {
///   a: u8,
///   b: u16,
///   c: Seq<u32>,
/// }
///
/// let mut buffer = [0u8; 1 + 2 + 4 * 2 + 4 * 3]; // a - 1 byte, b - 2 bytes, c - 2 u32s for header + 3 u32s
///
/// let size = serialize::<MySchema, _>(MySchemaSerialize {
///   a: 1,
///   b: 2,
///   c: 3..6,
/// }, &mut buffer).unwrap();
///
/// assert_eq!(size, buffer.len());
/// ```
#[inline(always)]
pub fn serialize<T, S>(serializable: S, output: &mut [u8]) -> Result<usize, usize>
where
    T: Schema,
    S: Serialize<T>,
{
    todo!()
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

    // offset of the metadata
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

    /// Serialize a value according to specified schema.
    /// Value is written to the output buffer associated with this serializer.
    /// Serializer takes care of moving data around if necessary.
    #[inline]
    pub fn serialize<S, T>(&mut self, value: T) -> Result<(), usize>
    where
        S: Schema,
        T: Serialize<S>,
    {
        match value.serialize(
            self.offset + self.payload,
            &mut self.output[self.payload..self.metadata],
        ) {
            Ok((metadata, payload)) => {
                self.payload += payload;
                self.metadata -= metadata;
                Ok(())
            }
            Err(size) => Err(size),
        }
    }

    /// Returns offset of the output buffer from the "start".
    /// Metadata should use this offset to point to the payload location.
    #[inline]
    pub fn offset(&self) -> usize {
        self.offset + self.payload
    }

    /// Finishes serialization.
    /// Performs final metadata move in the buffer to the end of the payload.
    /// Returns size of the metadata and payload.
    #[inline]
    fn flush(self) -> usize {
        let Self {
            output,
            payload,
            metadata,
            ..
        } = self;

        // Copy last `metadata` bytes to the end of `payload`
        output.copy_within(metadata.., payload);
        self.payload + (output.len() - metadata)
    }

    /// Finishes serialization without moving metadata
    /// to the end of the payload.
    ///
    /// Returns size of the payload and offset of the metadata.
    #[inline]
    fn finish(self) -> (usize, usize) {
        let Self {
            output,
            payload,
            metadata,
            ..
        } = self;

        (payload, metadata)
    }
}
