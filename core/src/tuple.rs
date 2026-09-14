use crate::{
    deserialize::{Deserialize, DeserializeError, Deserializer},
    element::{Element, heap_size, stack_size},
    formula::{ExactSize, Formula, SizeBound, SizeType},
    serialize::{Serialize, Serializer, Sizes, size_hint},
};

impl Formula for () {
    type StackSize<const SIZE_BYTES: usize> = ExactSize<0>;
    type HeapSize<const SIZE_BYTES: usize> = ExactSize<0>;
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
    fn size_hint<const SIZE_BYTES: usize>(&self) -> Option<Sizes> {
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

pub struct TupleStackSize<T: ?Sized, const SIZE_BYTES: usize>(T);
pub struct TupleHeapSize<T: ?Sized, const SIZE_BYTES: usize>(T);

macro_rules! formula_serialize_deserialize {
    (,) => {};
    ($at:ident $($a:ident)* , $bt:ident $($b:ident)*) => {
        impl<$($a,)* $at, const SIZE_BYTES: usize> SizeType for TupleStackSize<($($a,)* $at,), SIZE_BYTES>
        where
            $($a: Element,)*
            $at: Element + ?Sized,
        {
            const VALUE: SizeBound = {
                #[allow(unused_mut)]
                let mut total = stack_size::<$at::Formula, SIZE_BYTES>();
                $(
                    let next = stack_size::<$a::Formula, SIZE_BYTES>();
                    if total.is_unbounded() && !next.is_zero() {
                        panic!("Tuple contains stack-unbounded element that is not the last one");
                    }
                    total = total.add(next);
                )*
                total
            };
        }

        impl<$($a,)* $at, const SIZE_BYTES: usize> SizeType for TupleHeapSize<($($a,)* $at,), SIZE_BYTES>
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
            type StackSize<const SIZE_BYTES: usize> = TupleStackSize<($($a,)* $at,), SIZE_BYTES>;
            type HeapSize<const SIZE_BYTES: usize> = TupleHeapSize<($($a,)* $at,), SIZE_BYTES>;
            const INHABITED: bool = ( $($a::INHABITED &&)* $at::INHABITED );
        }

        impl<$($a,)* $at, $($b,)* $bt> Serialize<($($a,)* $at,)> for ($($b,)* $bt,)
        where
            $(
                $a: Element,
                $b: Serialize<$a::Formula>,
            )*
            $at: Element + ?Sized,
            $bt: Serialize<$at::Formula>,
        {
            #[inline]
            fn serialize<S>(&self, mut serializer: S) -> Result<(), S::Error>
            where
                S: Serializer,
            {
                #![allow(non_snake_case, unused_mut)]

                let ($($b,)* $bt,) = self;
                $(
                    simple_try!($a::serialize($b, &mut serializer));
                )*
                $at::serialize($bt, &mut serializer)
            }

            #[inline]
            fn size_hint<const SIZE_BYTES: usize>(&self) -> Option<Sizes> {
                #![allow(non_snake_case, unused_mut)]

                let ($($b,)* $bt,) = self;

                let mut sizes = simple_some!(size_hint::<$at, _, SIZE_BYTES>($bt));

                $(
                    sizes += simple_some!(size_hint::<$a, _, SIZE_BYTES>($b));
                )*

                Some(sizes)
            }
        }

        impl<'de, $($a,)* $at, $($b,)* $bt> Deserialize<'de, ($($a,)* $at,)> for ($($b,)* $bt,)
        where
            $(
                $a: Element,
                $b: Deserialize<'de, $a::Formula>,
            )*
            $at: Element + ?Sized,
            $bt: Deserialize<'de, $at::Formula>,
        {
            #[inline]
            fn deserialize<D>(mut de: D) -> Result<($($b,)* $bt,), DeserializeError>
            where
                D: Deserializer<'de>,
            {
                #![allow(non_snake_case)]
                $(
                    let $b = simple_try!($a::deserialize::<$b, _>(&mut de));
                )*

                let $bt = simple_try!($at::deserialize::<$bt, _>(&mut de));

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
                    simple_try!($a::deserialize_in_place($b, &mut de));
                )*
                $at::deserialize_in_place($bt, &mut de)
            }
        }
    };
}

for_tuple_2!(formula_serialize_deserialize);
