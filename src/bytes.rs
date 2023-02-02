use core::mem::size_of;

use crate::{
    formula::{Formula, Serialize},
    FixedUsize,
};

/// `Formula` for runtime sized bytes array.
///
/// Packed from `impl `[`AsRef`]`<[u8]>`.
/// Unpacks into `&[`[`u8`]`]`.
///
/// Serialized exactly as [`Str`] and [`Seq<u8>`].
///
/// [`Seq<u8>`]: crate::Seq
/// [`Str`]: crate::Str
pub enum Bytes {}

impl Formula for Bytes {
    type Access<'a> = &'a [u8];

    #[inline(always)]
    fn header() -> usize {
        size_of::<[FixedUsize; 2]>()
    }

    #[inline(always)]
    fn has_body() -> bool {
        true
    }

    #[inline(always)]
    fn access<'a>(input: &'a [u8]) -> &'a [u8] {
        let len = &input[..size_of::<FixedUsize>()];
        let len = FixedUsize::from_bytes(len.try_into().unwrap()).into();

        let offset = &input[size_of::<FixedUsize>()..][..size_of::<FixedUsize>()];
        let offset = FixedUsize::from_bytes(offset.try_into().unwrap()).into();

        &input[offset..][..len]
    }
}

#[repr(transparent)]
pub struct BytesHeader {
    len: FixedUsize,
}

impl<T> Serialize<Bytes> for T
where
    T: AsRef<[u8]>,
{
    type Header = BytesHeader;

    #[inline(always)]
    fn serialize_body(self, output: &mut [u8]) -> Result<(BytesHeader, usize), usize> {
        let slice = self.as_ref();
        let len = slice.len();

        if output.len() < len {
            return Err(len);
        }

        output[..len].copy_from_slice(slice);

        Ok((
            BytesHeader {
                len: FixedUsize::truncated(len),
            },
            len,
        ))
    }

    #[inline(always)]
    fn body_size(self) -> usize {
        self.as_ref().len()
    }

    #[inline(always)]
    fn serialize_header(header: BytesHeader, output: &mut [u8], offset: usize) -> bool {
        if output.len() < size_of::<[FixedUsize; 2]>() {
            return false;
        }

        let offset = FixedUsize::truncated(offset);

        output[..size_of::<FixedUsize>()].copy_from_slice(&header.len.to_bytes());
        output[size_of::<FixedUsize>()..][..size_of::<FixedUsize>()]
            .copy_from_slice(&offset.to_bytes());

        true
    }
}
