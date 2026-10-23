use core::{convert::Infallible, fmt};

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::cold_err;

/// Buffer API that is used by serializer.
/// Buffers can be extensible or fixed size.
/// Extensible buffers grow automatically when needed.
pub trait Buffer {
    /// Write error.
    type Error: 'static;

    /// Reborrowed buffer type.
    type Reborrow<'a>: Buffer<Error = Self::Error>
    where
        Self: 'a;

    const RESERVED_IS_SELF: bool;

    /// Reborrow this buffer.
    fn reborrow(&mut self) -> Self::Reborrow<'_>;

    /// Ensures that at least `additional` bytes can be written.
    ///
    /// Growable buffers should grow if needed, fixed buffers should return error if they cannot fit additional bytes.
    fn reserve(&mut self, heap: usize, stack: usize, additional: usize) -> Result<(), Self::Error>;

    /// Reserves and returns reserved kind of buffer.
    fn reserved(
        &mut self,
        heap: usize,
        stack: usize,
        additional: usize,
    ) -> Result<Option<OptimisticBuffer<'_>>, Self::Error>;

    /// Writes bytes to the stack.
    ///
    /// Space must be reserved before writing.
    fn write_stack(&mut self, stack: usize, bytes: &[u8]);

    /// Moves `len` bytes from stack to heap.
    fn move_to_heap(&mut self, heap: usize, stack: usize, len: usize);

    /// Ensures that at least `additional` bytes can be written.
    ///
    /// Growable buffers should grow if needed, fixed buffers should return error if they cannot fit additional bytes.
    ///
    /// Returns reserved kind of buffer that can be used to write exactly to `additional` bytes.
    /// This means that writing to either stack or heap is possible, with guarantee
    /// that if `additional` bytes are written in total, they will fit in the buffer withtout a gap.
    ///
    /// If more space is available, returns buffer aligned to heap.
    ///
    /// Returned buffer has same heap size filled, but stack is empty.
    fn reserved_heap(
        &mut self,
        heap: usize,
        stack: usize,
        additional: usize,
    ) -> Result<Option<OptimisticBuffer<'_>>, Self::Error>;
}

/// No-op buffer that does not write anything.
/// Used to measure the size of serialized data.
#[derive(Clone, Copy, Default)]
pub struct DryBuffer;

impl Buffer for DryBuffer {
    type Error = Infallible;
    type Reborrow<'a> = Self;

    const RESERVED_IS_SELF: bool = true;

    #[inline(always)]
    fn reborrow(&mut self) -> DryBuffer {
        *self
    }

    #[inline(always)]
    fn reserve(&mut self, _heap: usize, _stack: usize, _len: usize) -> Result<(), Infallible> {
        Ok(())
    }

    #[inline(always)]
    fn reserved(
        &mut self,
        _heap: usize,
        _stack: usize,
        _len: usize,
    ) -> Result<Option<OptimisticBuffer<'_>>, Infallible> {
        Ok(None)
    }

    #[inline(always)]
    fn write_stack(&mut self, _stack: usize, _bytes: &[u8]) {}

    #[inline(always)]
    fn move_to_heap(&mut self, _heap: usize, stack: usize, len: usize) {
        debug_assert!(stack >= len);
    }

    #[inline(always)]
    fn reserved_heap(
        &mut self,
        _heap: usize,
        _stack: usize,
        _len: usize,
    ) -> Result<Option<OptimisticBuffer<'_>>, Infallible> {
        Ok(None)
    }
}

// /// Fixed buffer without bound checks.
// /// If buffer is too small to fit serialized data, it will panic.
// impl<'a> Buffer for &'a mut [u8] {
//     // Panics rather than returning an error.
//     type Error = Infallible;

//     type Reborrow<'b>
//         = &'b mut [u8]
//     where
//         'a: 'b;

//     const RESERVED_IS_SELF: bool = true;

//     #[inline(always)]
//     fn reborrow(&mut self) -> &'_ mut [u8] {
//         self
//     }

//     #[inline(always)]
//     fn reserve(&mut self, heap: usize, stack: usize, len: usize) -> Result<(), Infallible> {
//         debug_assert!(
//             self.len() >= heap && self.len() - heap >= stack,
//             "{} > {} + {}",
//             self.len(),
//             heap,
//             stack
//         );

//         debug_assert!(self.len() - heap - stack >= len);
//         Ok(())
//     }

//     #[inline(always)]
//     fn reserved(
//         &mut self,
//         heap: usize,
//         stack: usize,
//         len: usize,
//     ) -> Result<Option<OptimisticBuffer<'_>>, Infallible> {
//         debug_assert!(
//             self.len() >= heap && self.len() - heap >= stack,
//             "{} > {} + {}",
//             self.len(),
//             heap,
//             stack
//         );
//         debug_assert!(self.len() - heap - stack >= len);
//         Ok(Some(OptimisticBuffer { buf: self }))
//     }

//     #[inline(always)]
//     fn write_stack(&mut self, stack: usize, bytes: &[u8]) {
//         assert!(self.len() >= stack && self.len() - stack >= bytes.len());

//         let at = self.len() - stack - bytes.len();
//         self[at..][..bytes.len()].copy_from_slice(bytes);
//     }

//     #[inline(always)]
//     fn move_to_heap(&mut self, heap: usize, stack: usize, len: usize) {
//         assert!(self.len() >= heap && self.len() - heap >= stack && stack >= len);

//         let start = self.len() - stack;
//         if start == heap {
//             return;
//         }

//         let end = start + len;
//         self.copy_within(start..end, heap);
//     }

//     #[inline(always)]
//     fn reserved_heap(
//         &mut self,
//         heap: usize,
//         stack: usize,
//         len: usize,
//     ) -> Result<Option<OptimisticBuffer<'_>>, Infallible> {
//         debug_assert!(self.len() >= heap && self.len() - heap >= stack);
//         assert!(self.len() >= heap && self.len() - heap >= len);

//         let end = heap + len;
//         Ok(Some(OptimisticBuffer {
//             buf: &mut self[..end],
//         }))
//     }
// }

#[repr(transparent)]
pub struct OptimisticBuffer<'a> {
    buf: &'a mut [u8],
}

impl<'a> OptimisticBuffer<'a> {
    #[inline(always)]
    pub const fn empty() -> OptimisticBuffer<'a> {
        OptimisticBuffer { buf: &mut [] }
    }

    #[inline(always)]
    pub fn new(buf: &'a mut [u8]) -> Self {
        OptimisticBuffer { buf }
    }

    #[inline(always)]
    pub fn reborrow(&mut self) -> OptimisticBuffer<'_> {
        OptimisticBuffer { buf: &mut self.buf }
    }

    #[inline(always)]
    pub fn buf(self) -> &'a mut [u8] {
        self.buf
    }

    #[inline(always)]
    pub fn get_range(&mut self, offset: usize, len: usize) -> OptimisticBuffer<'_> {
        let buf_len = self.buf.len();
        if buf_len >= offset && buf_len - offset >= len {
            OptimisticBuffer {
                buf: &mut self.buf[offset..][..len],
            }
        } else {
            failed_optimism();
            OptimisticBuffer::empty()
        }
    }

    // #[inline(always)]
    // pub fn split_at(self, pos: usize) -> (Self, Self) {
    //     let buf_len = self.buf.len();
    //     if buf_len >= pos {
    //         let (head, tail) = self.buf.split_at_mut(pos);

    //         (
    //             OptimisticBuffer { buf: head },
    //             OptimisticBuffer { buf: tail },
    //         )
    //     } else {
    //         failed_optimism();
    //         (OptimisticBuffer::empty(), OptimisticBuffer::empty())
    //     }
    // }

    #[inline(always)]
    pub fn take_head(&mut self, pos: usize) -> Option<OptimisticBuffer<'_>> {
        let buf_len = self.buf.len();
        if buf_len >= pos {
            let (head, tail) = core::mem::replace(&mut self.buf, &mut []).split_at_mut(pos);
            self.buf = tail;
            Some(OptimisticBuffer { buf: head })
        } else {
            failed_optimism();
            None
        }
    }

    #[inline(always)]
    pub fn copy_from_slice(&mut self, slice: &[u8]) {
        if self.buf.len() == slice.len() {
            self.buf.copy_from_slice(slice);
        } else {
            failed_optimism();
        }
    }

    #[inline(always)]
    pub fn fill(&mut self, value: u8) {
        self.buf.fill(value);
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.buf.len()
    }
}

#[cold]
#[inline(always)]
fn failed_optimism() {
    // panic!();
}

/// Fixed buffer without bound checks.
/// If buffer is too small to fit serialized data, it will panic.
impl<'a> Buffer for OptimisticBuffer<'a> {
    // Panics rather than returning an error.
    type Error = Infallible;

    type Reborrow<'b>
        = OptimisticBuffer<'b>
    where
        'a: 'b;

    const RESERVED_IS_SELF: bool = true;

    #[inline(always)]
    fn reborrow(&mut self) -> OptimisticBuffer<'_> {
        OptimisticBuffer { buf: &mut self.buf }
    }

    #[inline(always)]
    fn reserve(&mut self, _heap: usize, _stack: usize, _len: usize) -> Result<(), Infallible> {
        Ok(())
    }

    #[inline(always)]
    fn reserved(
        &mut self,
        _heap: usize,
        _stack: usize,
        _len: usize,
    ) -> Result<Option<OptimisticBuffer<'_>>, Infallible> {
        Ok(Some(OptimisticBuffer { buf: &mut self.buf }))
    }

    #[inline(always)]
    fn write_stack(&mut self, stack: usize, bytes: &[u8]) {
        let buf_len = self.buf.len();
        if buf_len >= stack && buf_len - stack >= bytes.len() {
            let at = buf_len - stack - bytes.len();
            self.buf[at..][..bytes.len()].copy_from_slice(bytes);
        }
    }

    #[inline(always)]
    fn move_to_heap(&mut self, heap: usize, stack: usize, len: usize) {
        let buf_len = self.buf.len();

        if buf_len >= heap && buf_len - heap >= stack && stack >= len {
            let start = buf_len - stack;
            if start == heap {
                return;
            }

            let end = start + len;
            self.buf.copy_within(start..end, heap);
        }
    }

    #[inline(always)]
    fn reserved_heap(
        &mut self,
        heap: usize,
        _stack: usize,
        len: usize,
    ) -> Result<Option<OptimisticBuffer<'_>>, Infallible> {
        let buf_len = self.buf.len();

        let end = if buf_len >= heap && buf_len - heap >= len {
            heap + len
        } else {
            0
        };

        Ok(Some(OptimisticBuffer {
            buf: &mut self.buf[..end],
        }))
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
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "buffer exhausted")
    }
}

/// Fixed buffer with bound checks.
/// If buffer is too small to fit serialized data, it will return error.
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

    const RESERVED_IS_SELF: bool = false;

    #[inline(always)]
    fn reborrow(&mut self) -> CheckedFixedBuffer<'_> {
        CheckedFixedBuffer { buf: self.buf }
    }

    #[inline(always)]
    fn reserve(&mut self, heap: usize, stack: usize, len: usize) -> Result<(), BufferExhausted> {
        debug_assert!(self.buf.len() >= heap && self.buf.len() - heap >= stack);
        if self.buf.len() - heap - stack < len {
            return cold_err(BufferExhausted);
        }
        Ok(())
    }

    #[inline(always)]
    fn reserved(
        &mut self,
        heap: usize,
        stack: usize,
        len: usize,
    ) -> Result<Option<OptimisticBuffer<'_>>, BufferExhausted> {
        debug_assert!(self.buf.len() >= heap && self.buf.len() - heap >= stack);
        if self.buf.len() - heap - stack < len {
            return cold_err(BufferExhausted);
        }
        Ok(Some(OptimisticBuffer { buf: &mut self.buf }))
    }

    #[inline(always)]
    fn write_stack(&mut self, stack: usize, bytes: &[u8]) {
        let at = self.buf.len() - stack - bytes.len();
        self.buf[at..][..bytes.len()].copy_from_slice(bytes);
    }

    #[inline(always)]
    fn move_to_heap(&mut self, heap: usize, stack: usize, len: usize) {
        let start = self.buf.len() - stack;
        if start == heap {
            return;
        }

        let end = start + len;
        self.buf.copy_within(start..end, heap);
    }

    #[inline(always)]
    fn reserved_heap(
        &mut self,
        heap: usize,
        stack: usize,
        len: usize,
    ) -> Result<Option<OptimisticBuffer<'_>>, BufferExhausted> {
        debug_assert!(self.buf.len() >= heap && self.buf.len() - heap >= stack);

        if self.buf.len() - heap - stack < len {
            return cold_err(BufferExhausted);
        }

        let end = heap + len;
        Ok(Some(OptimisticBuffer {
            buf: &mut self.buf[..end],
        }))
    }
}

/// Buffer that writes to a slice.
/// If buffer is too small to fit serialized data it keeps pretends to work
/// and tracks the size of the values that would've been written.
/// Returns `BufferSizeRequired` error if serialized data is too big.
pub struct MaybeFixedBuffer<'a> {
    buf: &'a mut [u8],
    exhausted: &'a mut bool,
}

impl<'a> MaybeFixedBuffer<'a> {
    /// Creates a new buffer with exhausted flag.
    #[inline(always)]
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

    const RESERVED_IS_SELF: bool = true;

    #[inline(always)]
    fn reborrow(&mut self) -> MaybeFixedBuffer<'_> {
        MaybeFixedBuffer {
            buf: self.buf,
            exhausted: self.exhausted,
        }
    }

    #[inline(always)]
    fn reserve(&mut self, heap: usize, stack: usize, len: usize) -> Result<(), Infallible> {
        if *self.exhausted {
            return Ok(());
        }

        debug_assert!(self.buf.len() >= heap && self.buf.len() - heap >= stack);
        if self.buf.len() - heap - stack < len {
            *self.exhausted = true;
        }

        Ok(())
    }

    #[inline(always)]
    fn reserved(
        &mut self,
        heap: usize,
        stack: usize,
        len: usize,
    ) -> Result<Option<OptimisticBuffer<'_>>, Infallible> {
        if *self.exhausted {
            return Ok(None);
        }

        debug_assert!(self.buf.len() >= heap && self.buf.len() - heap >= stack);
        if self.buf.len() - heap - stack < len {
            *self.exhausted = true;
            return Ok(None);
        }

        Ok(Some(OptimisticBuffer { buf: self.buf }))
    }

    #[inline(always)]
    fn write_stack(&mut self, stack: usize, bytes: &[u8]) {
        if !*self.exhausted {
            let at = self.buf.len() - stack - bytes.len();
            self.buf[at..][..bytes.len()].copy_from_slice(bytes);
        }
    }

    #[inline(always)]
    fn move_to_heap(&mut self, heap: usize, stack: usize, len: usize) {
        if !*self.exhausted {
            let start = self.buf.len() - stack;
            if start == heap {
                return;
            }

            let end = start + len;
            self.buf.copy_within(start..end, heap);
        }
    }

    #[inline(always)]
    fn reserved_heap(
        &mut self,
        heap: usize,
        stack: usize,
        len: usize,
    ) -> Result<Option<OptimisticBuffer<'_>>, Infallible> {
        if *self.exhausted {
            return Ok(None);
        }

        debug_assert!(self.buf.len() >= heap && self.buf.len() - heap >= stack);
        if self.buf.len() - heap - stack < len {
            *self.exhausted = true;
            return Ok(None);
        }

        let end = heap + len;
        Ok(Some(OptimisticBuffer {
            buf: &mut self.buf[..end],
        }))
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
    #[inline]
    pub fn new(buf: &'a mut Vec<u8>) -> Self {
        VecBuffer { buf }
    }
}

#[cfg(feature = "alloc")]
impl VecBuffer<'_> {
    #[cold]
    #[inline(never)]
    fn do_reserve(&mut self, heap: usize, stack: usize, additional: usize) {
        let old_len = self.buf.len();
        self.buf.reserve(heap + stack + additional - old_len);
        self.buf.resize(self.buf.capacity(), 0);

        let new_len = self.buf.len();
        self.buf
            .copy_within(old_len - stack..old_len, new_len - stack);
    }

    /// Ensures that at least `additional` bytes
    /// can be written between first `heap` and last `stack` bytes.
    #[inline(always)]
    fn reserve_vec(&mut self, heap: usize, stack: usize, additional: usize) {
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

    const RESERVED_IS_SELF: bool = false;

    #[inline(always)]
    fn reborrow(&mut self) -> VecBuffer<'_> {
        VecBuffer { buf: self.buf }
    }

    #[inline(always)]
    fn reserve(&mut self, heap: usize, stack: usize, len: usize) -> Result<(), Infallible> {
        debug_assert!(self.buf.len() >= heap && self.buf.len() - heap >= stack);
        self.reserve_vec(heap, stack, len);
        Ok(())
    }

    #[inline(always)]
    fn reserved(
        &mut self,
        heap: usize,
        stack: usize,
        len: usize,
    ) -> Result<Option<OptimisticBuffer<'_>>, Infallible> {
        debug_assert!(self.buf.len() >= heap && self.buf.len() - heap >= stack);
        self.reserve_vec(heap, stack, len);
        Ok(Some(OptimisticBuffer {
            buf: self.buf.as_mut_slice(),
        }))
    }

    #[inline(always)]
    fn write_stack(&mut self, stack: usize, bytes: &[u8]) {
        OptimisticBuffer::new(self.buf.as_mut_slice()).write_stack(stack, bytes);
    }

    #[inline(always)]
    fn move_to_heap(&mut self, heap: usize, stack: usize, len: usize) {
        OptimisticBuffer::new(self.buf.as_mut_slice()).move_to_heap(heap, stack, len);
    }

    #[inline(always)]
    fn reserved_heap(
        &mut self,
        heap: usize,
        stack: usize,
        len: usize,
    ) -> Result<Option<OptimisticBuffer<'_>>, Infallible> {
        debug_assert!(self.buf.len() >= heap && self.buf.len() - heap >= stack);
        self.reserve_vec(heap, stack, len);

        let end = heap + len;
        Ok(Some(OptimisticBuffer {
            buf: &mut self.buf[..end],
        }))
    }
}
