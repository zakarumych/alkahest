use crate::{
    buffer::Buffer,
    deserialize::{Deserialize, DeserializeError, Deserializer},
    formula::Formula,
    serialize::{write_bytes, Serialize, Sizes},
};

/// Formula for Variable-Length Quantity encoding.
///
/// If bit 8 is set then bits 0-6 contain length of the value in bytes.
/// Otherwise bits 4-7 contain the length of the value in bytes,
/// and bits 0-3 contain first 4 bits of the encoded value.
///
/// If bit 8 is set then bit 7 must be unset. It is reserved for
/// chaining value length to the next byte for big integers from
/// 2^ 63 and larger.
///
///
/// # Examples
///
/// Type of the value can be different than the type of the serialized value.
///
/// ```
/// # use alkahest::*;
///
/// let mut buffer = [0u8; 1024];
/// let (size, root) = serialize::<Vlq, u8>(0, &mut buffer).unwrap();
/// let value = deserialize_with_size::<Vlq, u16>(&buffer[..size], root).unwrap();
/// assert_eq!(0, value);
/// ```
///
/// It may be smaller, unlike fixed size formulas.
/// The only requirement is that the target type is large enough
/// to hold the value.
///
/// ```
/// # use alkahest::*;
///
/// let mut buffer = [0u8; 1024];
///
/// let (size, root) = serialize::<Vlq, u32>(8573, &mut buffer).unwrap();
/// let value = deserialize_with_size::<Vlq, u16>(&buffer[..size], root).unwrap();
/// assert_eq!(8573, value);
/// ```
///
/// If deserialize type can't fit the value, an error is returned.
///
/// ```
/// # use alkahest::*;
///
/// let mut buffer = [0u8; 1024];
///
/// let (size, root) = serialize::<Vlq, u32>(70000, &mut buffer).unwrap();
/// let err = deserialize_with_size::<Vlq, u16>(&buffer[..size], root).unwrap_err();
/// assert!(matches!(err, DeserializeError::IntegerOverflow));
/// ```
pub struct Vlq;

impl Formula for Vlq {
    const MAX_STACK_SIZE: Option<usize> = None;
    const EXACT_SIZE: bool = false;
    const HEAPLESS: bool = true;
}

trait VlqType: Copy {
    fn less_eq(&self, byte: u8) -> bool;

    /// Shifts the value right by 8 bits, and assigns the result to `self`.
    fn shr_byte_assign(&mut self) -> u8;

    /// Returns new value with the least significant byte set to `lsb`.
    fn from_lsb(lsb: u8) -> Self;

    /// Shifts left by 8 bits and sets the least significant byte to `lsb`.
    /// Returns false if shift would overflow.
    fn shl_byte_set(&mut self, lsb: u8) -> bool;
}

impl VlqType for u8 {
    #[inline(always)]
    fn less_eq(&self, byte: u8) -> bool {
        *self <= byte
    }

    #[inline(always)]
    fn shr_byte_assign(&mut self) -> u8 {
        core::mem::replace(self, 0)
    }

    #[inline(always)]
    fn from_lsb(lsb: u8) -> Self {
        lsb
    }

    #[inline(always)]
    fn shl_byte_set(&mut self, lsb: u8) -> bool {
        if *self > 0 {
            return false;
        }
        *self = lsb;
        true
    }
}

macro_rules! impl_vlq_int {
    ($($a:ident)*) => {
        $(
            impl VlqType for $a {
                #[inline(always)]
                fn less_eq(&self, byte: u8) -> bool {
                    *self <= $a::from(byte)
                }

                #[inline(always)]
                fn shr_byte_assign(&mut self) -> u8 {
                    let lsb = *self as u8;
                    *self >>= 8;
                    lsb
                }

                #[inline(always)]
                fn from_lsb(lsb: u8) -> Self {
                    $a::from(lsb)
                }

                #[inline(always)]
                fn shl_byte_set(&mut self, lsb: u8) -> bool {
                    if self.leading_zeros() < 8 {
                        return false;
                    }
                    *self <<= 8;
                    *self |= $a::from(lsb);
                    true
                }
            }
        )*
    };
}

impl_vlq_int!(u16 u32 u64 u128 usize);

impl<T> Serialize<Vlq> for T
where
    T: VlqType,
{
    #[inline(always)]
    fn size_hint(&self) -> Option<Sizes> {
        Some(size_hint(*self))
    }

    #[inline(always)]
    fn serialize<B>(self, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
    where
        B: Buffer,
    {
        serialize(self, sizes, buffer)
    }
}

impl<'de, T> Deserialize<'de, Vlq> for T
where
    T: VlqType,
{
    #[inline(always)]
    fn deserialize(de: Deserializer<'de>) -> Result<Self, DeserializeError> {
        deserialize(de)
    }

    #[inline(always)]
    fn deserialize_in_place(
        &mut self,
        deserializer: Deserializer<'de>,
    ) -> Result<(), DeserializeError> {
        *self = deserialize(deserializer)?;
        Ok(())
    }
}

#[inline(always)]
fn size_hint<T>(mut value: T) -> Sizes
where
    T: VlqType,
{
    let mut tail = 0;
    loop {
        if tail >= 8 {
            if value.less_eq(0) {
                break;
            }
        } else if value.less_eq(0xF) {
            break;
        }

        if tail == 63 {
            unimplemented!(
                "Encoding for values that require more than 63 bytes is not implemented yet."
            )
        }
        tail += 1;
        value.shr_byte_assign();
    }
    Sizes::with_stack(tail + 1)
}

#[inline(always)]
fn serialize<T, B>(mut value: T, sizes: &mut Sizes, buffer: B) -> Result<(), B::Error>
where
    T: VlqType,
    B: Buffer,
{
    let mut bytes = [0u8; 64];
    let mut tail = 0;

    loop {
        if tail >= 8 {
            if value.less_eq(0) {
                bytes[usize::from(tail)] = 0x80 | tail;
                return write_bytes(&bytes[..=usize::from(tail)], sizes, buffer);
            }
        } else if value.less_eq(0xF) {
            let lsb = value.shr_byte_assign();
            bytes[usize::from(tail)] = (tail << 4) | lsb;
            return write_bytes(&bytes[..=usize::from(tail)], sizes, buffer);
        }

        if tail == 63 {
            unimplemented!(
                "Encoding for values that require more than 63 bytes is not implemented yet."
            )
        }

        let lsb = value.shr_byte_assign();
        bytes[usize::from(tail)] = lsb;
        tail += 1;
    }
}

#[inline(always)]
fn deserialize<T>(mut de: Deserializer) -> Result<T, DeserializeError>
where
    T: VlqType,
{
    let header = de.read_bytes(1)?[0];

    let (tail, msb) = match header {
        0x00..=0x7F => (header >> 4, header & 0x0F),
        0x80..=0xBF => (header & 0x3F, 0),
        0xC0..=0xFF => {
            unimplemented!(
                "Decoding for values that require more than 63 bytes is not implemented yet."
            )
        }
    };

    let mut value = T::from_lsb(msb);

    let tail = de.read_bytes(usize::from(tail))?;

    for byte in tail.iter().rev() {
        if !value.shl_byte_set(*byte) {
            return Err(DeserializeError::IntegerOverflow);
        }
    }

    Ok(value)
}
