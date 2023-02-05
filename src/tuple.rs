use crate::{
    deserialize::{Deserialize, Deserializer, Error},
    formula::{combine_sizes, Formula, NonRefFormula},
    serialize::{Serialize, Serializer},
};

impl Formula for () {
    const MAX_SIZE: Option<usize> = Some(0);
}
impl NonRefFormula for () {}

impl Serialize<()> for () {
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ser.into().finish()
    }
}

impl Serialize<()> for &'_ () {
    #[inline(always)]
    fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ser.into().finish()
    }
}

impl Deserialize<'_, ()> for () {
    fn deserialize(_de: Deserializer) -> Result<(), Error> {
        Ok(())
    }

    fn deserialize_in_place(&mut self, _de: Deserializer) -> Result<(), Error> {
        Ok(())
    }
}

macro_rules! impl_for_tuple {
    ([$($a:ident),* $(,)?] [$($b:ident),* $(,)?]) => {
        impl<$($a,)*> Formula for ($($a,)*)
        where
            $($a: Formula,)*
        {
            const MAX_SIZE: Option<usize> = {
                let mut size = Some(0);
                $(size = combine_sizes(size, <$a as Formula>::MAX_SIZE);)*
                size
            };
        }

        impl<$($a,)*> NonRefFormula for ($($a,)*)
        where
            $($a: Formula,)*
        {}

        impl<$($a,)* $($b,)*> Serialize<($($a,)*)> for ($($b,)*)
        where
            $(
                $a: Formula,
                $b: Serialize<$a>,
            )*
        {
            #[inline]
            fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                #![allow(non_snake_case)]
                let mut ser = ser.into();
                let ($($b,)*) = self;
                $(
                    ser.write_value::<$a, $b>($b)?;
                )*
                ser.finish()
            }
        }

        impl<'__ser, $($a,)* $($b,)*> Serialize<($($a,)*)> for &'__ser ($($b,)*)
        where
            $(
                $a: Formula,
                &'__ser $b: Serialize<$a>,
            )*
        {
            #[inline]
            fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                #![allow(non_snake_case)]
                let mut ser = ser.into();
                let ($($b,)*) = self;
                $(
                    ser.write_value::<$a, &'__ser $b>(&$b)?;
                )*
                ser.finish()
            }
        }

        impl<'__de, $($a,)* $($b,)*> Deserialize<'__de, ($($a,)*)> for ($($b,)*)
        where
            $(
                $a: Formula,
                $b: Deserialize<'__de, $a>,
            )*
        {
            #[inline(always)]
            fn deserialize(mut de: Deserializer<'__de>) -> Result<($($b,)*), Error> {
                #![allow(non_snake_case)]
                $(
                    let $b = de.read_value::<$a, $b>()?;
                )*
                de.finish()?;

                let value = ($($b,)*);
                Ok(value)
            }

            #[inline(always)]
            fn deserialize_in_place(&mut self, mut de: Deserializer<'__de>) -> Result<(), Error> {
                #![allow(non_snake_case)]

                let ($($b,)*) = self;

                $(
                    de.read_in_place::<$a, $b>($b)?;
                )*
                de.finish()?;

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
