#[cold]
#[inline(never)]
pub const fn cold() {}

#[cold]
#[inline(never)]
#[track_caller]
pub fn err<T, E: core::fmt::Debug>(err: E) -> Result<T, E> {
    Err(err)
}
