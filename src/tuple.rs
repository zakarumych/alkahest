use crate::{
    deserialize::{Deserialize, DeserializeError, Deserializer},
    schema::Schema,
    serialize::{Serialize, Serializer},
};

impl Schema for () {}

impl Serialize<()> for () {
    #[inline(always)]
    fn serialize(self, _offset: usize, _output: &mut [u8]) -> Result<(usize, usize), usize> {
        Ok((0, 0))
    }

    #[inline(always)]
    fn size(self) -> usize {
        0
    }
}

impl Serialize<()> for &'_ () {
    #[inline(always)]
    fn serialize(self, _offset: usize, _output: &mut [u8]) -> Result<(usize, usize), usize> {
        Ok((0, 0))
    }

    #[inline(always)]
    fn size(self) -> usize {
        0
    }
}

impl Deserialize<'_, ()> for () {
    fn deserialize(_input: &'_ [u8]) -> Result<((), usize), DeserializeError> {
        Ok(((), 0))
    }

    fn deserialize_in_place(&mut self, _input: &'_ [u8]) -> Result<usize, DeserializeError> {
        Ok(0)
    }
}

macro_rules! inverse {
    () => {};
    ($head:block $($tail:block)*) => {
        inverse!($($tail)*);
        $head
    };
}

macro_rules! impl_for_tuple {
    ([$($a:ident),+ $(,)?] [$($b:ident),+ $(,)?]) => {
        impl<$($a),+> Schema for ($($a,)+)
        where
            $($a: Schema,)+
        {
        }

        impl<$($a),+ , $($b),+> Serialize<($($a,)+)> for ($($b,)+)
        where
            $($a: Schema, $b: Serialize<$a>,)+
        {
            #[inline]
            fn serialize(self, offset: usize, output: &mut [u8]) -> Result<(usize, usize), usize> {
                #![allow(non_snake_case)]

                let mut ser = Serializer::new(offset, output);

                let ($($b,)+) = self;

                let mut exhausted = false;
                let mut needs_more = 0;
                $(
                    if !exhausted {
                        match ser.put($b) {
                            Ok(()) => {}
                            Err(size) => {
                                exhausted = true;
                                needs_more += size;
                            }
                        }
                    } else {
                        let size = <$b as Serialize<$a>>::size($b);
                        needs_more += size;
                    }
                )+

                if exhausted {
                    Err(ser.written() + needs_more)
                } else {
                    Ok(ser.finish())
                }
            }
        }

        impl<'a, $($a),+ , $($b),+> Serialize<($($a,)+)> for &'a ($($b,)+)
        where
            $($a: Schema, &'a $b: Serialize<$a>,)+
        {
            #[inline]
            fn serialize(self, offset: usize, output: &mut [u8]) -> Result<(usize, usize), usize> {
                #![allow(non_snake_case)]

                let ($($b,)+) = self;
                let me = ($($b,)+);
                <($(&'a $b,)+) as Serialize<($($a,)+)>>::serialize(me, offset, output)
            }
        }

        impl<'__a, $($a),+ , $($b),+> Deserialize<'__a, ($($a,)+)> for ($($b,)+)
        where
            $($a: Schema, $b: Deserialize<'__a, $a>,)+
        {
            #[inline(always)]
            fn deserialize(input: &'__a [u8]) -> Result<(($($b,)+), usize), DeserializeError> {
                #![allow(non_snake_case)]

                let mut des = Deserializer::new(input);
                $(let $b;)+

                inverse!($({
                    $b = des.deserialize::<$b, $a>()?;
                })+);

                let value = ($($b,)+);
                Ok((value, des.end()))
            }

            #[inline(always)]
            fn deserialize_in_place(&mut self, input: &'__a [u8]) -> Result<usize, DeserializeError> {
                #![allow(non_snake_case)]

                let mut des = Deserializer::new(input);
                let ($($b,)+) = self;

                inverse!($({
                    des.deserialize_in_place::<$b, $a>($b)?;
                })+);

                Ok(des.end())
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
