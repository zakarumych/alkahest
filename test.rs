#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2018::*;
#[macro_use]
extern crate std;
struct AlignedBytes<T, const N: usize> {
    _align: [T; 0],
    bytes: [u8; N],
}
fn aligned_bytes<T, const N: usize>(bytes: [u8; N]) -> AlignedBytes<T, N> {
    AlignedBytes { _align: [], bytes }
}
fn main() {
    pub struct TestStruct<T> {
        pub a: u32,
        pub b: T,
        c: alkahest::Seq<u32>,
        d: alkahest::Seq<alkahest::Seq<u32>>,
    }
    #[allow(dead_code)]
    pub struct TestStructUnpacked<'a, T>
    where
        T: ::alkahest::Schema,
    {
        pub a: <u32 as ::alkahest::SchemaUnpack<'a>>::Unpacked,
        pub b: <T as ::alkahest::SchemaUnpack<'a>>::Unpacked,
        c: <alkahest::Seq<u32> as ::alkahest::SchemaUnpack<'a>>::Unpacked,
        d: <alkahest::Seq<alkahest::Seq<u32>> as ::alkahest::SchemaUnpack<'a>>::Unpacked,
    }
    impl<'a, T> ::alkahest::SchemaUnpack<'a> for TestStruct<T>
    where
        T: ::alkahest::Schema,
    {
        type Unpacked = TestStructUnpacked<'a, T>;
    }
    #[repr(C, packed)]
    pub struct TestStructPacked<T>
    where
        T: ::alkahest::Schema,
    {
        pub a: <u32 as ::alkahest::Schema>::Packed,
        pub b: <T as ::alkahest::Schema>::Packed,
        c: <alkahest::Seq<u32> as ::alkahest::Schema>::Packed,
        d: <alkahest::Seq<alkahest::Seq<u32>> as ::alkahest::Schema>::Packed,
    }
    impl<T> ::core::clone::Clone for TestStructPacked<T>
    where
        T: ::alkahest::Schema,
    {
        #[inline]
        fn clone(&self) -> Self {
            *self
        }
    }
    impl<T> ::core::marker::Copy for TestStructPacked<T> where T: ::alkahest::Schema {}
    unsafe impl<T> ::alkahest::Zeroable for TestStructPacked<T> where T: ::alkahest::Schema {}
    unsafe impl<T> ::alkahest::Pod for TestStructPacked<T> where T: ::alkahest::Schema {}
    impl<T> ::alkahest::Schema for TestStruct<T>
    where
        T: ::alkahest::Schema,
    {
        type Packed = TestStructPacked<T>;
        #[inline]
        fn align() -> usize {
            1 + (0
                | (<u32 as ::alkahest::Schema>::align() - 1)
                | (<T as ::alkahest::Schema>::align() - 1)
                | (<alkahest::Seq<u32> as ::alkahest::Schema>::align() - 1)
                | (<alkahest::Seq<alkahest::Seq<u32>> as ::alkahest::Schema>::align() - 1))
        }
        #[inline]
        fn unpack<'a>(packed: TestStructPacked<T>, bytes: &'a [u8]) -> TestStructUnpacked<'a, T> {
            TestStructUnpacked {
                a: <u32 as ::alkahest::Schema>::unpack(packed.a, bytes),
                b: <T as ::alkahest::Schema>::unpack(packed.b, bytes),
                c: <alkahest::Seq<u32> as ::alkahest::Schema>::unpack(packed.c, bytes),
                d: <alkahest::Seq<alkahest::Seq<u32>> as ::alkahest::Schema>::unpack(
                    packed.d, bytes,
                ),
            }
        }
    }
    #[allow(dead_code)]
    pub struct TestStructPack<ALKAHEST_T0, ALKAHEST_T1, ALKAHEST_T2, ALKAHEST_T3> {
        pub a: ALKAHEST_T0,
        pub b: ALKAHEST_T1,
        c: ALKAHEST_T2,
        d: ALKAHEST_T3,
    }
    impl<T, ALKAHEST_T0, ALKAHEST_T1, ALKAHEST_T2, ALKAHEST_T3> ::alkahest::Pack<TestStruct<T>>
        for TestStructPack<ALKAHEST_T0, ALKAHEST_T1, ALKAHEST_T2, ALKAHEST_T3>
    where
        T: ::alkahest::Schema,
        ALKAHEST_T0: ::alkahest::Pack<u32>,
        ALKAHEST_T1: ::alkahest::Pack<T>,
        ALKAHEST_T2: ::alkahest::Pack<alkahest::Seq<u32>>,
        ALKAHEST_T3: ::alkahest::Pack<alkahest::Seq<alkahest::Seq<u32>>>,
    {
        #[inline]
        fn pack(self, offset: usize, bytes: &mut [u8]) -> (TestStructPacked<T>, usize) {
            let mut used = 0;
            let packed = TestStructPacked {
                a: {
                    let (packed, field_used) = self.a.pack(offset + used, &mut bytes[used..]);
                    used += field_used;
                    packed
                },
                b: {
                    let (packed, field_used) = self.b.pack(offset + used, &mut bytes[used..]);
                    used += field_used;
                    packed
                },
                c: {
                    let (packed, field_used) = self.c.pack(offset + used, &mut bytes[used..]);
                    used += field_used;
                    packed
                },
                d: {
                    let (packed, field_used) = self.d.pack(offset + used, &mut bytes[used..]);
                    used += field_used;
                    packed
                },
            };
            (packed, used)
        }
    }
    let mut data = aligned_bytes::<u32, 1024>([0; 1024]);
    alkahest::write::<TestStruct<f32>, _>(
        &mut data.bytes,
        TestStructPack {
            a: 11,
            b: 42.0,
            c: std::array::IntoIter::new([1, 2, 3]),
            d: (0..3).map(|i| (i..i + 4)),
        },
    );
    let test = alkahest::read::<TestStruct<f32>>(&data.bytes);
    {
        ::std::io::_print(::core::fmt::Arguments::new_v1_formatted(
            &["", "\n"],
            &match (&test.a,) {
                (arg0,) => [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)],
            },
            &[::core::fmt::rt::v1::Argument {
                position: 0usize,
                format: ::core::fmt::rt::v1::FormatSpec {
                    fill: ' ',
                    align: ::core::fmt::rt::v1::Alignment::Unknown,
                    flags: 4u32,
                    precision: ::core::fmt::rt::v1::Count::Implied,
                    width: ::core::fmt::rt::v1::Count::Implied,
                },
            }],
        ));
    };
    {
        ::std::io::_print(::core::fmt::Arguments::new_v1_formatted(
            &["", "\n"],
            &match (&test.b,) {
                (arg0,) => [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)],
            },
            &[::core::fmt::rt::v1::Argument {
                position: 0usize,
                format: ::core::fmt::rt::v1::FormatSpec {
                    fill: ' ',
                    align: ::core::fmt::rt::v1::Alignment::Unknown,
                    flags: 4u32,
                    precision: ::core::fmt::rt::v1::Count::Implied,
                    width: ::core::fmt::rt::v1::Count::Implied,
                },
            }],
        ));
    };
    {
        ::std::io::_print(::core::fmt::Arguments::new_v1_formatted(
            &["", "\n"],
            &match (&test.c.as_slice(),) {
                (arg0,) => [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)],
            },
            &[::core::fmt::rt::v1::Argument {
                position: 0usize,
                format: ::core::fmt::rt::v1::FormatSpec {
                    fill: ' ',
                    align: ::core::fmt::rt::v1::Alignment::Unknown,
                    flags: 4u32,
                    precision: ::core::fmt::rt::v1::Count::Implied,
                    width: ::core::fmt::rt::v1::Count::Implied,
                },
            }],
        ));
    };
    for d in test.d {
        {
            ::std::io::_print(::core::fmt::Arguments::new_v1_formatted(
                &["", "\n"],
                &match (&d.as_slice(),) {
                    (arg0,) => [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)],
                },
                &[::core::fmt::rt::v1::Argument {
                    position: 0usize,
                    format: ::core::fmt::rt::v1::FormatSpec {
                        fill: ' ',
                        align: ::core::fmt::rt::v1::Alignment::Unknown,
                        flags: 4u32,
                        precision: ::core::fmt::rt::v1::Count::Implied,
                        width: ::core::fmt::rt::v1::Count::Implied,
                    },
                }],
            ));
        };
    }
    # [alkahest (const 2)]
    #[allow(dead_code)]
    enum TestEnum<T> {
        Foo,
        Bar(T),
        Baz { val: f32 },
        Fuss { val: T, var: alkahest::Seq<u32> },
    }
    alkahest::write::<TestEnum<u64>, _>(&mut data.bytes, TestEnumFussPack { val: 4, var: 0..4 });
    let test = alkahest::read::<TestEnum<u64>>(&data.bytes);
    match test {
        TestEnumUnpacked::Foo => {
            ::std::io::_print(::core::fmt::Arguments::new_v1(
                &["Foo\n"],
                &match () {
                    () => [],
                },
            ));
        }
        TestEnumUnpacked::Bar(val) => {
            ::std::io::_print(::core::fmt::Arguments::new_v1(
                &["Bar(", ")\n"],
                &match (&val,) {
                    (arg0,) => [::core::fmt::ArgumentV1::new(
                        arg0,
                        ::core::fmt::Display::fmt,
                    )],
                },
            ));
        }
        TestEnumUnpacked::Baz { val } => {
            ::std::io::_print(::core::fmt::Arguments::new_v1(
                &["Bar{val: ", "}\n"],
                &match (&val,) {
                    (arg0,) => [::core::fmt::ArgumentV1::new(
                        arg0,
                        ::core::fmt::Display::fmt,
                    )],
                },
            ));
        }
        TestEnumUnpacked::Fuss { val, var } => {
            ::std::io::_print(::core::fmt::Arguments::new_v1(
                &["Fuss{val: ", ", var_sum: ", "}\n"],
                &match (&val, &var.into_iter().sum::<u32>()) {
                    (arg0, arg1) => [
                        ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Display::fmt),
                        ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Display::fmt),
                    ],
                },
            ));
        }
    }
}
