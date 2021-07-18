/// The purpose of this trait is to define associated type for each lifetime for `Parcel` trait implementations.
/// This is a workaround for lack of HRTB support for associated types.
pub trait SchemaUnpack<'a> {
    /// Unpacked value type.
    type Unpacked;
}

/// Trait for data schemas.
///
/// This trait requires implementation of `SchemaUnpacked` trait for all possible lifetimes.
pub trait Schema: for<'a> SchemaUnpack<'a> {
    /// Packed value with this schema.
    /// Trivially readable from and writable to bytes.
    type Packed: bytemuck::Pod;

    /// Alignment required for successfully unpacking.
    /// See [`unpack`] method.
    fn align() -> usize;

    /// Unpack the value from packed value and bytes.
    fn unpack<'a>(packed: Self::Packed, bytes: &'a [u8]) -> Unpacked<'a, Self>;
}

/// Trait for packable types with that match specified schema.
pub trait Pack<T: Schema> {
    /// Packs into trivially serializable value.
    ///
    /// Returns packed data and number of bytes used from `bytes` storage.
    fn pack(self, offset: usize, bytes: &mut [u8]) -> (Packed<T>, usize);
}

/// Type alias for packed value type of `T`.
pub type Packed<T> = <T as Schema>::Packed;

/// Type alias for unpacked value type of `T`.
pub type Unpacked<'a, T> = <T as SchemaUnpack<'a>>::Unpacked;
