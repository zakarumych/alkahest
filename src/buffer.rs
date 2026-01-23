use core::{convert::Infallible, fmt};

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Buffer API that is used by serializer.
/// Buffers can be extensible or fixed size.
/// Extensible buffers grow automatically when needed.
pub trait Buffer {
    /// Write error.
    type Error;

    /// Reborrowed buffer type.
    type Reborrow<'a>: Buffer<Error = Self::Error>
    where
        Self: 'a;

    /// Reborrow this buffer.
    fn reborrow(&mut self) -> Self::Reborrow<'_>;

    /// Writes bytes to the stack.
    ///
    /// # Errors
    ///
    /// If buffer cannot write bytes, it should return `Err`.
    fn write_stack(&mut self, heap: usize, stack: usize, bytes: &[u8]) -> Result<(), Self::Error>;

    /// Add padding bytes to the stack.
    ///
    /// # Errors
    ///
    /// If buffer cannot add padding bytes, it should return `Err`.
    fn pad_stack(&mut self, heap: usize, stack: usize, len: usize) -> Result<(), Self::Error>;

    /// Moves bytes from stack to heap.
    fn move_to_heap(&mut self, heap: usize, stack: usize, len: usize);

    /// Reserves heap space and returns a mutable bytes slice reference.
    /// Which can be used as [`Buffer`].
    ///
    /// If buffer cannot reserve heap space, it may return either
    /// `Err` or `Ok([])`.
    /// If `Ok([])` is returned serializer should skip writing this
    /// part of the data and continue writing the rest.
    ///
    /// # Errors
    ///
    /// If buffer cannot reserve heap space, it should return `Err`.
    /// If nothing needs to be written to the reserved heap,
    /// it should return `Ok([])`.
    fn reserve_heap(
        &mut self,
        heap: usize,
        stack: usize,
        len: usize,
    ) -> Result<&mut [u8], Self::Error>;
}

/// No-op buffer that does not write anything.
/// Used to measure the size of serialized data.
#[derive(Clone, Copy, Default)]
pub struct DryBuffer;

impl Buffer for DryBuffer {
    type Error = Infallible;
    type Reborrow<'a> = Self;

    #[inline(always)]
    fn reborrow(&mut self) -> DryBuffer {
        *self
    }

    #[inline(always)]
    fn write_stack(
        &mut self,
        _heap: usize,
        _stack: usize,
        _bytes: &[u8],
    ) -> Result<(), Infallible> {
        Ok(())
    }

    #[inline(always)]
    fn pad_stack(&mut self, _heap: usize, _stack: usize, _len: usize) -> Result<(), Infallible> {
        Ok(())
    }

    #[inline(always)]
    fn move_to_heap(&mut self, _heap: usize, _stack: usize, _len: usize) {}

    #[inline(always)]
    fn reserve_heap(
        &mut self,
        _heap: usize,
        _stack: usize,
        _len: usize,
    ) -> Result<&mut [u8], Infallible> {
        Ok(&mut [])
    }
}

/// Error that may occur during serialization,
/// if buffer is too small to fit serialized data.
///
/// This type does not contain the size of the buffer required to fit serialized data.
/// To get the size use `serialize_or_size` function that returns `Result<usize, BufferSizeRequired>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BufferExhausted;

impl fmt::Display for BufferExhausted {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "buffer exhausted")
    }
}

/// Fixed buffer without bound checks.
/// If buffer is too small to fit serialized data, it will panic.
#[repr(transparent)]
pub struct CheckedFixedBuffer<'a> {
    buf: &'a mut [u8],
}

impl<'a> CheckedFixedBuffer<'a> {
    /// Creates a new buffer.
    #[inline(always)]
    pub fn new(buf: &'a mut [u8]) -> Self {
        CheckedFixedBuffer { buf }
    }
}

impl<'a> Buffer for CheckedFixedBuffer<'a> {
    type Error = BufferExhausted;
    type Reborrow<'b>
        = CheckedFixedBuffer<'b>
    where
        'a: 'b;

    #[inline(always)]
    fn reborrow(&mut self) -> CheckedFixedBuffer<'_> {
        CheckedFixedBuffer { buf: self.buf }
    }

    #[inline(always)]
    fn write_stack(
        &mut self,
        heap: usize,
        stack: usize,
        bytes: &[u8],
    ) -> Result<(), BufferExhausted> {
        debug_assert!(heap + stack <= self.buf.len());
        if self.buf.len() - heap - stack < bytes.len() {
            return Err(BufferExhausted);
        }
        let at = self.buf.len() - stack - bytes.len();
        self.buf[at..][..bytes.len()].copy_from_slice(bytes);
        Ok(())
    }

    #[inline(always)]
    fn pad_stack(&mut self, heap: usize, stack: usize, len: usize) -> Result<(), BufferExhausted> {
        debug_assert!(heap + stack <= self.buf.len());
        if self.buf.len() - heap - stack < len {
            return Err(BufferExhausted);
        }

        #[cfg(test)]
        {
            let at = self.buf.len() - stack - len;
            self.buf[at..][..len].fill(0);
        }
        Ok(())
    }

    #[inline(always)]
    fn move_to_heap(&mut self, heap: usize, stack: usize, len: usize) {
        debug_assert!(heap + stack <= self.buf.len());
        let start = self.buf.len() - stack;
        let end = start + len;
        self.buf.copy_within(start..end, heap);
    }

    #[inline(always)]
    fn reserve_heap(
        &mut self,
        heap: usize,
        stack: usize,
        len: usize,
    ) -> Result<&mut [u8], BufferExhausted> {
        debug_assert!(heap + stack <= self.buf.len());
        if self.buf.len() - heap - stack < len {
            return Err(BufferExhausted);
        }
        let end = heap + len;
        Ok(&mut self.buf[..end])
    }
}

impl<'a> Buffer for &'a mut [u8] {
    type Error = Infallible;

    type Reborrow<'b>
        = &'b mut [u8]
    where
        'a: 'b;

    #[inline(always)]
    fn reborrow(&mut self) -> &'_ mut [u8] {
        self
    }

    #[inline(always)]
    fn write_stack(&mut self, heap: usize, stack: usize, bytes: &[u8]) -> Result<(), Infallible> {
        debug_assert!(heap + stack <= self.len());
        let at = self.len() - stack - bytes.len();
        self[at..][..bytes.len()].copy_from_slice(bytes);
        Ok(())
    }

    #[inline(always)]
    fn pad_stack(&mut self, heap: usize, stack: usize, len: usize) -> Result<(), Infallible> {
        debug_assert!(heap + stack <= self.len());
        assert!(self.len() - heap - stack >= len);

        #[cfg(test)]
        {
            let at = self.len() - stack - len;
            self[at..][..len].fill(0);
        }
        Ok(())
    }

    #[inline(always)]
    fn move_to_heap(&mut self, heap: usize, stack: usize, len: usize) {
        debug_assert!(stack >= len);
        debug_assert!(heap + stack <= self.len());
        let start = self.len() - stack;
        let end = start + len;
        self.copy_within(start..end, heap);
    }

    #[inline(always)]
    fn reserve_heap(
        &mut self,
        heap: usize,
        stack: usize,
        len: usize,
    ) -> Result<&mut [u8], Infallible> {
        debug_assert!(heap + stack <= self.len());
        let end = heap + len;
        Ok(&mut self[..end])
    }
}

/// Buffer that writes to a slice.
/// If buffer is too small to fit serialized data it keeps pretends to work
/// and tracks the size of the values that would be written.
/// Returns `BufferSizeRequired` error if serialized data is too big.
pub struct MaybeFixedBuffer<'a> {
    buf: &'a mut [u8],
    exhausted: &'a mut bool,
}

impl<'a> MaybeFixedBuffer<'a> {
    /// Creates a new buffer with exhausted flag.
    pub fn new(buf: &'a mut [u8], exhausted: &'a mut bool) -> Self {
        MaybeFixedBuffer { buf, exhausted }
    }
}

impl<'a> Buffer for MaybeFixedBuffer<'a> {
    type Error = Infallible;

    type Reborrow<'b>
        = MaybeFixedBuffer<'b>
    where
        'a: 'b;

    #[inline(always)]
    fn reborrow(&mut self) -> MaybeFixedBuffer<'_> {
        MaybeFixedBuffer {
            buf: self.buf,
            exhausted: self.exhausted,
        }
    }

    #[inline(always)]
    fn write_stack(&mut self, heap: usize, stack: usize, bytes: &[u8]) -> Result<(), Infallible> {
        if !*self.exhausted {
            debug_assert!(heap + stack <= self.buf.len());
            if self.buf.len() - heap - stack < bytes.len() {
                *self.exhausted = true;
            }
        }

        if !*self.exhausted {
            let at = self.buf.len() - stack - bytes.len();
            self.buf[at..][..bytes.len()].copy_from_slice(bytes);
        }
        Ok(())
    }

    #[inline(always)]
    fn pad_stack(&mut self, heap: usize, stack: usize, len: usize) -> Result<(), Infallible> {
        if !*self.exhausted {
            debug_assert!(heap + stack <= self.buf.len());
            if self.buf.len() - heap - stack < len {
                *self.exhausted = true;
            }
        }
        Ok(())
    }

    #[inline(always)]
    fn move_to_heap(&mut self, heap: usize, stack: usize, len: usize) {
        debug_assert!(stack >= len);
        if !*self.exhausted {
            debug_assert!(heap + stack <= self.buf.len());
            let start = self.buf.len() - stack;
            let end = start + len;
            self.buf.copy_within(start..end, heap);
        }
    }

    #[inline(always)]
    fn reserve_heap(
        &mut self,
        heap: usize,
        stack: usize,
        len: usize,
    ) -> Result<&mut [u8], Infallible> {
        if !*self.exhausted {
            debug_assert!(heap + stack <= self.buf.len());
            if self.buf.len() - heap - stack < len {
                *self.exhausted = true;
            }
        }

        if *self.exhausted {
            Ok(&mut [])
        } else {
            let end = heap + len;
            Ok(&mut self.buf[..end])
        }
    }
}

/// Extensible buffer that writes to a vector.
/// If buffer is too small to fit serialized data it extends the vector.
/// Never returns an error, cannot fail to serialize data except for OOM error.
#[cfg(feature = "alloc")]
pub struct VecBuffer<'a> {
    buf: &'a mut Vec<u8>,
}

#[cfg(feature = "alloc")]
impl<'a> VecBuffer<'a> {
    /// Creates a new buffer that writes to the given vector.
    pub fn new(buf: &'a mut Vec<u8>) -> Self {
        VecBuffer { buf }
    }
}

#[cfg(feature = "alloc")]
impl VecBuffer<'_> {
    #[cold]
    fn do_reserve(&mut self, heap: usize, stack: usize, additional: usize) {
        let old_len = self.buf.len();
        self.buf.resize(heap + stack + additional, 0);
        let new_len = self.buf.len();
        self.buf
            .copy_within(old_len - stack..old_len, new_len - stack);
    }
    /// Ensures that at least `additional` bytes
    /// can be written between first `heap` and last `stack` bytes.
    fn reserve(&mut self, heap: usize, stack: usize, additional: usize) {
        let free = self.buf.len() - heap - stack;
        if free < additional {
            self.do_reserve(heap, stack, additional);
        }
    }
}

#[cfg(feature = "alloc")]
impl<'a> Buffer for VecBuffer<'a> {
    type Error = Infallible;
    type Reborrow<'b>
        = VecBuffer<'b>
    where
        'a: 'b;

    #[inline(always)]
    fn reborrow(&mut self) -> VecBuffer<'_> {
        VecBuffer { buf: self.buf }
    }

    #[inline(always)]
    fn write_stack(&mut self, heap: usize, stack: usize, bytes: &[u8]) -> Result<(), Infallible> {
        debug_assert!(heap + stack <= self.buf.len());
        self.reserve(heap, stack, bytes.len());
        let at = self.buf.len() - stack - bytes.len();
        self.buf[at..][..bytes.len()].copy_from_slice(bytes);
        Ok(())
    }

    #[inline(always)]
    fn pad_stack(&mut self, heap: usize, stack: usize, len: usize) -> Result<(), Infallible> {
        debug_assert!(heap + stack <= self.buf.len());
        self.reserve(heap, stack, len);

        #[cfg(test)]
        {
            let at = self.buf.len() - stack - len;
            self.buf[at..][..len].fill(0);
        }
        Ok(())
    }

    #[inline(always)]
    fn move_to_heap(&mut self, heap: usize, stack: usize, len: usize) {
        debug_assert!(heap + stack <= self.buf.len());
        debug_assert!(stack >= len);
        let at = self.buf.len() - stack;
        self.buf.copy_within(at..at + len, heap);
    }

    #[inline(always)]
    fn reserve_heap(
        &mut self,
        heap: usize,
        stack: usize,
        len: usize,
    ) -> Result<&mut [u8], Infallible> {
        debug_assert!(heap + stack <= self.buf.len());
        self.reserve(heap, stack, len);
        Ok(&mut self.buf[..heap + len])
    }
}
