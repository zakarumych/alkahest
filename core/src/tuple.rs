use crate::{
    deserialize::{Deserialize, DeserializeError, Deserializer},
    element::{Element, heap_size, stack_size},
    formula::{ExactSize, Formula, SizeBound, SizeType},
    serialize::{Serialize, Serializer, Sizes, size_hint},
};

impl Formula for () {
    type StackSize<const SIZE_BYTES: u8> = ExactSize<0>;
    type HeapSize<const SIZE_BYTES: u8> = ExactSize<0>;
    const INHABITED: bool = true;
}

impl Serialize<()> for () {
    #[inline]
    fn serialize<S>(&self, _serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        Ok(())
    }

    #[inline]
    fn size_hint<const SIZE_BYTES: u8>(&self) -> Option<Sizes> {
        Some(Sizes::ZERO)
    }
}

impl<'de> Deserialize<'de, ()> for () {
    #[inline]
    fn deserialize<D>(_de: D) -> Result<(), DeserializeError>
    where
        D: Deserializer<'de>,
    {
        Ok(())
    }

    #[inline]
    fn deserialize_in_place<D>(&mut self, _de: D) -> Result<(), DeserializeError>
    where
        D: Deserializer<'de>,
    {
        Ok(())
    }
}

pub struct TupleStackSize<T: ?Sized, const SIZE_BYTES: u8>(T);
pub struct TupleHeapSize<T: ?Sized, const SIZE_BYTES: u8>(T);

macro_rules! formula_serialize {
    (,) => {};
    ($at:ident $($a:ident)* , $bt:ident $($b:ident)*) => {
        impl<$($a,)* $at, const SIZE_BYTES: u8> SizeType for TupleStackSize<($($a,)* $at,), SIZE_BYTES>
        where
            $($a: Element,)*
            $at: Element + ?Sized,
        {
            const VALUE: SizeBound = {
                #[allow(unused_mut)]
                let mut total = stack_size::<$at::Formula, SIZE_BYTES>();
                $(
                    total = total.add(stack_size::<$a::Formula, SIZE_BYTES>());
                )*
                total
            };
        }

        impl<$($a,)* $at, const SIZE_BYTES: u8> SizeType for TupleHeapSize<($($a,)* $at,), SIZE_BYTES>
        where
            $($a: Element,)*
            $at: Element + ?Sized,
        {
            const VALUE: SizeBound = {
                #[allow(unused_mut)]
                let mut total = heap_size::<$at::Formula, SIZE_BYTES>();
                $(
                    total = total.add(heap_size::<$a::Formula, SIZE_BYTES>());
                )*
                total
            };
        }

        impl<$($a,)* $at> Formula for ($($a,)* $at,)
        where
            $($a: Element,)*
            $at: Element + ?Sized,
        {
            type StackSize<const SIZE_BYTES: u8> = TupleStackSize<($($a,)* $at,), SIZE_BYTES>;
            type HeapSize<const SIZE_BYTES: u8> = TupleHeapSize<($($a,)* $at,), SIZE_BYTES>;
            const INHABITED: bool = ( $($a::INHABITED &&)* $at::INHABITED );
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
            #[inline]
            fn serialize<S>(&self, mut serializer: S) -> Result<(), S::Error>
            where
                S: Serializer,
            {
                #![allow(non_snake_case, unused_mut)]

                let ($($b,)* $bt,) = self;
                $(
                    serializer.write_direct($b)?;
                )*
                serializer.write_direct($bt)
            }

            #[inline(always)]
            fn size_hint<const SIZE_BYTES: u8>(&self) -> Option<Sizes> {
                #![allow(non_snake_case, unused_mut)]

                let ($($b,)* $bt,) = self;

                let mut sizes = size_hint::<$at, _, SIZE_BYTES>($bt)?;

                $(
                    sizes += size_hint::<$a, _, SIZE_BYTES>($b)?;
                )*

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
            #[inline]
            fn deserialize<D>(mut de: D) -> Result<($($b,)* $bt,), DeserializeError>
            where
                D: Deserializer<'de>,
            {
                #![allow(non_snake_case)]
                $(
                    let $b = de.read_direct::<$a, $b>()?;
                )*

                let $bt = de.read_direct::<$at, $bt>()?;

                let value = ($($b,)* $bt,);
                Ok(value)
            }

            #[inline]
            fn deserialize_in_place<D>(&mut self, mut de: D) -> Result<(), DeserializeError>
            where
                D: Deserializer<'de>,
            {
                #![allow(non_snake_case)]

                let ($($b,)* $bt,) = self;

                $(
                    de.read_direct_in_place::<$a, $b>($b)?;
                )*
                de.read_direct_in_place::<$at, $bt>($bt)?;

                Ok(())
            }
        }
    };
}

for_tuple_2!(formula_serialize);
