use crate::{
    buffer::Buffer,
    formula::{BareFormulaType, FormulaType},
    iter::owned_iter_fast_sizes,
    serialize::{write_slice, Serialize, Sizes},
    size::SIZE_STACK,
    SerializeRef,
};

impl<F> FormulaType for [F]
where
    F: FormulaType,
{
    const MAX_STACK_SIZE: Option<usize> = match F::MAX_STACK_SIZE {
        Some(0) => Some(SIZE_STACK),
        _ => None,
    };
    const EXACT_SIZE: bool = false;
    const HEAPLESS: bool = F::HEAPLESS;

    #[cfg(feature = "evolution")]
    fn descriptor(builder: crate::evolution::DescriptorBuilder) {
        builder.sequence::<F>(None);
    }
}

impl<F> BareFormulaType for [F] where F: FormulaType {}

impl<F, T> SerializeRef<[F]> for [T]
where
    F: FormulaType,
    for<'a> &'a T: Serialize<F>,
{
    #[inline(always)]
    fn serialize<B>(&self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        write_slice::<F, &T, _>(self.iter(), sizes, buffer)
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        owned_iter_fast_sizes::<F, _, _>(self.iter())
    }
}
