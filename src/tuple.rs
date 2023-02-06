use crate::{
    deserialize::{Deserialize, Deserializer, Error},
    formula::{combine_sizes, Formula, NonRefFormula},
    serialize::{Serialize, SerializeOwned, Serializer},
};

impl NonRefFormula for () {
    const MAX_SIZE: Option<usize> = Some(0);
}

impl SerializeOwned<()> for () {
    #[cfg_attr(feature = "inline-more", inline(always))]
    fn serialize_owned<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ser.into().finish()
    }
}

impl SerializeOwned<()> for &'_ () {
    #[cfg_attr(feature = "inline-more", inline(always))]
    fn serialize_owned<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ser.into().finish()
    }
}

impl Deserialize<'_, ()> for () {
    #[cfg_attr(feature = "inline-more", inline(always))]
    fn deserialize(_de: Deserializer) -> Result<(), Error> {
        Ok(())
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
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

        impl<$($a,)* $at, $($b,)* $bt> SerializeOwned<($($a,)* $at,)> for ($($b,)* $bt,)
        where
            $(
                $a: Formula,
                $b: SerializeOwned<$a::NonRef>,
            )*
            $at: Formula + ?Sized,
            $bt: SerializeOwned<$at::NonRef>,
        {
            #[cfg_attr(feature = "inline-more", inline(always))]
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

        impl<$($a,)* $at, $($b,)* $bt,> Serialize<($($a,)* $at,)> for ($($b,)* $bt,)
        where
            $(
                $a: Formula,
                $b: Serialize<$a::NonRef>,
            )*
            $at: Formula + ?Sized,
            $bt: Serialize<$at::NonRef>,
        {
            #[cfg_attr(feature = "inline-more", inline(always))]
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

        impl<'__de, $($a,)* $at, $($b,)* $bt> Deserialize<'__de, ($($a,)* $at,)> for ($($b,)* $bt,)
        where
            $(
                $a: Formula,
                $b: Deserialize<'__de, $a::NonRef>,
            )*
            $at: Formula + ?Sized,
            $bt: Deserialize<'__de, $at::NonRef>,
        {
            #[cfg_attr(feature = "inline-more", inline(always))]
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

            #[cfg_attr(feature = "inline-more", inline(always))]
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
