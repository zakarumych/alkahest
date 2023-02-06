use crate::{
    deserialize::{Deserializer, Error, NonRefDeserialize},
    formula::{combine_sizes, Formula, NonRefFormula},
    serialize::{NonRefSerialize, NonRefSerializeOwned, Serializer},
};

impl NonRefFormula for () {
    const MAX_SIZE: Option<usize> = Some(0);
}

impl NonRefSerializeOwned<()> for () {
    #[inline(always)]
    fn serialize_owned<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ser.into().finish()
    }
}

impl NonRefSerializeOwned<()> for &'_ () {
    #[inline(always)]
    fn serialize_owned<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ser.into().finish()
    }
}

impl NonRefDeserialize<'_, ()> for () {
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
        impl<$($a,)* $at> NonRefFormula for ($($a,)* $at,)
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

        impl<$($a,)* $at, $($b,)* $bt> NonRefSerializeOwned<($($a,)* $at,)> for ($($b,)* $bt,)
        where
            $(
                $a: Formula,
                $b: NonRefSerializeOwned<$a::NonRef>,
            )*
            $at: Formula + ?Sized,
            $bt: NonRefSerializeOwned<$at::NonRef>,
        {
            #[inline(always)]
            fn serialize_owned<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
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

        impl<$($a,)* $at, $($b,)* $bt,> NonRefSerialize<($($a,)* $at,)> for ($($b,)* $bt,)
        where
            $(
                $a: Formula,
                $b: NonRefSerialize<$a::NonRef>,
            )*
            $at: Formula + ?Sized,
            $bt: NonRefSerialize<$at::NonRef>,
        {
            #[inline(always)]
            fn serialize<S>(&self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                #![allow(non_snake_case)]
                let mut ser = ser.into();
                let ($($b,)* $bt,) = self;
                $(
                    ser.write_value::<$a, &$b>($b)?;
                )*
                ser.write_value::<$at, &$bt>($bt)?;
                ser.finish()
            }
        }

        impl<'__de, $($a,)* $at, $($b,)* $bt> NonRefDeserialize<'__de, ($($a,)* $at,)> for ($($b,)* $bt,)
        where
            $(
                $a: Formula,
                $b: NonRefDeserialize<'__de, $a::NonRef>,
            )*
            $at: Formula + ?Sized,
            $bt: NonRefDeserialize<'__de, $at::NonRef>,
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
