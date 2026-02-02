use crate::formula::{ExactSize, Formula};

pub type Void = core::convert::Infallible;

impl Formula for Void {
    type StackSize<const SIZE_BYTES: u8> = ExactSize<0>;
    type HeapSize<const SIZE_BYTES: u8> = ExactSize<0>;

    const INHABITED: bool = false;
}
