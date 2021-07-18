use {
    crate::{Pack, Schema, SchemaUnpack},
    core::mem::align_of,
};

impl<'a, T> SchemaUnpack<'a> for T
where
    T: bytemuck::Pod,
{
    type Unpacked = Self;
}

impl<T> Schema for T
where
    T: bytemuck::Pod,
{
    type Packed = Self;

    fn align() -> usize {
        align_of::<Self>()
    }

    fn unpack<'a>(packed: Self, _bytes: &'a [u8]) -> Self {
        packed
    }
}

impl<T, U> Pack<T> for U
where
    T: bytemuck::Pod,
    U: core::borrow::Borrow<T>,
{
    fn pack(self, _offset: usize, _bytes: &mut [u8]) -> (T, usize) {
        (*self.borrow(), 0)
    }
}
