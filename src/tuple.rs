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
    #[inline(always)]
    fn deserialize(_de: Deserializer) -> Result<(), Error> {
        Ok(())
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, _de: Deserializer) -> Result<(), Error> {
        Ok(())
    }
}

macro_rules! impl_for_tuple {
    ([$at:ident $(,$a:ident)* $(,)?] [$bt:ident $(,$b:ident)* $(,)?]) => {
        impl<$($a,)* $at> Formula for ($($a,)* $at,)
        where
            $($a: Formula,)*
            $at: Formula + ?Sized,
        {
            const MAX_SIZE: Option<usize> = {
                let mut size = Some(0);
                $(size = combine_sizes(size, <$a as Formula>::MAX_SIZE);)*
                size = combine_sizes(size, <$at as Formula>::MAX_SIZE);
                size
            };
        }

        impl<$($a,)* $at> NonRefFormula for ($($a,)* $at,)
        where
            $($a: Formula,)*
            $at: Formula + ?Sized,
        {}

        impl<$($a,)* $at, $($b,)* $bt> Serialize<($($a,)* $at,)> for ($($b,)* $bt,)
        where
            $(
                $a: Formula,
                $b: Serialize<$a>,
            )*
            $at: Formula + ?Sized,
            $bt: Serialize<$at>,
        {
            #[inline(always)]
            fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                #![allow(non_snake_case)]
                let mut ser = ser.into();
                let ($($b,)* $bt,) = self;
                $(
                    ser.write_value::<$a, $b>($b)?;
                )*
                ser.write_value::<$at, $bt>($bt)?;
                ser.finish()
            }
        }

        impl<'__ser, $($a,)* $at, $($b,)* $bt,> Serialize<($($a,)* $at,)> for &'__ser ($($b,)* $bt,)
        where
            $(
                $a: Formula,
                &'__ser $b: Serialize<$a>,
            )*
            $at: Formula + ?Sized,
            &'__ser $bt: Serialize<$at>,
        {
            #[inline(always)]
            fn serialize<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                #![allow(non_snake_case)]
                let mut ser = ser.into();
                let ($($b,)* $bt,) = self;
                $(
                    ser.write_value::<$a, &'__ser $b>($b)?;
                )*
                ser.write_value::<$at, &'__ser $bt>($bt)?;
                ser.finish()
            }
        }

        impl<'__de, $($a,)* $at, $($b,)* $bt> Deserialize<'__de, ($($a,)* $at,)> for ($($b,)* $bt,)
        where
            $(
                $a: Formula,
                $b: Deserialize<'__de, $a>,
            )*
            $at: Formula + ?Sized,
            $bt: Deserialize<'__de, $at>,
        {
            #[inline(always)]
            fn deserialize(mut de: Deserializer<'__de>) -> Result<($($b,)* $bt,), Error> {
                #![allow(non_snake_case)]
                $(
                    let $b = de.read_value::<$a, $b>()?;
                )*
                let $bt = de.read_value::<$at, $bt>()?;
                de.finish()?;

                let value = ($($b,)* $bt,);
                Ok(value)
            }

            #[inline(always)]
            fn deserialize_in_place(&mut self, mut de: Deserializer<'__de>) -> Result<(), Error> {
                #![allow(non_snake_case)]

                let ($($b,)* $bt,) = self;

                $(
                    de.read_in_place::<$a, $b>($b)?;
                )*
                de.read_in_place::<$at, $bt>($bt)?;
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
