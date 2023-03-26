#[cold]
#[inline(always)]
pub const fn cold() {}

#[cold]
#[inline(never)]
pub fn err<T, E>(err: E) -> Result<T, E> {
    Err(err)
}
