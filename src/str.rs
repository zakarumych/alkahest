use {
    crate::{
        schema::{Pack, Schema, SchemaUnpack},
        FixedUsize,
    },
    core::{convert::TryFrom, mem::align_of},
};

#[cfg(feature = "alloc")]
use crate::schema::SchemaOwned;

/// `Schema` for runtime sized bytes array.
/// Should be used for bytes and strings alike.
///
/// Packed from `impl `[`AsRef`]`<str>`.
/// Unpacks into `&[`str`]`.
///
/// Serialized exactly as [`Bytes`] and [`Seq<u8>`].
///
/// [`Seq<u8>`]: crate::Seq
/// [`Bytes`]: crate::Bytes
#[cfg_attr(feature = "alloc", repr(transparent))]
pub struct Str {
    #[cfg(feature = "alloc")]
    string: alloc::boxed::Box<str>,
}

#[cfg(feature = "alloc")]
impl core::ops::Deref for Str {
    type Target = str;

    fn deref(&self) -> &str {
        &*self.string
    }
}

#[cfg(feature = "alloc")]
impl core::ops::DerefMut for Str {
    fn deref_mut(&mut self) -> &mut str {
        &mut *self.string
    }
}

#[cfg(feature = "alloc")]
impl Str {
    pub fn into_inner(self) -> alloc::boxed::Box<str> {
        self.string
    }
}

impl<'a> SchemaUnpack<'a> for Str {
    type Unpacked = &'a str;
}

impl Schema for Str {
    type Packed = [FixedUsize; 2];

    fn align() -> usize {
        align_of::<[FixedUsize; 2]>()
    }

    fn unpack<'a>(packed: [FixedUsize; 2], bytes: &'a [u8]) -> &'a str {
        let len = usize::try_from(packed[0]).expect("Slice is too large");
        let offset = usize::try_from(packed[1]).expect("Package is too large");
        core::str::from_utf8(&bytes[offset..][..len]).unwrap()
    }
}

impl<T> Pack<Str> for T
where
    T: AsRef<str>,
{
    #[inline]
    fn pack(self, offset: usize, output: &mut [u8]) -> ([FixedUsize; 2], usize) {
        let bytes = self.as_ref().as_bytes();

        let len32 = u32::try_from(bytes.len()).expect("Slice is too large");
        let offset32 = u32::try_from(offset).expect("Offset is too large");

        output[..bytes.len()].copy_from_slice(bytes);
        ([len32, offset32], bytes.len())
    }
}

#[cfg(feature = "alloc")]
impl SchemaOwned for Str {
    #[inline(always)]
    fn to_owned_schema<'a>(unpacked: &'a str) -> Str {
        Str {
            string: unpacked.into(),
        }
    }
}
