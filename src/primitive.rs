use core::{borrow::Borrow, mem::size_of};

use crate::{
    deserialize::{Deserialize, Deserializer, Error},
    formula::NonRefFormula,
    serialize::{Serialize, SerializeOwned, Serializer},
};

macro_rules! impl_primitive {
    () => {};

    ($head:ty $(, $tail:ty)* $(,)?) => {
        impl_primitive!(impl $head);
        impl_primitive!($($tail,)*);
    };

    (impl $ty:ty) => {
        impl NonRefFormula for $ty {
            const MAX_SIZE: Option<usize> = Some(size_of::<$ty>());
        }

        impl SerializeOwned<$ty> for $ty {
            #[cfg_attr(feature = "inline-more", inline(always))]
            fn serialize_owned<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let mut ser = ser.into();
                ser.write_bytes(&self.borrow().to_le_bytes())?;
                ser.finish()
            }
        }

        impl Serialize<$ty> for $ty {
            #[cfg_attr(feature = "inline-more", inline(always))]
            fn serialize<S>(&self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let mut ser = ser.into();
                ser.write_bytes(&self.borrow().to_le_bytes())?;
                ser.finish()
            }
        }

        impl<T> Deserialize<'_, $ty> for T
        where
            T: From<$ty>,
        {
            #[cfg_attr(feature = "inline-more", inline(always))]
            fn deserialize(mut de: Deserializer) -> Result<Self, Error> {
                let mut bytes = [0; size_of::<$ty>()];
                bytes.copy_from_slice(de.read_bytes(size_of::<$ty>())?);
                de.finish()?;
                let value = <$ty>::from_le_bytes(bytes);
                Ok(From::from(value))
            }

            #[cfg_attr(feature = "inline-more", inline(always))]
            fn deserialize_in_place(&mut self, de: Deserializer) -> Result<(), Error> {
                let value = <T as Deserialize<'_, $ty>>::deserialize(de)?;
                *self = value;
                Ok(())
            }
        }
    };
}

impl_primitive! {
    u8,
    u16,
    u32,
    u64,
    u128,
    i8,
    i16,
    i32,
    i64,
    i128,
    f32,
    f64,
}

impl NonRefFormula for bool {
    const MAX_SIZE: Option<usize> = Some(1);
}

impl SerializeOwned<bool> for bool {
    #[cfg_attr(feature = "inline-more", inline(always))]
    fn serialize_owned<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        <u8 as SerializeOwned<u8>>::serialize_owned(self as u8, ser)
    }
}

impl SerializeOwned<bool> for &bool {
    #[cfg_attr(feature = "inline-more", inline(always))]
    fn serialize_owned<S>(self, ser: impl Into<S>) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        <u8 as SerializeOwned<u8>>::serialize_owned(*self as u8, ser)
    }
}

impl<T> Deserialize<'_, bool> for T
where
    T: From<bool>,
{
    #[cfg_attr(feature = "inline-more", inline(always))]
    fn deserialize(de: Deserializer) -> Result<Self, Error> {
        let value = <u8 as Deserialize<u8>>::deserialize(de)?;
        Ok(From::from(value != 0))
    }

    #[cfg_attr(feature = "inline-more", inline(always))]
    fn deserialize_in_place(&mut self, de: Deserializer) -> Result<(), Error> {
        let value = <u8 as Deserialize<u8>>::deserialize(de)?;
        *self = From::from(value != 0);
        Ok(())
    }
}
