use std::hint::black_box;

pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
#[allow(non_camel_case_types)]
pub struct __Alkahest_Vector3StackSize<const __SIZE_BYTES: usize>;
impl<const __SIZE_BYTES: usize> ::alkahest::SizeType for __Alkahest_Vector3StackSize<__SIZE_BYTES> {
    const VALUE: ::alkahest::SizeBound = {
        let mut total = ::alkahest::stack_size::<f32, __SIZE_BYTES>();
        let next = ::alkahest::stack_size::<f32, __SIZE_BYTES>();
        if total.is_unbounded() && !next.is_zero() {
            {
                panic!(
                    "Composite formula contains stack-unbounded element that is not the last one",
                );
            };
        }
        total = total.add(next);
        let next = ::alkahest::stack_size::<f32, __SIZE_BYTES>();
        if total.is_unbounded() && !next.is_zero() {
            {
                panic!(
                    "Composite formula contains stack-unbounded element that is not the last one",
                );
            };
        }
        total = total.add(next);
        total
    };
}
#[allow(non_camel_case_types)]
pub struct __Alkahest_Vector3HeapSize<const __SIZE_BYTES: usize>;
impl<const __SIZE_BYTES: usize> ::alkahest::SizeType for __Alkahest_Vector3HeapSize<__SIZE_BYTES> {
    const VALUE: ::alkahest::SizeBound = ::alkahest::heap_size::<f32, __SIZE_BYTES>()
        .add(::alkahest::heap_size::<f32, __SIZE_BYTES>())
        .add(::alkahest::heap_size::<f32, __SIZE_BYTES>());
}
#[doc(hidden)]
#[allow(non_upper_case_globals)]
impl Vector3 {
    pub const __ALKAHEST_ORDER_OF_x: usize = 0;
    pub const __ALKAHEST_ORDER_OF_y: usize = 1;
    pub const __ALKAHEST_ORDER_OF_z: usize = 2;
    pub const __ALKAHEST_FIELD_COUNT: usize = 3usize;
    #[allow(unused)]
    fn __alkahest_construct(x: f32, y: f32, z: f32) -> Self {
        Vector3 { x, y, z }
    }
}
impl ::alkahest::Formula for Vector3 {
    type StackSize<const __SIZE_BYTES: usize> = __Alkahest_Vector3StackSize<__SIZE_BYTES>;
    type HeapSize<const __SIZE_BYTES: usize> = __Alkahest_Vector3HeapSize<__SIZE_BYTES>;
    const INHABITED: bool = ::alkahest::inhabited::<f32>()
        && ::alkahest::inhabited::<f32>()
        && ::alkahest::inhabited::<f32>();
}
impl ::alkahest::Serialize<Self> for Vector3 {
    #[inline]
    fn serialize<__Serializer>(
        &self,
        mut __serializer: __Serializer,
    ) -> ::alkahest::private::Result<(), __Serializer::Error>
    where
        __Serializer: ::alkahest::Serializer,
    {
        const {
            if !(<Self>::__ALKAHEST_ORDER_OF_x == 0usize) {
                panic!("assertion failed: <Self>::__ALKAHEST_ORDER_OF_x == 0usize",)
            }
        }
        const {
            if !(<Self>::__ALKAHEST_ORDER_OF_y == 1usize) {
                panic!("assertion failed: <Self>::__ALKAHEST_ORDER_OF_y == 1usize",)
            }
        }
        const {
            if !(<Self>::__ALKAHEST_ORDER_OF_z == 2usize) {
                panic!("assertion failed: <Self>::__ALKAHEST_ORDER_OF_z == 2usize",)
            }
        }
        let Vector3 { x, y, z } = self;
        ::alkahest::private::with_element(|Self { x, .. }: &Self| x)
            .serialize(x, &mut __serializer)?;
        ::alkahest::private::with_element(|Self { y, .. }: &Self| y)
            .serialize(y, &mut __serializer)?;
        ::alkahest::private::with_element(|Self { z, .. }: &Self| z)
            .serialize(z, &mut __serializer)?;
        Ok(())
    }
    #[inline]
    fn size_hint<const __SIZE_BYTES: usize>(&self) -> Option<::alkahest::Sizes> {
        let mut __total_size = ::alkahest::Sizes::ZERO;
        let Vector3 { x, y, z } = self;
        __total_size += ::alkahest::private::with_element(|Self { x, .. }: &Self| x)
            .size_hint::<_, __SIZE_BYTES>(x)?;
        __total_size += ::alkahest::private::with_element(|Self { y, .. }: &Self| y)
            .size_hint::<_, __SIZE_BYTES>(y)?;
        __total_size += ::alkahest::private::with_element(|Self { z, .. }: &Self| z)
            .size_hint::<_, __SIZE_BYTES>(z)?;
        ::alkahest::private::Some(__total_size)
    }
}
impl<'de> ::alkahest::Deserialize<'de, Self> for Vector3 {
    #[inline]
    fn deserialize<__Deserializer>(
        mut __deserializer: __Deserializer,
    ) -> ::alkahest::private::Result<Self, ::alkahest::DeserializeError>
    where
        __Deserializer: ::alkahest::Deserializer<'de>,
    {
        const {
            if !(<Self>::__ALKAHEST_ORDER_OF_x == 0usize) {
                panic!("assertion failed: <Self>::__ALKAHEST_ORDER_OF_x == 0usize",)
            }
        }
        const {
            if !(<Self>::__ALKAHEST_ORDER_OF_y == 1usize) {
                panic!("assertion failed: <Self>::__ALKAHEST_ORDER_OF_y == 1usize",)
            }
        }
        const {
            if !(<Self>::__ALKAHEST_ORDER_OF_z == 2usize) {
                panic!("assertion failed: <Self>::__ALKAHEST_ORDER_OF_z == 2usize",)
            }
        }
        let x = ::alkahest::private::with_element(|Self { x, .. }: &Self| x)
            .deserialize(&mut __deserializer)?;
        let y = ::alkahest::private::with_element(|Self { y, .. }: &Self| y)
            .deserialize(&mut __deserializer)?;
        let z = ::alkahest::private::with_element(|Self { z, .. }: &Self| z)
            .deserialize(&mut __deserializer)?;
        ::alkahest::private::Ok(Vector3 { x, y, z })
    }
    #[inline]
    fn deserialize_in_place<__Deserializer>(
        &mut self,
        mut __deserializer: __Deserializer,
    ) -> ::alkahest::private::Result<(), ::alkahest::DeserializeError>
    where
        __Deserializer: ::alkahest::Deserializer<'de>,
    {
        const {
            if !(<Self>::__ALKAHEST_ORDER_OF_x == 0usize) {
                panic!("assertion failed: <Self>::__ALKAHEST_ORDER_OF_x == 0usize",)
            }
        }
        const {
            if !(<Self>::__ALKAHEST_ORDER_OF_y == 1usize) {
                panic!("assertion failed: <Self>::__ALKAHEST_ORDER_OF_y == 1usize",)
            }
        }
        const {
            if !(<Self>::__ALKAHEST_ORDER_OF_z == 2usize) {
                panic!("assertion failed: <Self>::__ALKAHEST_ORDER_OF_z == 2usize",)
            }
        }
        let Vector3 { x, y, z } = self;
        ::alkahest::private::with_element(|Self { x, .. }: &Self| x)
            .deserialize_in_place(x, &mut __deserializer)?;
        ::alkahest::private::with_element(|Self { y, .. }: &Self| y)
            .deserialize_in_place(y, &mut __deserializer)?;
        ::alkahest::private::with_element(|Self { z, .. }: &Self| z)
            .deserialize_in_place(z, &mut __deserializer)?;
        ::alkahest::private::Ok(())
    }
}
pub struct Triangle {
    pub v0: Vector3,
    pub v1: Vector3,
    pub v2: Vector3,
    pub normal: Vector3,
}
#[allow(non_camel_case_types)]
pub struct __Alkahest_TriangleStackSize<const __SIZE_BYTES: usize>;
impl<const __SIZE_BYTES: usize> ::alkahest::SizeType
    for __Alkahest_TriangleStackSize<__SIZE_BYTES>
{
    const VALUE: ::alkahest::SizeBound = {
        let mut total = ::alkahest::stack_size::<Vector3, __SIZE_BYTES>();
        let next = ::alkahest::stack_size::<Vector3, __SIZE_BYTES>();
        if total.is_unbounded() && !next.is_zero() {
            {
                panic!(
                    "Composite formula contains stack-unbounded element that is not the last one",
                );
            };
        }
        total = total.add(next);
        let next = ::alkahest::stack_size::<Vector3, __SIZE_BYTES>();
        if total.is_unbounded() && !next.is_zero() {
            {
                panic!(
                    "Composite formula contains stack-unbounded element that is not the last one",
                );
            };
        }
        total = total.add(next);
        let next = ::alkahest::stack_size::<Vector3, __SIZE_BYTES>();
        if total.is_unbounded() && !next.is_zero() {
            {
                panic!(
                    "Composite formula contains stack-unbounded element that is not the last one",
                );
            };
        }
        total = total.add(next);
        total
    };
}
#[allow(non_camel_case_types)]
pub struct __Alkahest_TriangleHeapSize<const __SIZE_BYTES: usize>;
impl<const __SIZE_BYTES: usize> ::alkahest::SizeType for __Alkahest_TriangleHeapSize<__SIZE_BYTES> {
    const VALUE: ::alkahest::SizeBound = ::alkahest::heap_size::<Vector3, __SIZE_BYTES>()
        .add(::alkahest::heap_size::<Vector3, __SIZE_BYTES>())
        .add(::alkahest::heap_size::<Vector3, __SIZE_BYTES>())
        .add(::alkahest::heap_size::<Vector3, __SIZE_BYTES>());
}
#[doc(hidden)]
#[allow(non_upper_case_globals)]
impl Triangle {
    pub const __ALKAHEST_ORDER_OF_v0: usize = 0;
    pub const __ALKAHEST_ORDER_OF_v1: usize = 1;
    pub const __ALKAHEST_ORDER_OF_v2: usize = 2;
    pub const __ALKAHEST_ORDER_OF_normal: usize = 3;
    pub const __ALKAHEST_FIELD_COUNT: usize = 4usize;
    #[allow(unused)]
    fn __alkahest_construct(v0: Vector3, v1: Vector3, v2: Vector3, normal: Vector3) -> Self {
        Triangle { v0, v1, v2, normal }
    }
}
impl ::alkahest::Formula for Triangle {
    type StackSize<const __SIZE_BYTES: usize> = __Alkahest_TriangleStackSize<__SIZE_BYTES>;
    type HeapSize<const __SIZE_BYTES: usize> = __Alkahest_TriangleHeapSize<__SIZE_BYTES>;
    const INHABITED: bool = ::alkahest::inhabited::<Vector3>()
        && ::alkahest::inhabited::<Vector3>()
        && ::alkahest::inhabited::<Vector3>()
        && ::alkahest::inhabited::<Vector3>();
}
impl ::alkahest::Serialize<Self> for Triangle {
    #[inline]
    fn serialize<__Serializer>(
        &self,
        mut __serializer: __Serializer,
    ) -> ::alkahest::private::Result<(), __Serializer::Error>
    where
        __Serializer: ::alkahest::Serializer,
    {
        const {
            if !(<Self>::__ALKAHEST_ORDER_OF_v0 == 0usize) {
                panic!("assertion failed: <Self>::__ALKAHEST_ORDER_OF_v0 == 0usize",)
            }
        }
        const {
            if !(<Self>::__ALKAHEST_ORDER_OF_v1 == 1usize) {
                panic!("assertion failed: <Self>::__ALKAHEST_ORDER_OF_v1 == 1usize",)
            }
        }
        const {
            if !(<Self>::__ALKAHEST_ORDER_OF_v2 == 2usize) {
                panic!("assertion failed: <Self>::__ALKAHEST_ORDER_OF_v2 == 2usize",)
            }
        }
        const {
            if !(<Self>::__ALKAHEST_ORDER_OF_normal == 3usize) {
                panic!("assertion failed: <Self>::__ALKAHEST_ORDER_OF_normal == 3usize",)
            }
        }
        let Triangle { v0, v1, v2, normal } = self;
        ::alkahest::private::with_element(|Self { v0, .. }: &Self| v0)
            .serialize(v0, &mut __serializer)?;
        ::alkahest::private::with_element(|Self { v1, .. }: &Self| v1)
            .serialize(v1, &mut __serializer)?;
        ::alkahest::private::with_element(|Self { v2, .. }: &Self| v2)
            .serialize(v2, &mut __serializer)?;
        ::alkahest::private::with_element(|Self { normal, .. }: &Self| normal)
            .serialize(normal, &mut __serializer)?;
        Ok(())
    }
    #[inline]
    fn size_hint<const __SIZE_BYTES: usize>(&self) -> Option<::alkahest::Sizes> {
        let mut __total_size = ::alkahest::Sizes::ZERO;
        let Triangle { v0, v1, v2, normal } = self;
        __total_size += ::alkahest::private::with_element(|Self { v0, .. }: &Self| v0)
            .size_hint::<_, __SIZE_BYTES>(v0)?;
        __total_size += ::alkahest::private::with_element(|Self { v1, .. }: &Self| v1)
            .size_hint::<_, __SIZE_BYTES>(v1)?;
        __total_size += ::alkahest::private::with_element(|Self { v2, .. }: &Self| v2)
            .size_hint::<_, __SIZE_BYTES>(v2)?;
        __total_size += ::alkahest::private::with_element(|Self { normal, .. }: &Self| normal)
            .size_hint::<_, __SIZE_BYTES>(normal)?;
        ::alkahest::private::Some(__total_size)
    }
}
impl<'de> ::alkahest::Deserialize<'de, Self> for Triangle {
    #[inline]
    fn deserialize<__Deserializer>(
        mut __deserializer: __Deserializer,
    ) -> ::alkahest::private::Result<Self, ::alkahest::DeserializeError>
    where
        __Deserializer: ::alkahest::Deserializer<'de>,
    {
        const {
            if !(<Self>::__ALKAHEST_ORDER_OF_v0 == 0usize) {
                panic!("assertion failed: <Self>::__ALKAHEST_ORDER_OF_v0 == 0usize",)
            }
        }
        const {
            if !(<Self>::__ALKAHEST_ORDER_OF_v1 == 1usize) {
                panic!("assertion failed: <Self>::__ALKAHEST_ORDER_OF_v1 == 1usize",)
            }
        }
        const {
            if !(<Self>::__ALKAHEST_ORDER_OF_v2 == 2usize) {
                panic!("assertion failed: <Self>::__ALKAHEST_ORDER_OF_v2 == 2usize",)
            }
        }
        const {
            if !(<Self>::__ALKAHEST_ORDER_OF_normal == 3usize) {
                panic!("assertion failed: <Self>::__ALKAHEST_ORDER_OF_normal == 3usize",)
            }
        }
        let v0 = ::alkahest::private::with_element(|Self { v0, .. }: &Self| v0)
            .deserialize(&mut __deserializer)?;
        let v1 = ::alkahest::private::with_element(|Self { v1, .. }: &Self| v1)
            .deserialize(&mut __deserializer)?;
        let v2 = ::alkahest::private::with_element(|Self { v2, .. }: &Self| v2)
            .deserialize(&mut __deserializer)?;
        let normal = ::alkahest::private::with_element(|Self { normal, .. }: &Self| normal)
            .deserialize(&mut __deserializer)?;
        ::alkahest::private::Ok(Triangle { v0, v1, v2, normal })
    }
    #[inline]
    fn deserialize_in_place<__Deserializer>(
        &mut self,
        mut __deserializer: __Deserializer,
    ) -> ::alkahest::private::Result<(), ::alkahest::DeserializeError>
    where
        __Deserializer: ::alkahest::Deserializer<'de>,
    {
        const {
            if !(<Self>::__ALKAHEST_ORDER_OF_v0 == 0usize) {
                panic!("assertion failed: <Self>::__ALKAHEST_ORDER_OF_v0 == 0usize",)
            }
        }
        const {
            if !(<Self>::__ALKAHEST_ORDER_OF_v1 == 1usize) {
                panic!("assertion failed: <Self>::__ALKAHEST_ORDER_OF_v1 == 1usize",)
            }
        }
        const {
            if !(<Self>::__ALKAHEST_ORDER_OF_v2 == 2usize) {
                panic!("assertion failed: <Self>::__ALKAHEST_ORDER_OF_v2 == 2usize",)
            }
        }
        const {
            if !(<Self>::__ALKAHEST_ORDER_OF_normal == 3usize) {
                panic!("assertion failed: <Self>::__ALKAHEST_ORDER_OF_normal == 3usize",)
            }
        }
        let Triangle { v0, v1, v2, normal } = self;
        ::alkahest::private::with_element(|Self { v0, .. }: &Self| v0)
            .deserialize_in_place(v0, &mut __deserializer)?;
        ::alkahest::private::with_element(|Self { v1, .. }: &Self| v1)
            .deserialize_in_place(v1, &mut __deserializer)?;
        ::alkahest::private::with_element(|Self { v2, .. }: &Self| v2)
            .deserialize_in_place(v2, &mut __deserializer)?;
        ::alkahest::private::with_element(|Self { normal, .. }: &Self| normal)
            .deserialize_in_place(normal, &mut __deserializer)?;
        ::alkahest::private::Ok(())
    }
}
pub struct Mesh {
    pub triangles: Vec<Triangle>,
}
#[allow(non_camel_case_types)]
pub struct __Alkahest_MeshStackSize<const __SIZE_BYTES: usize>;
impl<const __SIZE_BYTES: usize> ::alkahest::SizeType for __Alkahest_MeshStackSize<__SIZE_BYTES> {
    const VALUE: ::alkahest::SizeBound = ::alkahest::stack_size::<Vec<Triangle>, __SIZE_BYTES>();
}
#[allow(non_camel_case_types)]
pub struct __Alkahest_MeshHeapSize<const __SIZE_BYTES: usize>;
impl<const __SIZE_BYTES: usize> ::alkahest::SizeType for __Alkahest_MeshHeapSize<__SIZE_BYTES> {
    const VALUE: ::alkahest::SizeBound = ::alkahest::heap_size::<Vec<Triangle>, __SIZE_BYTES>();
}
#[doc(hidden)]
#[allow(non_upper_case_globals)]
impl Mesh {
    pub const __ALKAHEST_ORDER_OF_triangles: usize = 0;
    pub const __ALKAHEST_FIELD_COUNT: usize = 1usize;
    #[allow(unused)]
    fn __alkahest_construct(triangles: Vec<Triangle>) -> Self {
        Mesh { triangles }
    }
}
impl ::alkahest::Formula for Mesh {
    type StackSize<const __SIZE_BYTES: usize> = __Alkahest_MeshStackSize<__SIZE_BYTES>;
    type HeapSize<const __SIZE_BYTES: usize> = __Alkahest_MeshHeapSize<__SIZE_BYTES>;
    const INHABITED: bool = ::alkahest::inhabited::<Vec<Triangle>>();
}
impl ::alkahest::Serialize<Self> for Mesh {
    #[inline]
    fn serialize<__Serializer>(
        &self,
        mut __serializer: __Serializer,
    ) -> ::alkahest::private::Result<(), __Serializer::Error>
    where
        __Serializer: ::alkahest::Serializer,
    {
        const {
            if !(<Self>::__ALKAHEST_ORDER_OF_triangles == 0usize) {
                panic!("assertion failed: <Self>::__ALKAHEST_ORDER_OF_triangles == 0usize",)
            }
        }
        let Mesh { triangles } = self;
        ::alkahest::private::with_element(|Self { triangles, .. }: &Self| triangles)
            .serialize(triangles, &mut __serializer)?;
        Ok(())
    }
    #[inline]
    fn size_hint<const __SIZE_BYTES: usize>(&self) -> Option<::alkahest::Sizes> {
        let mut __total_size = ::alkahest::Sizes::ZERO;
        let Mesh { triangles } = self;
        __total_size +=
            ::alkahest::private::with_element(|Self { triangles, .. }: &Self| triangles)
                .size_hint::<_, __SIZE_BYTES>(triangles)?;
        ::alkahest::private::Some(__total_size)
    }
}
impl<'de> ::alkahest::Deserialize<'de, Self> for Mesh {
    #[inline]
    fn deserialize<__Deserializer>(
        mut __deserializer: __Deserializer,
    ) -> ::alkahest::private::Result<Self, ::alkahest::DeserializeError>
    where
        __Deserializer: ::alkahest::Deserializer<'de>,
    {
        const {
            if !(<Self>::__ALKAHEST_ORDER_OF_triangles == 0usize) {
                panic!("assertion failed: <Self>::__ALKAHEST_ORDER_OF_triangles == 0usize",)
            }
        }
        let triangles =
            ::alkahest::private::with_element(|Self { triangles, .. }: &Self| triangles)
                .deserialize(&mut __deserializer)?;
        ::alkahest::private::Ok(Mesh { triangles })
    }
    #[inline]
    fn deserialize_in_place<__Deserializer>(
        &mut self,
        mut __deserializer: __Deserializer,
    ) -> ::alkahest::private::Result<(), ::alkahest::DeserializeError>
    where
        __Deserializer: ::alkahest::Deserializer<'de>,
    {
        const {
            if !(<Self>::__ALKAHEST_ORDER_OF_triangles == 0usize) {
                panic!("assertion failed: <Self>::__ALKAHEST_ORDER_OF_triangles == 0usize",)
            }
        }
        let Mesh { triangles } = self;
        ::alkahest::private::with_element(|Self { triangles, .. }: &Self| triangles)
            .deserialize_in_place(triangles, &mut __deserializer)?;
        ::alkahest::private::Ok(())
    }
}
fn generate_mesh() -> Mesh {
    black_box(Mesh {
        triangles: vec![Triangle {
            v0: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            v1: Vector3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            v2: Vector3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            normal: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
        }],
    })
}

#[inline]
fn serialize_mesh(mesh: &Mesh, buffer: &mut [u8]) -> usize {
    alkahest::serialize_unchecked::<Mesh, Mesh>(mesh, buffer)
}

#[inline]
fn deserialize_mesh(buffer: &[u8]) -> Mesh {
    alkahest::deserialize::<Mesh, Mesh>(buffer).unwrap()
}

fn main() {
    let mut buffer: Vec<u8> = vec![0u8; 1000_000];
    let mesh = generate_mesh();
    let size = serialize_mesh(&mesh, &mut buffer[..]);
    let deserialized = deserialize_mesh(&buffer[..size]);
    black_box(deserialized);
}
