#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use alkahest::{
    deserialize, serialize, serialized_size, Deserialize, Schema, Serialize, SizedSchema,
};
struct X;
#[automatically_derived]
impl ::core::clone::Clone for X {
    #[inline]
    fn clone(&self) -> X {
        *self
    }
}
#[automatically_derived]
impl ::core::marker::Copy for X {}
#[automatically_derived]
impl ::core::fmt::Debug for X {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(f, "X")
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for X {}
#[automatically_derived]
impl ::core::cmp::PartialEq for X {
    #[inline]
    fn eq(&self, other: &X) -> bool {
        true
    }
}
#[automatically_derived]
impl ::core::marker::StructuralEq for X {}
#[automatically_derived]
impl ::core::cmp::Eq for X {
    #[inline]
    #[doc(hidden)]
    #[no_coverage]
    fn assert_receiver_is_total_eq(&self) -> () {}
}
#[automatically_derived]
impl ::core::cmp::PartialOrd for X {
    #[inline]
    fn partial_cmp(&self, other: &X) -> ::core::option::Option<::core::cmp::Ordering> {
        ::core::option::Option::Some(::core::cmp::Ordering::Equal)
    }
}
#[automatically_derived]
impl ::core::cmp::Ord for X {
    #[inline]
    fn cmp(&self, other: &X) -> ::core::cmp::Ordering {
        ::core::cmp::Ordering::Equal
    }
}
impl X {
    #[doc(hidden)]
    #[inline(always)]
    pub const fn __alkahest_schema_field_count() -> [(); 0usize] {
        [(); 0usize]
    }
}
impl ::alkahest::Schema for X {}
impl ::alkahest::SizedSchema for X {
    const SIZE: ::alkahest::private::usize = 0;
}
impl ::alkahest::Serialize<Self> for X {
    fn serialize(
        self,
        offset: ::alkahest::private::usize,
        output: &mut [::alkahest::private::u8],
    ) -> ::alkahest::private::Result<
        (::alkahest::private::usize, ::alkahest::private::usize),
        ::alkahest::private::usize,
    > {
        use ::alkahest::private::Result;
        #[allow(unused)]
        let _ = || {};
        let mut ser = ::alkahest::Serializer::new(offset, output);
        #[allow(unused_mut)]
        let mut err = Result::<(), usize>::Ok(());
        err?;
        Result::Ok(ser.finish())
    }
    fn size(self) -> ::alkahest::private::usize {
        #[allow(unused_mut)]
        let mut size = 0;
        size
    }
}
impl<'ser> ::alkahest::Serialize<X> for &'ser X {
    fn serialize(
        self,
        offset: ::alkahest::private::usize,
        output: &mut [::alkahest::private::u8],
    ) -> ::alkahest::private::Result<
        (::alkahest::private::usize, ::alkahest::private::usize),
        ::alkahest::private::usize,
    > {
        use ::alkahest::private::Result;
        let mut ser = ::alkahest::Serializer::new(offset, output);
        #[allow(unused_mut)]
        let mut err = Result::<(), usize>::Ok(());
        err?;
        Result::Ok(ser.finish())
    }
    fn size(self) -> ::alkahest::private::usize {
        #[allow(unused_mut)]
        let mut size = 0;
        size
    }
}
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
    pub const fn __alkahest_schema_field_a_idx_is() -> [(); 0usize] {
        [(); 0usize]
    }
    #[doc(hidden)]
    #[inline(always)]
    pub const fn __alkahest_schema_field_b_idx_is() -> [(); 1usize] {
        [(); 1usize]
    }
    #[doc(hidden)]
    #[inline(always)]
    pub const fn __alkahest_schema_field_c_idx_is() -> [(); 2usize] {
        [(); 2usize]
    }
    #[doc(hidden)]
    #[inline(always)]
    pub const fn __alkahest_schema_field_count() -> [(); 3usize] {
        [(); 3usize]
    }
}
impl<T: ?Sized> ::alkahest::Schema for Test<T>
where
    u32: ::alkahest::Schema,
    X: ::alkahest::Schema,
    T: ::alkahest::Schema,
{}
#[alkahest(
    serialize(
        for<'ser,
        U>Test<[U]>where
        U:SizedSchema+'ser,
        T:'ser,
        &'ser
        T:Serialize<U>
    )
)]
#[alkahest(serialize(noref(for<U>Test<[U]>where U:SizedSchema, T:Serialize<U>)))]
#[alkahest(deserialize(for<'de, U>Test<[U]>where U:SizedSchema, T:Deserialize<'de, U>))]
struct TestS<T> {
    a: u32,
    b: X,
    c: Vec<T>,
}
#[automatically_derived]
impl<T: ::core::fmt::Debug> ::core::fmt::Debug for TestS<T> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field3_finish(
            f,
            "TestS",
            "a",
            &&self.a,
            "b",
            &&self.b,
            "c",
            &&self.c,
        )
    }
}
impl<T, U> ::alkahest::Serialize<Test<[U]>> for TestS<T>
where
    U: SizedSchema,
    T: Serialize<U>,
{
    fn serialize(
        self,
        offset: ::alkahest::private::usize,
        output: &mut [::alkahest::private::u8],
    ) -> ::alkahest::private::Result<
        (::alkahest::private::usize, ::alkahest::private::usize),
        ::alkahest::private::usize,
    > {
        use ::alkahest::private::Result;
        #[allow(unused)]
        let _ = || {
            let _: [(); 0usize] = <Test<[U]>>::__alkahest_schema_field_a_idx_is();
            let _: [(); 1usize] = <Test<[U]>>::__alkahest_schema_field_b_idx_is();
            let _: [(); 2usize] = <Test<[U]>>::__alkahest_schema_field_c_idx_is();
            let _: [(); 3usize] = <Test<[U]>>::__alkahest_schema_field_count();
        };
        let mut ser = ::alkahest::Serializer::new(offset, output);
        #[allow(unused_mut)]
        let mut err = Result::<(), usize>::Ok(());
        let with_schema = ::alkahest::private::with_schema(|s: &Test<[U]>| &s.a);
        if let Result::Err(size) = err {
            err = Result::Err(size + with_schema.size_value(self.a));
        } else {
            if let Result::Err(size) = with_schema.serialize_value(&mut ser, self.a) {
                err = Result::Err(size);
            }
        }
        let with_schema = ::alkahest::private::with_schema(|s: &Test<[U]>| &s.b);
        if let Result::Err(size) = err {
            err = Result::Err(size + with_schema.size_value(self.b));
        } else {
            if let Result::Err(size) = with_schema.serialize_value(&mut ser, self.b) {
                err = Result::Err(size);
            }
        }
        let with_schema = ::alkahest::private::with_schema(|s: &Test<[U]>| &s.c);
        if let Result::Err(size) = err {
            err = Result::Err(size + with_schema.size_value(self.c));
        } else {
            if let Result::Err(size) = with_schema.serialize_value(&mut ser, self.c) {
                err = Result::Err(size);
            }
        }
        err?;
        Result::Ok(ser.finish())
    }
    fn size(self) -> ::alkahest::private::usize {
        #[allow(unused_mut)]
        let mut size = 0;
        let with_schema = ::alkahest::private::with_schema(|s: &Test<[U]>| &s.a);
        size += with_schema.size_value(self.a);
        let with_schema = ::alkahest::private::with_schema(|s: &Test<[U]>| &s.b);
        size += with_schema.size_value(self.b);
        let with_schema = ::alkahest::private::with_schema(|s: &Test<[U]>| &s.c);
        size += with_schema.size_value(self.c);
        size
    }
}
impl<'ser, T, U> ::alkahest::Serialize<Test<[U]>> for &'ser TestS<T>
where
    U: SizedSchema + 'ser,
    T: 'ser,
    &'ser T: Serialize<U>,
{
    fn serialize(
        self,
        offset: ::alkahest::private::usize,
        output: &mut [::alkahest::private::u8],
    ) -> ::alkahest::private::Result<
        (::alkahest::private::usize, ::alkahest::private::usize),
        ::alkahest::private::usize,
    > {
        use ::alkahest::private::Result;
        let mut ser = ::alkahest::Serializer::new(offset, output);
        #[allow(unused_mut)]
        let mut err = Result::<(), usize>::Ok(());
        let with_schema = ::alkahest::private::with_schema(|s: &Test<[U]>| &s.a);
        if let Result::Err(size) = err {
            err = Result::Err(size + with_schema.size_value(&self.a));
        } else {
            if let Result::Err(size) = with_schema.serialize_value(&mut ser, &self.a) {
                err = Result::Err(size);
            }
        }
        let with_schema = ::alkahest::private::with_schema(|s: &Test<[U]>| &s.b);
        if let Result::Err(size) = err {
            err = Result::Err(size + with_schema.size_value(&self.b));
        } else {
            if let Result::Err(size) = with_schema.serialize_value(&mut ser, &self.b) {
                err = Result::Err(size);
            }
        }
        let with_schema = ::alkahest::private::with_schema(|s: &Test<[U]>| &s.c);
        if let Result::Err(size) = err {
            err = Result::Err(size + with_schema.size_value(&self.c));
        } else {
            if let Result::Err(size) = with_schema.serialize_value(&mut ser, &self.c) {
                err = Result::Err(size);
            }
        }
        err?;
        Result::Ok(ser.finish())
    }
    fn size(self) -> ::alkahest::private::usize {
        #[allow(unused_mut)]
        let mut size = 0;
        let with_schema = ::alkahest::private::with_schema(|s: &Test<[U]>| &s.a);
        size += with_schema.size_value(&self.a);
        let with_schema = ::alkahest::private::with_schema(|s: &Test<[U]>| &s.b);
        size += with_schema.size_value(&self.b);
        let with_schema = ::alkahest::private::with_schema(|s: &Test<[U]>| &s.c);
        size += with_schema.size_value(&self.c);
        size
    }
}
impl<'de, T, U> ::alkahest::Deserialize<'de, Test<[U]>> for TestS<T>
where
    U: SizedSchema,
    T: Deserialize<'de, U>,
{
    fn deserialize(
        len: ::alkahest::private::usize,
        input: &'de [::alkahest::private::u8],
    ) -> ::alkahest::private::Result<Self, ::alkahest::DeserializeError> {
        #[allow(unused)]
        let _ = || {
            let _: [(); 0usize] = <Test<[U]>>::__alkahest_schema_field_a_idx_is();
            let _: [(); 1usize] = <Test<[U]>>::__alkahest_schema_field_b_idx_is();
            let _: [(); 2usize] = <Test<[U]>>::__alkahest_schema_field_c_idx_is();
        };
        let _: [(); 3usize] = <Test<[U]>>::__alkahest_schema_field_count();
        let mut des = ::alkahest::Deserializer::new(len, input);
        let a = ::alkahest::private::with_schema(|s: &Test<[U]>| &s.a)
            .deserialize_sized(&mut des)?;
        let b = ::alkahest::private::with_schema(|s: &Test<[U]>| &s.b)
            .deserialize_sized(&mut des)?;
        let c = ::alkahest::private::with_schema(|s: &Test<[U]>| &s.c)
            .deserialize_rest(&mut des)?;
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
fn main() {
    let value = TestS {
        a: 1,
        b: X,
        c: <[_]>::into_vec(#[rustc_box] ::alloc::boxed::Box::new([2, 3])),
    };
    let size = serialized_size::<Test<[u32]>, _>(&value);
    {
        ::std::io::_print(
            ::core::fmt::Arguments::new_v1(
                &["size: ", "\n"],
                &[::core::fmt::ArgumentV1::new_display(&size)],
            ),
        );
    };
    let mut buffer = ::alloc::vec::from_elem(0, size);
    let size = serialize::<Test<[u32]>, _>(&value, &mut buffer).unwrap();
    match (&size, &buffer.len()) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    let (value, size) = deserialize::<Test<[u32]>, TestS<u32>>(&buffer).unwrap();
    match (&size, &buffer.len()) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    match (&value.a, &1) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    match (&value.b, &X) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    match (&value.c, &<[_]>::into_vec(#[rustc_box] ::alloc::boxed::Box::new([2, 3]))) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    {
        ::std::io::_print(
            ::core::fmt::Arguments::new_v1(
                &["", "\n"],
                &[::core::fmt::ArgumentV1::new_debug(&value)],
            ),
        );
    };
}
