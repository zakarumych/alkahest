use core::{convert::Infallible, fmt};

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Buffer API that is used by serializer.
/// Buffers can be extensible or fixed size.
/// Extensible buffers grow automatically when needed.
pub trait Buffer {
    type Error;

    type Sub<'a>: Buffer<Error = Self::Error>
    where
        Self: 'a;

    fn sub(&mut self, stack: usize) -> Option<Self::Sub<'_>>;

    /// Writes bytes to the stack.
    fn write_stack(&mut self, heap: usize, stack: usize, bytes: &[u8]) -> Result<(), Self::Error>;

    /// Moves bytes from stack to heap.
    fn move_to_heap(&mut self, heap: usize, stack: usize, count: usize);

    /// Reserves heap space and returns a buffer over it.
    fn reserve_heap(
        &mut self,
        heap: usize,
        stack: usize,
        len: usize,
    ) -> Result<Option<FixedBuffer<'_>>, Self::Error>;

    /// Finalizes the buffer and returns the result.
    fn finish(self, heap: usize, stack: usize) -> Result<(), Self::Error>;
}

#[derive(Clone, Copy, Default)]
pub struct DryBuffer;

impl Buffer for DryBuffer {
    type Error = Infallible;
    type Sub<'a> = DryBuffer;

    #[inline(always)]
    fn sub(&mut self, _stack: usize) -> Option<DryBuffer> {
        Some(DryBuffer)
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
    fn move_to_heap(&mut self, _heap: usize, _stack: usize, _count: usize) {}

    #[inline(always)]
    fn reserve_heap(
        &mut self,
        _heap: usize,
        _stack: usize,
        _len: usize,
    ) -> Result<Option<FixedBuffer<'_>>, Infallible> {
        Ok(None)
    }

    #[inline(always)]
    fn finish(self, _heap: usize, _stack: usize) -> Result<(), Infallible> {
        Ok(())
    }
}

pub struct FixedBuffer<'a> {
    buf: &'a mut [u8],
}

impl<'a> FixedBuffer<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        FixedBuffer { buf }
    }
}

/// DeserializeError that may occur during serialization,
/// if buffer is too small to fit serialized data.
///
/// This type does not contain the size of the buffer required to fit serialized data.
/// To get the size use `serialize_or_size` function that returns `Result<usize, BufferSizeRequired>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BufferExhausted;

impl fmt::Display for BufferExhausted {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "buffer exhausted")
    }
}

impl<'a> Buffer for FixedBuffer<'a> {
    type Error = BufferExhausted;

    type Sub<'b> = FixedBuffer<'b> where 'a: 'b;

    #[inline(always)]
    fn sub(&mut self, stack: usize) -> Option<FixedBuffer<'_>> {
        let at = self.buf.len() - stack;
        Some(FixedBuffer {
            buf: &mut self.buf[..at],
        })
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
    fn move_to_heap(&mut self, heap: usize, stack: usize, count: usize) {
        debug_assert!(stack >= count);
        debug_assert!(heap + stack <= self.buf.len());
        let start = self.buf.len() - stack;
        let end = start + count;
        self.buf.copy_within(start..end, heap);
    }

    #[inline(always)]
    fn reserve_heap(
        &mut self,
        heap: usize,
        stack: usize,
        len: usize,
    ) -> Result<Option<FixedBuffer<'_>>, BufferExhausted> {
        debug_assert!(heap + stack <= self.buf.len());
        if self.buf.len() - heap - stack < len {
            return Err(BufferExhausted);
        }
        let end = heap + len;
        Ok(Some(FixedBuffer {
            buf: &mut self.buf[..end],
        }))
    }

    #[inline(always)]
    fn finish(self, _heap: usize, _stack: usize) -> Result<(), BufferExhausted> {
        Ok(())
    }
}

pub struct UncheckedFixedBuffer<'a> {
    buf: &'a mut [u8],
}

impl<'a> UncheckedFixedBuffer<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        UncheckedFixedBuffer { buf }
    }
}

impl<'a> Buffer for UncheckedFixedBuffer<'a> {
    type Error = Infallible;

    type Sub<'b> = UncheckedFixedBuffer<'b> where 'a: 'b;

    #[inline(always)]
    fn sub(&mut self, stack: usize) -> Option<UncheckedFixedBuffer<'_>> {
        let at = self.buf.len() - stack;
        Some(UncheckedFixedBuffer {
            buf: &mut self.buf[..at],
        })
    }

    #[inline(always)]
    fn write_stack(&mut self, heap: usize, stack: usize, bytes: &[u8]) -> Result<(), Infallible> {
        debug_assert!(heap + stack <= self.buf.len());
        let at = self.buf.len() - stack - bytes.len();
        self.buf[at..][..bytes.len()].copy_from_slice(bytes);
        Ok(())
    }

    #[inline(always)]
    fn move_to_heap(&mut self, heap: usize, stack: usize, count: usize) {
        debug_assert!(stack >= count);
        debug_assert!(heap + stack <= self.buf.len());
        let start = self.buf.len() - stack;
        let end = start + count;
        self.buf.copy_within(start..end, heap);
    }

    #[inline(always)]
    fn reserve_heap(
        &mut self,
        heap: usize,
        stack: usize,
        len: usize,
    ) -> Result<Option<FixedBuffer<'_>>, Infallible> {
        debug_assert!(heap + stack <= self.buf.len());
        let end = heap + len;
        Ok(Some(FixedBuffer {
            buf: &mut self.buf[..end],
        }))
    }

    #[inline(always)]
    fn finish(self, _heap: usize, _stack: usize) -> Result<(), Infallible> {
        Ok(())
    }
}

#[derive(Default)]
pub struct MaybeFixedBuffer<'a> {
    buf: Option<&'a mut [u8]>,
}

impl<'a> MaybeFixedBuffer<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        MaybeFixedBuffer { buf: Some(buf) }
    }
}

/// DeserializeError that may occur during serialization,
/// if buffer is too small to fit serialized data.
///
/// Contains the size of the buffer required to fit serialized data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct BufferSizeRequired {
    pub required: usize,
}

impl fmt::Display for BufferSizeRequired {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "buffer size required: {}", self.required)
    }
}

impl<'a> Buffer for MaybeFixedBuffer<'a> {
    type Error = BufferSizeRequired;
    type Sub<'b> = MaybeFixedBuffer<'b> where 'a: 'b;

    #[inline(always)]
    fn sub(&mut self, stack: usize) -> Option<MaybeFixedBuffer<'_>> {
        if let Some(buf) = &mut self.buf {
            let at = buf.len() - stack;
            Some(MaybeFixedBuffer {
                buf: Some(&mut buf[..at]),
            })
        } else {
            None
        }
    }

    #[inline(always)]
    fn write_stack(
        &mut self,
        heap: usize,
        stack: usize,
        bytes: &[u8],
    ) -> Result<(), BufferSizeRequired> {
        if let Some(buf) = &self.buf {
            debug_assert!(heap + stack <= buf.len());
            if buf.len() - heap - stack < bytes.len() {
                self.buf = None;
            }
        }

        if let Some(buf) = &mut self.buf {
            let at = buf.len() - stack - bytes.len();
            buf[at..][..bytes.len()].copy_from_slice(bytes);
        }
        Ok(())
    }

    #[inline(always)]
    fn move_to_heap(&mut self, heap: usize, stack: usize, count: usize) {
        debug_assert!(stack >= count);
        if let Some(buf) = &mut self.buf {
            debug_assert!(heap + stack <= buf.len());
            let start = buf.len() - stack;
            let end = start + count;
            buf.copy_within(start..end, heap);
        }
    }

    #[inline(always)]
    fn reserve_heap(
        &mut self,
        heap: usize,
        stack: usize,
        len: usize,
    ) -> Result<Option<FixedBuffer<'_>>, BufferSizeRequired> {
        if let Some(ref buf) = self.buf {
            debug_assert!(heap + stack <= buf.len());
            if buf.len() - heap - stack < len {
                self.buf = None;
            }
        }

        match self.buf {
            None => Ok(None),
            Some(ref mut buf) => {
                let end = heap + len;
                Ok(Some(FixedBuffer {
                    buf: &mut buf[..end],
                }))
            }
        }
    }

    #[inline(always)]
    fn finish(self, heap: usize, stack: usize) -> Result<(), BufferSizeRequired> {
        match self.buf {
            None => Err(BufferSizeRequired {
                required: heap + stack,
            }),
            Some(_) => Ok(()),
        }
    }
}

#[cfg(feature = "alloc")]
pub struct VecBuffer<'a> {
    pub buf: &'a mut Vec<u8>,
    pub stack_ext: usize,
}

#[cfg(feature = "alloc")]
impl<'a> VecBuffer<'a> {
    pub fn new(buf: &'a mut Vec<u8>) -> Self {
        VecBuffer { buf, stack_ext: 0 }
    }
}

#[cfg(feature = "alloc")]
impl VecBuffer<'_> {
    /// Ensures that at least `additional` bytes
    /// can be written between first `heap` and last `stack` bytes.
    fn reserve(&mut self, heap: usize, stack: usize, additional: usize) {
        let free = self.buf.len() - heap - stack - self.stack_ext;
        if free < additional {
            let old_len = self.buf.len();
            self.buf.reserve(additional - free);
            self.buf.resize(self.buf.capacity(), 0);
            let new_len = self.buf.len();
            let total_stack = stack - self.stack_ext;
            self.buf
                .copy_within(old_len - total_stack..old_len, new_len - total_stack);
        }
    }
}

#[cfg(feature = "alloc")]
impl<'a> Buffer for VecBuffer<'a> {
    type Error = Infallible;
    type Sub<'b> = VecBuffer<'b> where 'a: 'b;

    #[inline(always)]
    fn write_stack(&mut self, heap: usize, stack: usize, bytes: &[u8]) -> Result<(), Infallible> {
        debug_assert!(self.stack_ext + heap + stack <= self.buf.len());
        self.reserve(heap, stack, bytes.len());
        let at = self.buf.len() - self.stack_ext - stack - bytes.len();
        self.buf[at..][..bytes.len()].copy_from_slice(bytes);
        Ok(())
    }

    #[inline(always)]
    fn sub(&mut self, stack: usize) -> Option<VecBuffer<'_>> {
        Some(VecBuffer {
            buf: self.buf,
            stack_ext: self.stack_ext + stack,
        })
    }

    #[inline(always)]
    fn move_to_heap(&mut self, heap: usize, stack: usize, count: usize) {
        debug_assert!(self.stack_ext + heap + stack <= self.buf.len());
        debug_assert!(stack >= count);
        let at = self.buf.len() - self.stack_ext - stack;
        self.buf.copy_within(at..at + count, heap);
    }

    #[inline(always)]
    fn reserve_heap(
        &mut self,
        heap: usize,
        stack: usize,
        len: usize,
    ) -> Result<Option<FixedBuffer<'_>>, Infallible> {
        debug_assert!(self.stack_ext + heap + stack <= self.buf.len());
        self.reserve(heap, stack, len);
        let sub_buf = &mut self.buf[..heap + len];
        Ok(Some(FixedBuffer { buf: sub_buf }))
    }

    #[inline(always)]
    fn finish(self, _heap: usize, _stack: usize) -> Result<(), Infallible> {
        Ok(())
    }
}
