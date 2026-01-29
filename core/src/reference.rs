#[cfg(feature = "alloc")]
use alloc::sync::Arc;

/// An enum that abstracts over owning or referencing a value of type `T`.
/// This is used when value may be created at compile time or runtime.
/// At compile time, a static reference is used.
/// At runtime, an owned `Arc<T>` is used.
pub(crate) enum Reference<T: ?Sized + 'static> {
    #[cfg(feature = "alloc")]
    Owned(Arc<T>),
    Static(&'static T),
}

impl<T> Reference<T>
where
    T: ?Sized + 'static,
{
    pub const fn from_static(value: &'static T) -> Self {
        Reference::Static(value)
    }

    #[cfg(feature = "alloc")]
    pub fn from_arc(value: Arc<T>) -> Self {
        Reference::Owned(value)
    }
}

impl<T> Reference<T> {
    #[cfg(feature = "alloc")]
    pub fn new(value: T) -> Self
    where
        T: Sized,
    {
        Reference::Owned(Arc::new(value))
    }
}

impl<T> Reference<[T]> {
    #[cfg(feature = "alloc")]
    pub fn from_vec(value: alloc::vec::Vec<T>) -> Self
    where
        T: Sized,
    {
        Reference::Owned(value.into())
    }

    #[cfg(feature = "alloc")]
    pub fn clone_from_slice(value: &[T]) -> Self
    where
        T: Clone + Sized,
    {
        Reference::Owned(value.into())
    }
}

impl Reference<str> {
    #[cfg(feature = "alloc")]
    pub fn clone_from_str(value: &str) -> Self {
        Reference::Owned(value.into())
    }
}

impl<T> AsRef<T> for Reference<T>
where
    T: ?Sized + 'static,
{
    #[inline(always)]
    fn as_ref(&self) -> &T {
        match self {
            #[cfg(feature = "alloc")]
            Reference::Owned(b) => &**b,
            Reference::Static(r) => &**r,
        }
    }
}

impl<T> Clone for Reference<T>
where
    T: ?Sized + 'static,
{
    #[inline(always)]
    fn clone(&self) -> Self {
        match self {
            #[cfg(feature = "alloc")]
            Reference::Owned(b) => Reference::Owned(b.clone()),
            Reference::Static(r) => Reference::Static(r),
        }
    }
}
