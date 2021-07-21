use {
    crate::{
        schema::{Pack, Schema, SchemaUnpack},
        FixedUsize,
    },
    core::{convert::TryFrom, mem::align_of},
};

/// `Schema` for runtime sized bytes array.
/// Should be used for bytes and strings alike.
///
/// Packed from `impl `[`AsRef`]`<[u8]>`.
/// Unpacks into `&[`[`u8`]`]`.
///
/// Serialized exactly as [`Seq<u8>`].
///
/// [`Seq<u8>`]: crate::Seq
pub struct Bytes;

impl<'a> SchemaUnpack<'a> for Bytes {
    type Unpacked = &'a [u8];
}

impl Schema for Bytes {
    type Packed = [FixedUsize; 2];

    fn align() -> usize {
        align_of::<[FixedUsize; 2]>()
    }

    fn unpack<'a>(packed: [FixedUsize; 2], bytes: &'a [u8]) -> &'a [u8] {
        let len = usize::try_from(packed[0]).expect("Slice is too large");
        let offset = usize::try_from(packed[1]).expect("Package is too large");
        &bytes[offset..][..len]
    }
}

impl<T> Pack<Bytes> for T
where
    T: AsRef<[u8]>,
{
    #[inline]
    fn pack(self, offset: usize, output: &mut [u8]) -> ([FixedUsize; 2], usize) {
        let bytes = self.as_ref();

        let len32 = u32::try_from(bytes.len()).expect("Slice is too large");
        let offset32 = u32::try_from(offset).expect("Offset is too large");

        output[..bytes.len()].copy_from_slice(bytes);
        ([len32, offset32], bytes.len())
    }
}
