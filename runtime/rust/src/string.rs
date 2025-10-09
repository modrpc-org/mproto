use crate::{BaseLen, Compatible, Decode, DecodeCursor, DecodeError, DecodeResult, Encode, EncodeCursor};

#[cfg(any(feature = "std", feature = "alloc"))]
use crate::{Lazy, Owned};

#[cfg(any(feature = "std", feature = "alloc"))]
impl Compatible<String> for String {}
impl Compatible<str> for str {}
#[cfg(any(feature = "std", feature = "alloc"))]
impl Compatible<String> for str {}
#[cfg(any(feature = "std", feature = "alloc"))]
impl Compatible<str> for String {}

#[cfg(any(feature = "std", feature = "alloc"))]
impl Owned for String {
    type Lazy<'a> = &'a str;

    fn lazy_to_owned(lazy: Self::Lazy<'_>) -> DecodeResult<Self> {
        Ok(lazy.into())
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'a> Lazy<'a> for &'a str {
    type Owned = String;
}

impl BaseLen for str {
    const BASE_LEN: usize = 4 + 4;
}

impl Encode for str {
    fn scratch_len(&self) -> usize {
        self.len()
    }

    fn encode(&self, cursor: &mut EncodeCursor) {
        cursor
            .base(4)
            .copy_from_slice(&(self.len() as u32).to_le_bytes());
        let scratch_buf = cursor.scratch(self.len());
        scratch_buf.copy_from_slice(self.as_bytes());
    }
}

impl<'a> Decode<'a> for &'a str {
    fn decode(cursor: &DecodeCursor<'a>) -> DecodeResult<Self> {
        let len_bytes = cursor.base(4).try_into().map_err(|_| DecodeError)?;
        let len = u32::from_le_bytes(len_bytes) as usize;
        let scratch = cursor.scratch(len);

        let string = core::str::from_utf8(scratch).map_err(|_| DecodeError)?;

        Ok(string)
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl BaseLen for String {
    const BASE_LEN: usize = 4 + 4;
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl Encode for String {
    fn scratch_len(&self) -> usize {
        self.len()
    }

    fn encode(&self, cursor: &mut EncodeCursor) {
        cursor
            .base(4)
            .copy_from_slice(&(self.len() as u32).to_le_bytes());
        let scratch_buf = cursor.scratch(self.len());
        scratch_buf.copy_from_slice(self.as_bytes());
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'a> Decode<'a> for String {
    fn decode(cursor: &DecodeCursor<'a>) -> DecodeResult<Self> {
        let len_bytes = cursor.base(4).try_into().map_err(|_| DecodeError)?;
        let len = u32::from_le_bytes(len_bytes) as usize;
        let scratch = cursor.scratch(len);

        let string = core::str::from_utf8(scratch).map_err(|_| DecodeError)?;

        Ok(string.into())
    }
}
