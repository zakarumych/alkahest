//!
//! This module provides schema for serializing unsized types through a reference.
//!

use core::marker::PhantomData;

/// `Ref` is a schema wrapper.
/// It serializes the value in dynamic payload
/// and stores relative offset and the ref metadata.
/// Metadata is required for unsized types and is `()` for all sized types.
/// The `slice` type is unsized type that uses length metadata.
/// Structures allows last field to be of unsized type. In this case
/// metadata of the field inherited by the struct.
pub struct Ref<T: ?Sized> {
    marker: PhantomData<fn() -> T>,
}

impl<T> Schema for Ref<T> where T: Schema {}

impl<S, T> Serialize<Ref<S>> for T
where
    T: Serialize<S>,
{
    #[inline(always)]
    fn serialize(self, offset: usize, output: &mut [u8]) -> Result<(usize, usize), usize> {
        let mut ser = Serializer::new(offset, output);
        if let Err(size) = ser.serialize(self) {
            return Err(size + size_of::<FixedUsize>());
        }
        let size = ser.flush();
        let mut ser = Serializer::new(offset + size, &mut output[size..]);
        if let Err(ref_size) = ser.serialize(FixedUsize::truncate(offset + size)) {
            return Err(ref_size + size);
        }
        let (payload, metadata) = ser.finish();
        Ok((payload, metadata))
    }
}
