use crate::{
    deserialize::{Deserialize, DeserializeError, Deserializer},
    schema::{Schema, SizedSchema},
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
    fn deserialize(len: usize, _input: &'_ [u8]) -> Result<(), DeserializeError> {
        if len != 0 {
            return Err(DeserializeError::WrongLength);
        }
        Ok(())
    }

    fn deserialize_in_place(
        &mut self,
        len: usize,
        _input: &'_ [u8],
    ) -> Result<(), DeserializeError> {
        if len != 0 {
            return Err(DeserializeError::WrongLength);
        }
        Ok(())
    }
}

macro_rules! impl_for_tuple {
    ([$at:ident $(, $a:ident)* $(,)?] [$bt:ident $(,$b:ident)* $(,)?]) => {
        impl<$($a,)* $at> Schema for ($($a,)* $at,)
        where
            $($a: SizedSchema,)*
            $at: Schema + ?Sized,
        {
        }

        impl<$($a,)* $at> SizedSchema for ($($a,)* $at,)
        where
            $($a: SizedSchema,)*
            $at: SizedSchema + ?Sized,
        {
            const SIZE: usize = 0 $( + <$a as SizedSchema>::SIZE)*;
        }

        impl<$($a,)* $at, $($b,)* $bt> Serialize<($($a,)* $at,)> for ($($b,)* $bt,)
        where
            $($a: SizedSchema, $b: Serialize<$a>,)*
            $at: Schema + ?Sized, $bt: Serialize<$at>,
        {
            #[inline]
            fn serialize(self, offset: usize, output: &mut [u8]) -> Result<(usize, usize), usize> {
                #![allow(non_snake_case)]

                let mut ser = Serializer::new(offset, output);

                let ($($b,)* $bt,) = self;

                let mut exhausted = false;
                let mut needs_more = 0;
                $(
                    if !exhausted {
                        match ser.serialize_value::<$a, $b>($b) {
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
                )*

                if !exhausted {
                    match ser.serialize_value::<$at, $bt>($bt) {
                        Ok(()) => {}
                        Err(size) => {
                            exhausted = true;
                            needs_more += size;
                        }
                    }
                } else {
                    let size = <$bt as Serialize<$at>>::size($bt);
                    needs_more += size;
                }

                if exhausted {
                    Err(ser.written() + needs_more)
                } else {
                    Ok(ser.finish())
                }
            }
        }

        impl<'a, $($a,)* $at, $($b,)* $bt> Serialize<($($a,)* $at,)> for &'a ($($b,)* $bt,)
        where
            $($a: SizedSchema, &'a $b: Serialize<$a>,)*
            $at: Schema + ?Sized, $bt: ?Sized, &'a $bt: Serialize<$at>,
        {
            #[inline]
            fn serialize(self, offset: usize, output: &mut [u8]) -> Result<(usize, usize), usize> {
                #![allow(non_snake_case)]

                let ($($b,)* $bt,) = self;
                let me = ($($b,)* $bt,);
                <($(&'a $b,)* &'a $bt,) as Serialize<($($a,)* $at,)>>::serialize(me, offset, output)
            }
        }

        impl<'__a, $($a,)* $at, $($b,)* $bt> Deserialize<'__a, ($($a,)* $at,)> for ($($b,)* $bt,)
        where
            $($a: SizedSchema, $b: Deserialize<'__a, $a>,)*
            $at: Schema + ?Sized, $bt: Deserialize<'__a, $at>,
        {
            #[inline(always)]
            fn deserialize(len: usize, input: &'__a [u8]) -> Result<($($b,)* $bt,), DeserializeError> {
                #![allow(non_snake_case)]

                let tuple_no_tail_size: usize = 0$( + <$a as SizedSchema>::SIZE)*;
                if tuple_no_tail_size > len {
                    return Err(DeserializeError::WrongLength);
                }

                let mut des = Deserializer::new(input);
                $(let $b;)*

                $(
                    $b = des.deserialize_sized::<$a, $b>()?;
                )*

                let $bt = des.deserialize(len - tuple_no_tail_size)?;

                let value = ($($b,)* $bt,);
                Ok(value)
            }

            #[inline(always)]
            fn deserialize_in_place(&mut self, len: usize, input: &'__a [u8]) -> Result<(), DeserializeError> {
                #![allow(non_snake_case)]

                let tuple_no_tail_size: usize = 0$( + <$a as SizedSchema>::SIZE)*;
                if tuple_no_tail_size > len {
                    return Err(DeserializeError::WrongLength);
                }

                let mut des = Deserializer::new(input);
                let ($($b,)* $bt,) = self;

                $(
                    des.deserialize_in_place_sized::<$a, $b>($b)?;
                )*

                des.deserialize_in_place::<$at, $bt>($bt, len - tuple_no_tail_size)?;

                Ok(())
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
