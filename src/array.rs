use crate::schema::{Pack, Packed, Schema, SchemaUnpack, Unpacked};

impl<'a, T, const N: usize> SchemaUnpack<'a> for [T; N]
where
    T: Schema,
{
    type Unpacked = [<T as SchemaUnpack<'a>>::Unpacked; N];
}

impl<T, const N: usize> Schema for [T; N]
where
    T: Schema,
{
    type Packed = [T::Packed; N];

    fn align() -> usize {
        T::align()
    }

    #[inline]
    fn unpack<'a>(packed: [T::Packed; N], input: &'a [u8]) -> Unpacked<'a, Self> {
        packed.map(|packed| T::unpack(packed, input))
    }
}

impl<T, U, const N: usize> Pack<[T; N]> for [U; N]
where
    T: Schema,
    U: Pack<T>,
{
    #[inline]
    fn pack(self, offset: usize, output: &mut [u8]) -> (Packed<[T; N]>, usize) {
        let mut used = 0;

        let packed = self.map(|pack| {
            let (packed, size) = pack.pack(offset + used, &mut output[used..]);
            used += size;
            packed
        });
        (packed, used)
    }
}

impl<T, U, const N: usize> Pack<[T; N]> for &'_ [U; N]
where
    T: Schema,
    for<'a> &'a U: Pack<T>,
{
    #[inline]
    fn pack(self, offset: usize, output: &mut [u8]) -> (Packed<[T; N]>, usize) {
        let mut storage: Packed<[T; N]> = bytemuck::Zeroable::zeroed();

        let mut used = 0;

        for i in 0..N {
            let (packed, size) = (&self[i]).pack(offset + used, &mut output[used..]);
            storage[i] = packed;
            used += size;
        }

        (storage, used)
    }
}
