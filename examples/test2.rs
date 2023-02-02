use alkahest::{
    deserialize, serialize, serialized_size, Deserialize, Formula, Ref, Serialize, UnsizedFormula,
};
struct X;
impl X {
    #[doc(hidden)]
    #[inline(always)]
    pub const fn __alkahest_formula_field_count() -> [(); 0usize] {
        [(); 0usize]
    }
}
impl ::alkahest::UnsizedFormula for X {}
impl<'de> ::alkahest::Deserialize<'de, Self> for X {
    fn deserialize(
        len: ::alkahest::private::usize,
        input: &'de [::alkahest::private::u8],
    ) -> ::alkahest::private::Result<Self, ::alkahest::DeserializeError> {
        #[allow(unused)]
        let _ = || {};
        let mut des = ::alkahest::Deserializer::new(len, input);
        des.finish_checked()?;
        ::alkahest::private::Result::Ok(X {})
    }
    fn deserialize_in_place(
        &mut self,
        len: usize,
        input: &[u8],
    ) -> Result<(), ::alkahest::DeserializeError> {
        ::core::panicking::panic("not yet implemented")
    }
}
struct Test<T: ?Sized> {
    a: u32,
    b: X,
    c: T,
}
impl<T: ?Sized> Test<T> {
    #[doc(hidden)]
    #[inline(always)]
    pub const fn __alkahest_formula_field_a_idx_is() -> [(); 0usize] {
        [(); 0usize]
    }
    #[doc(hidden)]
    #[inline(always)]
    pub const fn __alkahest_formula_field_b_idx_is() -> [(); 1usize] {
        [(); 1usize]
    }
    #[doc(hidden)]
    #[inline(always)]
    pub const fn __alkahest_formula_field_c_idx_is() -> [(); 2usize] {
        [(); 2usize]
    }
    #[doc(hidden)]
    #[inline(always)]
    pub const fn __alkahest_formula_field_count() -> [(); 3usize] {
        [(); 3usize]
    }
}
impl<T: ?Sized> ::alkahest::UnsizedFormula for Test<T>
where
    u32: ::alkahest::UnsizedFormula,
    X: ::alkahest::UnsizedFormula,
    T: ::alkahest::UnsizedFormula,
{
}
// #[alkahest(
//     deserialize(for<'de, U:?Sized>Test<U>where U:UnsizedFormula, T:Deserialize<'de, U>)
// )]
struct TestS<T> {
    a: u32,
    b: X,
    c: T,
}
impl<'de, T, U: ?Sized> ::alkahest::Deserialize<'de, Test<U>> for TestS<T>
where
    U: UnsizedFormula,
    T: Deserialize<'de, U>,
{
    fn deserialize(
        len: ::alkahest::private::usize,
        input: &'de [::alkahest::private::u8],
    ) -> ::alkahest::private::Result<Self, ::alkahest::DeserializeError> {
        #[allow(unused)]
        let _ = || {
            let _: [(); 0usize] = <Test<U>>::__alkahest_formula_field_a_idx_is();
            let _: [(); 1usize] = <Test<U>>::__alkahest_formula_field_b_idx_is();
            let _: [(); 2usize] = <Test<U>>::__alkahest_formula_field_c_idx_is();
        };
        let _: [(); 3usize] = <Test<U>>::__alkahest_formula_field_count();
        let mut des = ::alkahest::Deserializer::new(len, input);
        let a =
            ::alkahest::private::with_formula(|s: &Test<U>| &s.a).deserialize_sized(&mut des)?;
        let b =
            ::alkahest::private::with_formula(|s: &Test<U>| &s.b).deserialize_sized(&mut des)?;
        let c = ::alkahest::private::with_formula(|s: &Test<U>| &s.c).deserialize_rest(&mut des)?;
        des.finish_checked()?;
        ::alkahest::private::Result::Ok(TestS { a, b, c })
    }
    fn deserialize_in_place(
        &mut self,
        len: usize,
        input: &[u8],
    ) -> Result<(), ::alkahest::DeserializeError> {
        ::core::panicking::panic("not yet implemented")
    }
}
fn main() {}
