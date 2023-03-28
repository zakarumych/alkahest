use core::mem::size_of;

use crate::{
    buffer::Buffer,
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::{sum_size, BareFormula, Formula},
    serialize::{field_size_hint, write_field, Serialize, Sizes},
    size::FixedUsize,
};

impl Formula for () {
    const MAX_STACK_SIZE: Option<usize> = Some(0);
    const EXACT_SIZE: bool = true;
    const HEAPLESS: bool = true;
}

impl BareFormula for () {}

impl Serialize<()> for () {
    #[inline(always)]
    fn serialize<B>(self, _sizes: &mut Sizes, _buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        Ok(())
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        Some(Sizes::ZERO)
    }
}

impl Serialize<()> for &'_ () {
    #[inline(always)]
    fn serialize<B>(self, _sizes: &mut Sizes, _buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        Ok(())
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        Some(Sizes::ZERO)
    }
}

impl Deserialize<'_, ()> for () {
    #[inline(always)]
    fn deserialize(_de: Deserializer) -> Result<(), DeserializeError> {
        Ok(())
    }

    #[inline(always)]
    fn deserialize_in_place(&mut self, _de: Deserializer) -> Result<(), DeserializeError> {
        Ok(())
    }
}

macro_rules! for_tuple_2 {
    ($macro:ident) => {
        for_tuple_2!($macro for
            AA AB AC AD AE AF AG AH AI AJ AK AL AM AN AO AP,
            BA BB BC BD BE BF BG BH BI BJ BK BL BM BN BO BP
        );
    };
    ($macro:ident for ,) => {
        $macro!(,);
    };
    ($macro:ident for $a_head:ident $($a_tail:ident)*, $b_head:ident $($b_tail:ident)*) => {
        for_tuple_2!($macro for $($a_tail)*, $($b_tail)*);

        $macro!($a_head $($a_tail)*, $b_head $($b_tail)*);
    };
}

macro_rules! formula_serialize {
    (,) => {};
    ($at:ident $($a:ident)* , $bt:ident $($b:ident)*) => {
        impl<$($a,)* $at> Formula for ($($a,)* $at,)
        where
            $($a: Formula,)*
            $at: Formula + ?Sized,
        {
            const MAX_STACK_SIZE: Option<usize> = {
                let mut size = Some(0);
                $(size = sum_size(size, <$a as Formula>::MAX_STACK_SIZE);)*
                size = sum_size(size, <$at as Formula>::MAX_STACK_SIZE);
                size
            };

            const EXACT_SIZE: bool = $(<$a as Formula>::EXACT_SIZE &&)* <$at as Formula>::EXACT_SIZE;
            const HEAPLESS: bool = $(<$a as Formula>::HEAPLESS &&)* <$at as Formula>::HEAPLESS;
        }

        impl<$($a,)* $at> BareFormula for ($($a,)* $at,)
        where
            $($a: Formula,)*
            $at: Formula + ?Sized,
        {
        }


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
            fn serialize<B>(self, sizes: &mut Sizes, mut buffer: B) -> Result<(), B::Error>
            where
                B: Buffer,
            {
                #![allow(non_snake_case, unused_mut)]

                let ($($b,)* $bt,) = self;
                $(
                    write_field::<$a, $b, _>($b, sizes, buffer.reborrow(), false)?;
                )*
                write_field::<$at, $bt, _>($bt, sizes, buffer, true)
            }

            #[inline(always)]
            fn size_hint(&self) -> Option<Sizes> {
                #![allow(non_snake_case, unused_mut)]
                let mut sizes = Sizes::ZERO;
                let ($($b,)* $bt,) = self;
                $(
                    if $a::MAX_STACK_SIZE.is_none() {
                        sizes.add_stack(size_of::<FixedUsize>());
                    }
                    sizes += field_size_hint::<$a>($b, false)?;
                )*
                sizes += field_size_hint::<$at>($bt, true)?;
                Some(sizes)
            }
        }

        impl<'ser, $($a,)* $at, $($b,)* $bt,> Serialize<($($a,)* $at,)> for &'ser ($($b,)* $bt,)
        where
            $(
                $a: Formula,
                &'ser $b: Serialize<$a>,
            )*
            $at: Formula + ?Sized,
            &'ser $bt: Serialize<$at>,
            $bt: ?Sized,
        {
            #[inline(always)]
            fn serialize<B>(self, sizes: &mut Sizes, mut buffer: B) -> Result<(), B::Error>
            where
                B: Buffer,
            {
                #![allow(non_snake_case, unused_mut)]
                let ($($b,)* $bt,) = self;
                $(
                    write_field::<$a, &$b, _>($b, sizes, buffer.reborrow(), false)?;
                )*
                write_field::<$at, &$bt, _>($bt, sizes, buffer, true)
            }

            #[inline(always)]
            fn size_hint(&self) -> Option<Sizes> {
                #![allow(non_snake_case, unused_mut)]
                let mut sizes = Sizes::ZERO;
                let ($($b,)* $bt,) = self;
                $(
                    if $a::MAX_STACK_SIZE.is_none() {
                        sizes.add_stack(size_of::<FixedUsize>());
                    }
                    sizes += field_size_hint::<$a>(&$b, false)?;
                )*

                sizes += field_size_hint::<$at>(&$bt, true)?;

                Some(sizes)
            }
        }

        impl<'de, $($a,)* $at, $($b,)* $bt> Deserialize<'de, ($($a,)* $at,)> for ($($b,)* $bt,)
        where
            $(
                $a: Formula,
                $b: Deserialize<'de, $a>,
            )*
            $at: Formula + ?Sized,
            $bt: Deserialize<'de, $at>,
        {
            #[inline(always)]
            fn deserialize(mut de: Deserializer<'de>) -> Result<($($b,)* $bt,), DeserializeError> {
                #![allow(non_snake_case)]
                $(
                    let $b = de.read_value::<$a, $b>(false)?;
                )*
                let $bt = de.read_value::<$at, $bt>(true)?;
                de.finish()?;

                let value = ($($b,)* $bt,);
                Ok(value)
            }

            #[inline(always)]
            fn deserialize_in_place(&mut self, mut de: Deserializer<'de>) -> Result<(), DeserializeError> {
                #![allow(non_snake_case)]

                let ($($b,)* $bt,) = self;

                $(
                    de.read_in_place::<$a, $b>($b, false)?;
                )*
                de.read_in_place::<$at, $bt>($bt, true)?;
                de.finish()?;

                Ok(())
            }
        }
    };
}

for_tuple_2!(formula_serialize);
