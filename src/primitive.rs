use {
    crate::schema::{Pack, Schema, SchemaUnpack},
    core::mem::align_of,
};

macro_rules! impl_primitive {
    ($head:ty $(, $tail:ty)+ $(,)?) => {
        impl_primitive!($head);
        impl_primitive!($($tail),+);
    };
    ($ty:ty) => {
        impl<'a> SchemaUnpack<'a> for $ty {
            type Unpacked = Self;
        }

        impl Schema for $ty {
            type Packed = Self;

            fn align() -> usize {
                align_of::<$ty>()
            }

            #[cfg(target_endian = "little")]
            fn unpack<'a>(packed: $ty, _bytes: &'a [u8]) -> $ty {
                packed
            }

            #[cfg(not(target_endian = "little"))]
            fn unpack<'a>(packed: $ty, _bytes: &'a [u8]) -> $ty {
                <$ty>::from_le(packed)
            }
        }

        impl<T> Pack<$ty> for T
        where
            T: core::borrow::Borrow<$ty>,
        {
            #[cfg(target_endian = "little")]
            fn pack(self, _offset: usize, _bytes: &mut [u8]) -> ($ty, usize) {
                (*self.borrow(), 0)
            }

            #[cfg(not(target_endian = "little"))]
            fn pack(self, _offset: usize, _bytes: &mut [u8]) -> ($ty, usize) {
                (<$ty>::to_le(*self.borrow()), 0)
            }
        }
    };
}

impl_primitive!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);
