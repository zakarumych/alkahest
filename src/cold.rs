#[cold]
#[inline(always)]
pub const fn cold() {}

#[cold]
#[inline(always)]
#[track_caller]
pub fn err<T, E: core::fmt::Debug>(err: E) -> Result<T, E> {
    Err(err)
}
