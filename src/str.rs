use crate::{
    bytes::{Bytes, BytesHeader},
    formula::{Formula, Serialize},
};

/// `Formula` for strings.
///
/// Serialized from `impl `[`AsRef`]`<str>`.
/// Access into `&[str]`.
///
/// Serialized exactly as [`Bytes`] and [`Seq<u8>`].
///
/// [`Seq<u8>`]: crate::Seq
/// [`Bytes`]: Bytes
pub enum Str {}

impl Formula for Str {
    type Access<'a> = &'a str;

    #[inline(always)]
    fn header() -> usize {
        <Bytes as Formula>::header()
    }

    #[inline(always)]
    fn has_body() -> bool {
        <Bytes as Formula>::has_body()
    }

    #[inline(always)]
    fn access<'a>(input: &'a [u8]) -> &'a str {
        let bytes = <Bytes as Formula>::access(input);
        core::str::from_utf8(bytes).expect("invalid utf8")
    }
}

impl<T> Serialize<Str> for T
where
    T: AsRef<str>,
{
    type Header = BytesHeader;

    #[inline(always)]
    fn serialize_body(self, output: &mut [u8]) -> Result<(BytesHeader, usize), usize> {
        <&[u8] as Serialize<Bytes>>::serialize_body(self.as_ref().as_bytes(), output)
    }

    #[inline(always)]
    fn body_size(self) -> usize {
        <&[u8] as Serialize<Bytes>>::body_size(self.as_ref().as_bytes())
    }

    #[inline(always)]
    fn serialize_header(header: BytesHeader, output: &mut [u8], offset: usize) -> bool {
        <&[u8] as Serialize<Bytes>>::serialize_header(header, output, offset)
    }
}
