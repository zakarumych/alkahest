use crate::{
    deserialize::{Deserialize, DeserializeError, Deserializer},
    element::{Element, heap_size_versioned, stack_size_versioned},
    formula::{ExactSize, Formula, SizeBound, SizeType},
    serialize::{Serialize, Serializer, Sizes, size_hint_versioned},
};

impl<const VERSION: u32> Formula<VERSION> for () {
    const SCHEMA: crate::version::Schema =
        crate::version::Schema::new::<Self, VERSION>(crate::version::SchemaKind::Record(&[]));
    type StackSize<const SIZE_BYTES: usize> = ExactSize<0>;
    type HeapSize<const SIZE_BYTES: usize> = ExactSize<0>;
    const INHABITED: bool = true;
}

impl<const VERSION: u32> Serialize<(), VERSION> for () {
    #[inline(always)]
    fn serialize<S>(&self, _serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        Ok(())
    }

    #[inline(always)]
    fn size_hint<const SIZE_BYTES: usize>(&self) -> Option<Sizes> {
        Some(Sizes::ZERO)
    }
}

impl<'de, const VERSION: u32> Deserialize<'de, (), VERSION> for () {
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

/// The combined stack size of a tuple's elements.
///
/// A stack-unbounded element cannot precede a nonzero-sized element.
///
/// ```compile_fail
/// use alkahest_core::{List, SizeBound, stack_size};
/// const INVALID: SizeBound = stack_size::<(List<u8>, u8), 1>();
/// ```
pub struct TupleStackSize<T: ?Sized, const SIZE_BYTES: usize, const VERSION: u32 = 0>(T);
pub struct TupleHeapSize<T: ?Sized, const SIZE_BYTES: usize, const VERSION: u32 = 0>(T);

macro_rules! formula_serialize_deserialize {
    (,) => {};
    ($at:ident $($a:ident)* , $bt:ident $($b:ident)*) => {
        impl<$($a,)* $at, const SIZE_BYTES: usize, const VERSION: u32> SizeType for TupleStackSize<($($a,)* $at,), SIZE_BYTES, VERSION>
        where
            $($a: Element<VERSION>,)*
            $at: Element<VERSION> + ?Sized,
        {
            const VALUE: SizeBound = {
                #[allow(unused_mut)]
                let mut total = SizeBound::Exact(0);
                $(
                    let next = stack_size_versioned::<$a, SIZE_BYTES, VERSION>();
                    if total.is_unbounded() && !next.is_zero() {
                        panic!("Tuple contains stack-unbounded element that is not the last one");
                    }
                    total = total.padded().add(next);
                )*
                let next = stack_size_versioned::<$at, SIZE_BYTES, VERSION>();
                if total.is_unbounded() && !next.is_zero() {
                    panic!("Tuple contains stack-unbounded element that is not the last one");
                }
                total.padded().add(next)
            };
        }

        impl<$($a,)* $at, const SIZE_BYTES: usize, const VERSION: u32> SizeType for TupleHeapSize<($($a,)* $at,), SIZE_BYTES, VERSION>
        where
            $($a: Element<VERSION>,)*
            $at: Element<VERSION> + ?Sized,
        {
            const VALUE: SizeBound = {
                #[allow(unused_mut)]
                let mut total = heap_size_versioned::<$at, SIZE_BYTES, VERSION>();
                $(
                    total = total.add(heap_size_versioned::<$a, SIZE_BYTES, VERSION>());
                )*
                total
            };
        }

        impl<$($a,)* $at, const VERSION: u32> Formula<VERSION> for ($($a,)* $at,)
        where
            $($a: Element<VERSION>,)*
            $at: Element<VERSION> + ?Sized,
        {
            const SCHEMA: crate::version::Schema = crate::version::Schema::new::<Self, VERSION>(
                crate::version::SchemaKind::Record(&[
                    $(crate::version::Field { name: "", element: $a::SCHEMA },)*
                    crate::version::Field { name: "", element: $at::SCHEMA },
                ]),
            );
            type StackSize<const SIZE_BYTES: usize> = TupleStackSize<($($a,)* $at,), SIZE_BYTES, VERSION>;
            type HeapSize<const SIZE_BYTES: usize> = TupleHeapSize<($($a,)* $at,), SIZE_BYTES, VERSION>;
            const INHABITED: bool = ( $($a::INHABITED &&)* $at::INHABITED );
        }

        impl<$($a,)* $at, $($b,)* $bt, const VERSION: u32> Serialize<($($a,)* $at,), VERSION> for ($($b,)* $bt,)
        where
            $(
                $a: Element<VERSION>,
                $b: Serialize<$a::Formula, VERSION>,
            )*
            $at: Element<VERSION> + ?Sized,
            $bt: Serialize<$at::Formula, VERSION>,
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

                let mut sizes = simple_some!(size_hint_versioned::<$at, _, SIZE_BYTES, VERSION>($bt));

                $(
                    let mut next = simple_some!(size_hint_versioned::<$a, _, SIZE_BYTES, VERSION>($b));
                    if let SizeBound::Bounded(max_stack) = stack_size_versioned::<$a, SIZE_BYTES, VERSION>() {
                        next.stack = max_stack;
                    }
                    sizes += next;
                )*

                Some(sizes)
            }
        }

        impl<'de, $($a,)* $at, $($b,)* $bt, const VERSION: u32> Deserialize<'de, ($($a,)* $at,), VERSION> for ($($b,)* $bt,)
        where
            $(
                $a: Element<VERSION>,
                $b: Deserialize<'de, $a::Formula, VERSION>,
            )*
            $at: Element<VERSION> + ?Sized,
            $bt: Deserialize<'de, $at::Formula, VERSION>,
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
