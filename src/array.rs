use crate::schema::{Access, Schema, Serialize};

impl<T, const N: usize> Schema for [T; N]
where
    T: Schema,
{
    type Access<'a> = [<T as Schema>::Access<'a>; N];

    #[inline(always)]
    fn header() -> usize {
        N * T::header()
    }

    #[inline(always)]
    fn has_body() -> bool {
        T::has_body()
    }

    #[inline(always)]
    fn access<'a>(mut input: &'a [u8]) -> Access<'a, Self> {
        [(); N].map(|()| {
            let data = input;
            input = &input[T::header()..];
            <T as Schema>::access(data)
        })
    }
}

impl<T, U, const N: usize> Serialize<[T; N]> for [U; N]
where
    T: Schema,
    U: Serialize<T>,
{
    type Header = [(<U as Serialize<T>>::Header, usize); N];

    #[inline]
    fn serialize_header(header: Self::Header, output: &mut [u8], offset: usize) -> bool {
        let header_size = <T as Schema>::header() * N;

        if output.len() < header_size {
            return false;
        }

        let mut total_offset = offset;
        let mut output = output;

        for (header, element_offset) in header {
            let (head, tail) = output.split_at_mut(<T as Schema>::header());
            output = tail;

            <U as Serialize<T>>::serialize_header(header, head, total_offset + element_offset);
            total_offset -= <T as Schema>::header();
        }

        let _ = (output, total_offset);
        true
    }

    #[inline]
    fn serialize_body(self, output: &mut [u8]) -> Result<(Self::Header, usize), usize> {
        let mut written = 0;
        let mut exhausted = false;

        let headers = self.map(|elem| {
            let offset = written;
            if !exhausted {
                match <U as Serialize<T>>::serialize_body(elem, &mut output[offset..]) {
                    Ok((header, size)) => {
                        written += size;
                        Some((header, offset))
                    }
                    Err(size) => {
                        exhausted = true;
                        written += size;
                        None
                    }
                }
            } else {
                let size = <U as Serialize<T>>::body_size(elem);
                written += size;
                None
            }
        });

        if exhausted {
            Err(written)
        } else {
            let headers = headers.map(Option::unwrap);
            Ok((headers, written))
        }
    }
}

impl<'a, T, U, const N: usize> Serialize<[T; N]> for &'a [U; N]
where
    T: Schema,
    &'a U: Serialize<T>,
{
    type Header = [(<&'a U as Serialize<T>>::Header, usize); N];

    #[inline]
    fn serialize_header(header: Self::Header, output: &mut [u8], offset: usize) -> bool {
        <[&'a U; N] as Serialize<[T; N]>>::serialize_header(header, output, offset)
    }

    #[inline]
    fn serialize_body(self, output: &mut [u8]) -> Result<(Self::Header, usize), usize> {
        let mut written = 0;
        let mut exhausted = false;

        let headers = self.map_ref(|elem| {
            let offset = written;
            if !exhausted {
                match <&'a U as Serialize<T>>::serialize_body(elem, &mut output[offset..]) {
                    Ok((header, size)) => {
                        written += size;
                        Some((header, offset))
                    }
                    Err(size) => {
                        exhausted = true;
                        written += size;
                        None
                    }
                }
            } else {
                let size = <&'a U as Serialize<T>>::body_size(elem);
                written += size;
                None
            }
        });

        if exhausted {
            Err(written)
        } else {
            let headers = headers.map(Option::unwrap);
            Ok((headers, written))
        }
    }
}

trait MapArrayRef<const N: usize> {
    type Item: Sized;

    fn map_ref<'a, F, U>(&'a self, f: F) -> [U; N]
    where
        F: FnMut(&'a Self::Item) -> U;

    fn map_mut<'a, F, U>(&'a mut self, f: F) -> [U; N]
    where
        F: FnMut(&'a mut Self::Item) -> U;
}

impl<T, const N: usize> MapArrayRef<N> for [T; N] {
    type Item = T;

    #[inline]
    fn map_ref<'a, F, U>(&'a self, mut f: F) -> [U; N]
    where
        F: FnMut(&'a Self::Item) -> U,
    {
        let mut iter = self.iter();
        [(); N].map(|()| f(iter.next().unwrap()))
    }

    #[inline]
    fn map_mut<'a, F, U>(&'a mut self, mut f: F) -> [U; N]
    where
        F: FnMut(&'a mut Self::Item) -> U,
    {
        let mut iter = self.iter_mut();
        [(); N].map(|()| f(iter.next().unwrap()))
    }
}
