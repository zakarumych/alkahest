//! Runtime wire-version selection against a root formula's static version table.
//!
//! Bare functions take the wire version explicitly. Packet functions encode a
//! four-byte little-endian version followed by an address-width payload length
//! and the bare payload. Every address in the payload is payload-relative.

use core::{convert::Infallible, fmt};

use crate::{
    CompatibilityError, Deserialize, DeserializeError, Formula, Schema, Serialize, SizeBound,
    advanced::BufferExhausted, layout, schema, verify_compatibility,
};

/// Failure to select a declared wire contract, validate a frame, or run a codec.
#[derive(Clone, Copy, Debug)]
pub enum VersionError<E> {
    UnknownVersion(u32),
    DuplicateVersion(u32),
    MissingLocalVersion(u32),
    InvalidSchema,
    UnsupportedSizeBytes(usize),
    Incompatible {
        local_version: u32,
        wire_version: u32,
        reason: CompatibilityError,
    },
    LengthOverflow,
    PayloadLength {
        expected: usize,
        actual: usize,
    },
    Truncated {
        needed: usize,
        available: usize,
    },
    Codec(E),
}

impl<E: fmt::Display> fmt::Display for VersionError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownVersion(v) => write!(f, "wire version {v} is not declared"),
            Self::DuplicateVersion(v) => write!(f, "version {v} is declared more than once"),
            Self::MissingLocalVersion(v) => {
                write!(f, "local version {v} is absent from the version table")
            }
            Self::InvalidSchema => {
                f.write_str("local schema does not match its version-table declaration")
            }
            Self::UnsupportedSizeBytes(w) => write!(f, "unsupported address/length width: {w}"),
            Self::Incompatible {
                local_version,
                wire_version,
                reason,
            } => write!(
                f,
                "local version {local_version} and wire version {wire_version} are incompatible: {reason}"
            ),
            Self::LengthOverflow => {
                f.write_str("packet length exceeds the selected width or platform capacity")
            }
            Self::PayloadLength { expected, actual } => {
                write!(
                    f,
                    "formula requires {expected} payload bytes; received {actual}"
                )
            }
            Self::Truncated { needed, available } => {
                write!(f, "packet needs {needed} bytes; only {available} available")
            }
            Self::Codec(error) => error.fmt(f),
        }
    }
}

impl<E: fmt::Debug + fmt::Display> core::error::Error for VersionError<E> {}

fn validate_width<E>(width: usize) -> Result<(), VersionError<E>> {
    if (1..=16).contains(&width) {
        Ok(())
    } else {
        Err(VersionError::UnsupportedSizeBytes(width))
    }
}

fn select<F: Formula<LOCAL_VERSION> + ?Sized, const LOCAL_VERSION: u32, E>(
    wire_version: u32,
) -> Result<(&'static Schema, &'static Schema), VersionError<E>> {
    let local = schema::<F, LOCAL_VERSION>();
    if local.version != LOCAL_VERSION {
        return Err(VersionError::InvalidSchema);
    }
    let mut found_local = false;
    let mut wire = None;
    for (index, resolve) in F::VERSIONS.iter().enumerate() {
        let candidate = resolve();
        if F::VERSIONS[..index]
            .iter()
            .any(|other| other().version == candidate.version)
        {
            return Err(VersionError::DuplicateVersion(candidate.version));
        }
        if candidate.version == LOCAL_VERSION {
            found_local = true;
        }
        if candidate.version == wire_version {
            wire = Some(if wire_version == LOCAL_VERSION {
                local
            } else {
                candidate
            });
        }
    }
    if !found_local {
        return Err(VersionError::MissingLocalVersion(LOCAL_VERSION));
    }
    Ok((
        local,
        wire.ok_or(VersionError::UnknownVersion(wire_version))?,
    ))
}

fn validate<
    F: Formula<LOCAL_VERSION> + ?Sized,
    const SIZE_BYTES: usize,
    const LOCAL_VERSION: u32,
    E,
>(
    wire_version: u32,
    writing: bool,
) -> Result<(), VersionError<E>> {
    validate_width(SIZE_BYTES)?;
    let (local, wire) = select::<F, LOCAL_VERSION, E>(wire_version)?;
    if wire_version == LOCAL_VERSION {
        return Ok(());
    }
    let (reader, writer) = if writing {
        (wire, local)
    } else {
        (local, wire)
    };
    verify_compatibility(reader, writer, SIZE_BYTES).map_err(|reason| VersionError::Incompatible {
        local_version: LOCAL_VERSION,
        wire_version,
        reason,
    })
}

fn encode<F: Formula<V> + ?Sized, T: Serialize<F, V>, const V: u32>(
    value: &T,
    output: &mut [u8],
    width: usize,
) -> Result<usize, VersionError<BufferExhausted>> {
    with_size_bytes!(WIDTH = width => {
        crate::serialize::serialize_at_version::<F, T, WIDTH, V>(value, output).map_err(VersionError::Codec)
    } else { Err(VersionError::UnsupportedSizeBytes(width)) })
}

fn size<F: Formula<V> + ?Sized, T: Serialize<F, V>, const V: u32, E>(
    value: &T,
    width: usize,
) -> Result<usize, VersionError<E>> {
    with_size_bytes!(WIDTH = width => {
        Ok(crate::serialize::serialized_size_at_version::<F, T, WIDTH, V>(value))
    } else { Err(VersionError::UnsupportedSizeBytes(width)) })
}

fn validate_payload_length<F: Formula<V> + ?Sized, const V: u32, E>(
    actual: usize,
    width: usize,
) -> Result<(), VersionError<E>> {
    let layout = layout::<F, V>(width).ok_or(VersionError::UnsupportedSizeBytes(width))?;
    let exact = |bound| match bound {
        SizeBound::Exact(n) => Some(n),
        SizeBound::Bounded(0) => Some(0),
        _ => None,
    };
    if let (Some(stack), Some(heap)) = (exact(layout.stack), exact(layout.heap)) {
        let expected = stack
            .checked_add(heap)
            .ok_or(VersionError::LengthOverflow)?;
        if actual != expected {
            return Err(VersionError::PayloadLength { expected, actual });
        }
    }
    Ok(())
}

/// Serialize for a declared runtime wire version, using the local codec only
/// when its entire declared value domain is accepted by the target version.
/// Compatibility is checked before output is touched. No migration is performed.
pub fn serialize_versioned<F, T, const SIZE_BYTES: usize, const LOCAL_VERSION: u32>(
    value: &T,
    wire_version: u32,
    output: &mut [u8],
) -> Result<usize, VersionError<BufferExhausted>>
where
    F: Formula<LOCAL_VERSION> + ?Sized,
    T: Serialize<F, LOCAL_VERSION>,
{
    validate::<F, SIZE_BYTES, LOCAL_VERSION, _>(wire_version, true)?;
    encode::<F, T, LOCAL_VERSION>(value, output, SIZE_BYTES)
}

/// Size of a bare payload after runtime version validation.
pub fn serialized_size_versioned<F, T, const SIZE_BYTES: usize, const LOCAL_VERSION: u32>(
    value: &T,
    wire_version: u32,
) -> Result<usize, VersionError<Infallible>>
where
    F: Formula<LOCAL_VERSION> + ?Sized,
    T: Serialize<F, LOCAL_VERSION>,
{
    validate::<F, SIZE_BYTES, LOCAL_VERSION, _>(wire_version, true)?;
    size::<F, T, LOCAL_VERSION, _>(value, SIZE_BYTES)
}

/// Deserialize a bare payload whose runtime wire version is known externally.
pub fn deserialize_versioned<'de, F, T, const SIZE_BYTES: usize, const LOCAL_VERSION: u32>(
    input: &'de [u8],
    wire_version: u32,
) -> Result<T, VersionError<DeserializeError>>
where
    F: Formula<LOCAL_VERSION> + ?Sized,
    T: Deserialize<'de, F, LOCAL_VERSION>,
{
    validate::<F, SIZE_BYTES, LOCAL_VERSION, _>(wire_version, false)?;
    validate_payload_length::<F, LOCAL_VERSION, _>(input.len(), SIZE_BYTES)?;
    with_size_bytes!(WIDTH = SIZE_BYTES => {
        crate::deserialize::deserialize_at_version::<F, T, WIDTH, LOCAL_VERSION>(input).map_err(VersionError::Codec)
    } else { Err(VersionError::UnsupportedSizeBytes(SIZE_BYTES)) })
}

/// Validate the wire version before modifying the destination.
pub fn deserialize_in_place_versioned<
    'de,
    F,
    T,
    const SIZE_BYTES: usize,
    const LOCAL_VERSION: u32,
>(
    place: &mut T,
    input: &'de [u8],
    wire_version: u32,
) -> Result<(), VersionError<DeserializeError>>
where
    F: Formula<LOCAL_VERSION> + ?Sized,
    T: Deserialize<'de, F, LOCAL_VERSION> + ?Sized,
{
    validate::<F, SIZE_BYTES, LOCAL_VERSION, _>(wire_version, false)?;
    validate_payload_length::<F, LOCAL_VERSION, _>(input.len(), SIZE_BYTES)?;
    with_size_bytes!(WIDTH = SIZE_BYTES => {
        crate::deserialize::deserialize_in_place_at_version::<F, T, WIDTH, LOCAL_VERSION>(place, input).map_err(VersionError::Codec)
    } else { Err(VersionError::UnsupportedSizeBytes(SIZE_BYTES)) })
}

fn frame_size<E>(payload: usize, width: usize) -> Result<usize, VersionError<E>> {
    validate_width(width)?;
    if width < 16 && (payload as u128) >= (1u128 << (width * 8)) {
        return Err(VersionError::LengthOverflow);
    }
    payload
        .checked_add(4 + width)
        .ok_or(VersionError::LengthOverflow)
}

/// Size of a framed payload, including the version and length prefix.
pub fn pack_size_versioned<F, T, const SIZE_BYTES: usize, const LOCAL_VERSION: u32>(
    value: &T,
    wire_version: u32,
) -> Result<usize, VersionError<Infallible>>
where
    F: Formula<LOCAL_VERSION> + ?Sized,
    T: Serialize<F, LOCAL_VERSION>,
{
    validate::<F, SIZE_BYTES, LOCAL_VERSION, _>(wire_version, true)?;
    let payload = size::<F, T, LOCAL_VERSION, _>(value, SIZE_BYTES)?;
    frame_size(payload, SIZE_BYTES)
}

/// Write `[u32 LE version][SIZE_BYTES LE payload length][bare payload]`.
/// Addresses inside the payload are relative to its own first byte.
pub fn pack_versioned<F, T, const SIZE_BYTES: usize, const LOCAL_VERSION: u32>(
    value: &T,
    wire_version: u32,
    output: &mut [u8],
) -> Result<usize, VersionError<BufferExhausted>>
where
    F: Formula<LOCAL_VERSION> + ?Sized,
    T: Serialize<F, LOCAL_VERSION>,
{
    validate::<F, SIZE_BYTES, LOCAL_VERSION, _>(wire_version, true)?;
    let payload = size::<F, T, LOCAL_VERSION, _>(value, SIZE_BYTES)?;
    let total = frame_size(payload, SIZE_BYTES)?;
    if output.len() < total {
        return Err(VersionError::Codec(BufferExhausted));
    }
    let header = 4 + SIZE_BYTES;
    let written = encode::<F, T, LOCAL_VERSION>(value, &mut output[header..total], SIZE_BYTES)?;
    if written != payload {
        return Err(VersionError::InvalidSchema);
    }
    output[..4].copy_from_slice(&wire_version.to_le_bytes());
    output[4..header].copy_from_slice(&(payload as u128).to_le_bytes()[..SIZE_BYTES]);
    Ok(total)
}

fn frame<const SIZE_BYTES: usize, E>(input: &[u8]) -> Result<(u32, &[u8], usize), VersionError<E>> {
    validate_width(SIZE_BYTES)?;
    let header = 4 + SIZE_BYTES;
    if input.len() < header {
        return Err(VersionError::Truncated {
            needed: header,
            available: input.len(),
        });
    }
    let version = u32::from_le_bytes(input[..4].try_into().unwrap());
    let mut length = [0u8; 16];
    length[..SIZE_BYTES].copy_from_slice(&input[4..header]);
    let payload =
        usize::try_from(u128::from_le_bytes(length)).map_err(|_| VersionError::LengthOverflow)?;
    let total = payload
        .checked_add(header)
        .ok_or(VersionError::LengthOverflow)?;
    if input.len() < total {
        return Err(VersionError::Truncated {
            needed: total,
            available: input.len(),
        });
    }
    Ok((version, &input[header..total], total))
}

/// Read one versioned packet and return its total consumed length.
pub fn unpack_versioned<'de, F, T, const SIZE_BYTES: usize, const LOCAL_VERSION: u32>(
    input: &'de [u8],
) -> Result<(T, usize), VersionError<DeserializeError>>
where
    F: Formula<LOCAL_VERSION> + ?Sized,
    T: Deserialize<'de, F, LOCAL_VERSION>,
{
    let (version, payload, consumed) = frame::<SIZE_BYTES, _>(input)?;
    let value = deserialize_versioned::<F, T, SIZE_BYTES, LOCAL_VERSION>(payload, version)?;
    Ok((value, consumed))
}

/// Decode one packet in place and return its total consumed length.
pub fn unpack_in_place_versioned<'de, F, T, const SIZE_BYTES: usize, const LOCAL_VERSION: u32>(
    place: &mut T,
    input: &'de [u8],
) -> Result<usize, VersionError<DeserializeError>>
where
    F: Formula<LOCAL_VERSION> + ?Sized,
    T: Deserialize<'de, F, LOCAL_VERSION> + ?Sized,
{
    let (version, payload, consumed) = frame::<SIZE_BYTES, _>(input)?;
    deserialize_in_place_versioned::<F, T, SIZE_BYTES, LOCAL_VERSION>(place, payload, version)?;
    Ok(consumed)
}

macro_rules! fixed_width {
    ($name:ident, $width:literal) => {
        /// Runtime version APIs using this module's fixed address/length width.
        pub mod $name {
            use super::*;
            pub fn serialize_versioned<F, T, const LOCAL_VERSION: u32>(
                value: &T,
                wire_version: u32,
                output: &mut [u8],
            ) -> Result<usize, VersionError<BufferExhausted>>
            where
                F: Formula<LOCAL_VERSION> + ?Sized,
                T: Serialize<F, LOCAL_VERSION>,
            {
                super::serialize_versioned::<F, T, $width, LOCAL_VERSION>(
                    value,
                    wire_version,
                    output,
                )
            }
            pub fn serialized_size_versioned<F, T, const LOCAL_VERSION: u32>(
                value: &T,
                wire_version: u32,
            ) -> Result<usize, VersionError<Infallible>>
            where
                F: Formula<LOCAL_VERSION> + ?Sized,
                T: Serialize<F, LOCAL_VERSION>,
            {
                super::serialized_size_versioned::<F, T, $width, LOCAL_VERSION>(value, wire_version)
            }
            pub fn deserialize_versioned<'de, F, T, const LOCAL_VERSION: u32>(
                input: &'de [u8],
                wire_version: u32,
            ) -> Result<T, VersionError<DeserializeError>>
            where
                F: Formula<LOCAL_VERSION> + ?Sized,
                T: Deserialize<'de, F, LOCAL_VERSION>,
            {
                super::deserialize_versioned::<F, T, $width, LOCAL_VERSION>(input, wire_version)
            }
            pub fn deserialize_in_place_versioned<'de, F, T, const LOCAL_VERSION: u32>(
                place: &mut T,
                input: &'de [u8],
                wire_version: u32,
            ) -> Result<(), VersionError<DeserializeError>>
            where
                F: Formula<LOCAL_VERSION> + ?Sized,
                T: Deserialize<'de, F, LOCAL_VERSION> + ?Sized,
            {
                super::deserialize_in_place_versioned::<F, T, $width, LOCAL_VERSION>(
                    place,
                    input,
                    wire_version,
                )
            }
            pub fn pack_versioned<F, T, const LOCAL_VERSION: u32>(
                value: &T,
                wire_version: u32,
                output: &mut [u8],
            ) -> Result<usize, VersionError<BufferExhausted>>
            where
                F: Formula<LOCAL_VERSION> + ?Sized,
                T: Serialize<F, LOCAL_VERSION>,
            {
                super::pack_versioned::<F, T, $width, LOCAL_VERSION>(value, wire_version, output)
            }
            pub fn pack_size_versioned<F, T, const LOCAL_VERSION: u32>(
                value: &T,
                wire_version: u32,
            ) -> Result<usize, VersionError<Infallible>>
            where
                F: Formula<LOCAL_VERSION> + ?Sized,
                T: Serialize<F, LOCAL_VERSION>,
            {
                super::pack_size_versioned::<F, T, $width, LOCAL_VERSION>(value, wire_version)
            }
            pub fn unpack_versioned<'de, F, T, const LOCAL_VERSION: u32>(
                input: &'de [u8],
            ) -> Result<(T, usize), VersionError<DeserializeError>>
            where
                F: Formula<LOCAL_VERSION> + ?Sized,
                T: Deserialize<'de, F, LOCAL_VERSION>,
            {
                super::unpack_versioned::<F, T, $width, LOCAL_VERSION>(input)
            }
            pub fn unpack_in_place_versioned<'de, F, T, const LOCAL_VERSION: u32>(
                place: &mut T,
                input: &'de [u8],
            ) -> Result<usize, VersionError<DeserializeError>>
            where
                F: Formula<LOCAL_VERSION> + ?Sized,
                T: Deserialize<'de, F, LOCAL_VERSION> + ?Sized,
            {
                super::unpack_in_place_versioned::<F, T, $width, LOCAL_VERSION>(place, input)
            }
        }
    };
}

fixed_width!(small, 1);
fixed_width!(medium, 2);
fixed_width!(large, 4);
fixed_width!(huge, 8);
fixed_width!(humongous, 16);
