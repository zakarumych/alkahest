use {
    crate::{
        schema::{Pack, Schema, SchemaUnpack},
        FixedUsize,
    },
    core::{convert::TryFrom, mem::align_of},
};

#[derive(Clone, Copy)]
pub struct BytesUnpacked<'a> {
    offset: usize,
    len: usize,
    bytes: &'a [u8],
}

pub struct Bytes;

impl<'a> SchemaUnpack<'a> for Bytes {
    type Unpacked = BytesUnpacked<'a>;
}

impl Schema for Bytes {
    type Packed = [FixedUsize; 2];

    fn align() -> usize {
        align_of::<[FixedUsize; 2]>()
    }

    fn unpack<'a>(packed: [FixedUsize; 2], bytes: &'a [u8]) -> BytesUnpacked<'a> {
        BytesUnpacked {
            len: usize::try_from(packed[0]).expect("Bytesuence is too large"),
            offset: usize::try_from(packed[1]).expect("Package is too large"),
            bytes,
        }
    }
}

impl<'a> BytesUnpacked<'a> {
    /// View raw bytes slice.
    pub fn as_slice(&self) -> &[u8] {
        &self.bytes[self.offset..][..self.len]
    }

    /// View bytes as a string.
    pub fn as_str(&self) -> Result<&str, core::str::Utf8Error> {
        core::str::from_utf8(&self.bytes[self.offset..][..self.len])
    }
}

impl<T> Pack<Bytes> for T
where
    T: AsRef<[u8]>,
{
    fn pack(self, offset: usize, output: &mut [u8]) -> ([FixedUsize; 2], usize) {
        let bytes = self.as_ref();

        let len32 = u32::try_from(bytes.len()).expect("Sequence is too large");
        let offset32 = u32::try_from(offset).expect("Sequence is too large");

        output[..bytes.len()].copy_from_slice(bytes);
        ([len32, offset32], bytes.len())
    }
}
