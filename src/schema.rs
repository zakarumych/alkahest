/// Trait for data schemas.
///
/// This trait requires implementation of [`SchemaUnpack`] trait for all lifetimes.
pub trait Schema {
    /// Type of the unpacked value returned from [`SchemaUnpack::unpack`].
    type Access<'a>;

    // Header size in bytes.
    fn header() -> usize;

    /// Returns `true` if schema allows values having a body.
    /// If this method returns `false`, [`Serialize::serialize_body`] should write 0 bytes.
    fn has_body() -> bool {
        true
    }

    /// Deserializes the value from bytes.
    /// `input` must be aligned according to [`Self::align`].
    /// `input` must be at least [`Self::header`] bytes long.
    fn access<'a>(input: &'a [u8]) -> Access<'a, Self>;
}

/// Trait for types that can be serialized with specified [`Schema`].
pub trait Serialize<T: Schema> {
    /// Value produced by [`Serialize::serialize_body`] method.
    /// Must be passed into [`Serialize::serialize_header`] method.
    type Header;

    /// Serializes body of the value in streaming fashion.
    ///
    /// If serialized successfully `Ok((header, size))` is returned with size of serialized value.
    /// If the output buffer is too small `Err(size)` is returned where `size` is the size in bytes required to write the value.
    fn serialize_body(self, output: &mut [u8]) -> Result<(Self::Header, usize), usize>;

    /// Serializes header of the value.
    /// Header serialization cannot be streamed.
    /// Size of the header is known in advance from [`Schema::header`].
    /// `offset` is the offset of the header relative to beginning of the body.
    ///
    /// Returns `true` if the header was successfully serialized.
    /// Returns `false` if the output buffer is too small.
    fn serialize_header(header: Self::Header, output: &mut [u8], offset: usize) -> bool;

    /// Returns size of buffer required to serialize the value's body.
    ///
    /// Implementations *should* override this method for performance reasons.
    #[inline]
    fn body_size(self) -> usize
    where
        Self: Sized,
    {
        match self.serialize_body(&mut []) {
            Ok((_, size)) => {
                debug_assert_eq!(
                    size, 0,
                    "`Serialize::serialize` returned `Ok((_, size))` where `size` > buffer size"
                );
                0
            }
            Err(size) => size,
        }
    }
}

/// Type alias for Access value with [`Schema`] of type `T`.
pub type Access<'a, T> = <T as Schema>::Access<'a>;
