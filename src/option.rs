use crate::{Access, Formula, Serialize};

impl<T> Formula for Option<T>
where
    T: Formula,
{
    type Access<'a> = Option<Access<'a, T>>;

    #[inline(always)]
    fn header() -> usize {
        1 + <T as Formula>::header()
    }

    #[inline(always)]
    fn has_body() -> bool {
        <T as Formula>::has_body()
    }

    #[inline(always)]
    fn access<'a>(input: &'a [u8]) -> Access<'a, Self> {
        if input[0] == 0 {
            None
        } else {
            Some(<T as Formula>::access(&input[1..]))
        }
    }
}

impl<T, U> Serialize<Option<T>> for Option<U>
where
    T: Formula,
    U: Serialize<T>,
{
    type Header = Option<U::Header>;

    #[inline(always)]
    fn serialize_body(self, output: &mut [u8]) -> Result<(Self::Header, usize), usize> {
        match self {
            None => Ok((None, 0)),
            Some(value) => {
                let (header, offset) =
                    <U as Serialize<T>>::serialize_body(value, &mut output[1..])?;
                output[0] = 1;
                Ok((Some(header), offset + 1))
            }
        }
    }

    #[inline(always)]
    fn body_size(self) -> usize
    where
        Self: Sized,
    {
        match self {
            None => 0,
            Some(value) => <U as Serialize<T>>::body_size(value),
        }
    }

    #[inline(always)]
    fn serialize_header(header: Option<U::Header>, output: &mut [u8], offset: usize) -> bool {
        if output.len() < <Option<T> as Formula>::header() {
            return false;
        }

        match header {
            None => {
                output[offset] = 0;
                true
            }
            Some(header) => {
                output[offset] = 1;
                <U as Serialize<T>>::serialize_header(header, &mut output[1..], offset - 1)
            }
        }
    }
}
