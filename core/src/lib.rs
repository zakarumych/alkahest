#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod buffer;
mod deserialize;
mod formula;
mod primitive;
mod reference;
mod serialize;

pub mod private {
    use std::marker::PhantomData;

    /// Used for formula elements in generated formula types.
    /// Indicates that this element is stored indirectly.
    /// Indirect storage allows element formula to grow without breaking layout compatibility of this formula.
    #[allow(non_camel_case_types)]
    struct __Alkahest_Element_Indirect<T>(PhantomData<T>);

    /// Used for formula elements in generated formula types.
    /// Indicates that this element is stored directly.
    /// Direct storage is more efficient, but growing the element formula will break layout compatibility of this formula,
    /// unless this is the last element in a record or tuple.
    #[allow(non_camel_case_types)]
    struct __Alkahest_Element<T>(PhantomData<T>);
}
