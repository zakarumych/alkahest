use crate::schema::{Access, Schema, Serialize};

impl Schema for () {
    type Access<'a> = ();

    #[inline(always)]
    fn header() -> usize {
        0
    }

    #[inline(always)]
    fn access<'a>(_input: &'a [u8]) -> Access<'a, Self> {}
}

impl Serialize<()> for () {
    type Header = ();

    #[inline(always)]
    fn serialize_body(self, _output: &mut [u8]) -> Result<((), usize), usize> {
        Ok(((), 0))
    }

    #[inline(always)]
    fn serialize_header(_header: (), _output: &mut [u8], _offset: usize) -> bool {
        true
    }
}

impl Serialize<()> for &'_ () {
    type Header = ();

    #[inline(always)]
    fn serialize_body(self, _output: &mut [u8]) -> Result<((), usize), usize> {
        Ok(((), 0))
    }

    #[inline(always)]
    fn serialize_header(_header: (), _output: &mut [u8], _offset: usize) -> bool {
        true
    }
}

macro_rules! impl_for_tuple {
    ([$($a:ident),+ $(,)?] [$($b:ident),+ $(,)?]) => {
        impl<$($a),+> Schema for ($($a,)+)
        where
            $($a: Schema,)+
        {
            type Access<'__a> = ($(Access<'__a, $a>,)+);

            #[inline(always)]
            fn header() -> usize {
                0 $(+ <$a as Schema>::header())+
            }

            #[inline(always)]
            fn has_body() -> bool {
                false $(|| <$a as Schema>::has_body())+
            }

            #[inline(always)]
            fn access<'__a>(input: &'__a [u8]) -> ($(Access<'__a, $a>,)+) {
                #![allow(unused_assignments)]

                let mut offset = 0;
                ($({
                    let cur = offset;
                    offset += <$a as Schema>::header();
                    <$a as Schema>::access(&input[cur..])
                },)+)
            }
        }

        impl<$($a),+ , $($b),+> Serialize<($($a,)+)> for ($($b,)+)
        where
            $($a: Schema, $b: Serialize<$a>,)+
        {
            type Header = ($(
                (<$b as Serialize<$a>>::Header, usize),
            )+);

            #[inline]
            fn serialize_header(header: Self::Header, output: &mut [u8], offset: usize) -> bool {
                #![allow(non_snake_case)]

                let header_size = 0 $(+ <$a as Schema>::header())+;

                if output.len() < header_size {
                    return false;
                }

                let ($($b,)+) = header;

                let mut total_offset = offset;
                let mut output = output;
                $(
                    let (header, element_offset) = $b;

                    let (head, tail) = output.split_at_mut(<$a as Schema>::header());
                    output = tail;

                    <$b as Serialize<$a>>::serialize_header(header, head, total_offset + element_offset);
                    total_offset -= <$a as Schema>::header();
                )+

                let _ = (output, total_offset);
                true
            }

            #[inline]
            fn serialize_body(self, output: &mut [u8]) -> Result<(Self::Header, usize), usize> {
                #![allow(non_snake_case)]

                let ($($b,)+) = self;
                let ($(mut $a,)+) = ($({let _ = $b; (None, 0)},)+);

                let mut written = 0;
                let mut exhausted = false;
                $(
                    let offset = written;
                    if !exhausted {
                        match <$b as Serialize<$a>>::serialize_body($b, &mut output[offset..]) {
                            Ok((header, size)) => {
                                $a = (Some(header), offset);
                                written += size;
                            }
                            Err(size) => {
                                exhausted = true;
                                written += size;
                            }
                        }
                    } else {
                        let size = <$b as Serialize<$a>>::body_size($b);
                        written += size;
                    }
                )+

                if exhausted {
                    Err(written)
                } else {
                    let header = ($(($a.0.unwrap(), $a.1),)+);
                    Ok((header, written))
                }
            }
        }

        impl<'a, $($a),+ , $($b),+> Serialize<($($a,)+)> for &'a ($($b,)+)
        where
            $($a: Schema, &'a $b: Serialize<$a>,)+
        {
            type Header = ($(
                (<&'a $b as Serialize<$a>>::Header, usize),
            )+);

            #[inline(always)]
            fn serialize_header(header: Self::Header, output: &mut [u8], offset: usize) -> bool {
                <($(&'a $b,)+) as Serialize<($($a,)+)>>::serialize_header(header, output, offset)
            }

            #[inline]
            fn serialize_body(self, output: &mut [u8]) -> Result<(Self::Header, usize), usize> {
                #![allow(non_snake_case)]
                let ($($b,)+) = self;
                let me = ($($b,)+);
                <($(&'a $b,)+) as Serialize<($($a,)+)>>::serialize_body(me, output)
            }
        }
    };
}

impl_for_tuple!([A][B]);
impl_for_tuple!([A, B][C, D]);
impl_for_tuple!([A, B, C][D, E, F]);
impl_for_tuple!([A, B, C, D][E, F, G, H]);
impl_for_tuple!([A, B, C, D, E][F, G, H, I, J]);
impl_for_tuple!([A, B, C, D, E, F][G, H, I, J, K, L]);
impl_for_tuple!([A, B, C, D, E, F, G][H, I, J, K, L, M, N]);
impl_for_tuple!([A, B, C, D, E, F, G, H][I, J, K, L, M, N, O, P]);
