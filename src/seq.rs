use core::{
    convert::Infallible,
    fmt::{self, Debug},
    marker::PhantomData,
    mem::size_of,
};

use crate::{Access, FixedUsize, Schema, Serialize};

/// `Schema` for runtime sized sequence of `T`.
pub enum Seq<T> {
    Uninhabited {
        void: Infallible,
        marker: PhantomData<[T]>,
    },
}

/// Access sequence.
pub struct SeqAccess<'a, T: Schema> {
    len: usize,
    input: &'a [u8],
    marker: PhantomData<[Access<'a, T>]>,
}

impl<T> Copy for SeqAccess<'_, T> where T: Schema {}
impl<T> Clone for SeqAccess<'_, T>
where
    T: Schema,
{
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> IntoIterator for SeqAccess<'a, T>
where
    T: Schema,
{
    type IntoIter = SeqIter<'a, T>;
    type Item = Access<'a, T>;

    #[inline(always)]
    fn into_iter(self) -> SeqIter<'a, T> {
        self.iter()
    }
}

impl<'a, T> SeqAccess<'a, T>
where
    T: Schema,
{
    #[inline(always)]
    pub fn iter(&self) -> SeqIter<'a, T> {
        SeqIter {
            len: self.len,
            input: self.input,
            marker: PhantomData,
        }
    }
}

impl<'a, T> Debug for SeqAccess<'a, T>
where
    T: Schema,
    Access<'a, T>: Debug,
{
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T> Schema for Seq<T>
where
    T: Schema,
{
    type Access<'a> = SeqAccess<'a, T>;

    #[inline(always)]
    fn header() -> usize {
        size_of::<[FixedUsize; 2]>()
    }

    #[inline(always)]
    fn has_body() -> bool {
        true
    }

    #[inline(always)]
    fn access<'a>(input: &'a [u8]) -> SeqAccess<'a, T> {
        let len = &input[..size_of::<FixedUsize>()];
        let len = FixedUsize::from_bytes(len.try_into().unwrap()).into();

        let offset = &input[size_of::<FixedUsize>()..][..size_of::<FixedUsize>()];
        let offset = FixedUsize::from_bytes(offset.try_into().unwrap()).into();

        SeqAccess {
            len,
            input: &input[offset..],
            marker: PhantomData,
        }
    }
}

#[repr(transparent)]
pub struct SeqHeader {
    len: FixedUsize,
}

impl<I, T, U> Serialize<Seq<T>> for I
where
    T: Schema,
    I: IntoIterator<Item = U>,
    I::IntoIter: ExactSizeIterator<Item = U>,
    U: Serialize<T>,
{
    type Header = SeqHeader;

    #[inline]
    fn serialize_body(self, output: &mut [u8]) -> Result<(SeqHeader, usize), usize> {
        let iter = self.into_iter();
        let len = iter.len();
        let header = SeqHeader {
            len: FixedUsize::truncated(len),
        };

        let headers_size = T::header() * len;
        let mut exhausted = output.len() < headers_size;

        let mut headers_offset = 0;
        let mut bodies_offset = headers_size;
        for item in iter.take(len) {
            if !exhausted {
                match <U as Serialize<T>>::serialize_body(item, &mut output[bodies_offset..]) {
                    Ok((header, size)) => {
                        <U as Serialize<T>>::serialize_header(
                            header,
                            &mut output[headers_offset..],
                            bodies_offset - headers_offset,
                        );
                        headers_offset += T::header();
                        bodies_offset += size;
                    }
                    Err(size) => {
                        exhausted = true;
                        bodies_offset += size;
                    }
                }
            } else {
                let size = <U as Serialize<T>>::body_size(item);
                bodies_offset += size;
            }
        }

        if exhausted {
            Err(bodies_offset)
        } else {
            Ok((header, bodies_offset))
        }
    }

    #[inline]
    fn serialize_header(header: Self::Header, output: &mut [u8], offset: usize) -> bool {
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

pub struct SeqIter<'a, T: Schema> {
    len: usize,
    input: &'a [u8],
    marker: PhantomData<[Access<'a, T>]>,
}

impl<'a, T> Iterator for SeqIter<'a, T>
where
    T: Schema,
{
    type Item = Access<'a, T>;

    #[inline(always)]
    fn next(&mut self) -> Option<Access<'a, T>> {
        if self.len == 0 {
            None
        } else {
            let item = T::access(&self.input);
            self.len -= 1;
            self.input = &self.input[T::header()..];
            Some(item)
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> ExactSizeIterator for SeqIter<'a, T>
where
    T: Schema,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.len
    }
}
