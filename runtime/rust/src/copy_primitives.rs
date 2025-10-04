use crate::{
    BaseLen, Decode, DecodeCursor, DecodeError, DecodeResult, Encode, EncodeCursor, Lazy, Owned,
};

macro_rules! copy_primitive_owned_impl {
    ($t:ty) => {
        impl Owned for $t {
            type Lazy<'a> = $t;

            fn lazy_to_owned(lazy: Self::Lazy<'_>) -> DecodeResult<Self> {
                Ok(lazy)
            }
        }

        impl Lazy<'_> for $t {
            type Owned = $t;
        }
    };
}

copy_primitive_owned_impl!(());
copy_primitive_owned_impl!(bool);
copy_primitive_owned_impl!(u8);
copy_primitive_owned_impl!(u16);
copy_primitive_owned_impl!(u32);
copy_primitive_owned_impl!(u64);
copy_primitive_owned_impl!(i8);
copy_primitive_owned_impl!(i16);
copy_primitive_owned_impl!(i32);
copy_primitive_owned_impl!(i64);
copy_primitive_owned_impl!(f32);
copy_primitive_owned_impl!(f64);

impl BaseLen for () {
    const BASE_LEN: usize = 0;
}

impl Encode for () {
    fn scratch_len(&self) -> usize {
        0
    }

    fn encode(&self, _: &mut EncodeCursor) {}
}

impl<'a> Decode<'a> for () {
    fn decode(_: &DecodeCursor<'a>) -> DecodeResult<Self> {
        Ok(())
    }
}

impl BaseLen for bool {
    const BASE_LEN: usize = 1;
}

impl Encode for bool {
    fn scratch_len(&self) -> usize {
        0
    }

    fn encode(&self, cursor: &mut EncodeCursor) {
        cursor.base(1)[0] = if *self { 1 } else { 0 };
    }
}

impl<'a> Decode<'a> for bool {
    fn decode(cursor: &DecodeCursor<'a>) -> DecodeResult<Self> {
        let b = cursor.base(1)[0];
        if b <= 1 {
            Ok(b == 1)
        } else {
            Err(DecodeError)
        }
    }
}

impl BaseLen for u8 {
    const BASE_LEN: usize = 1;
}

impl Encode for u8 {
    fn scratch_len(&self) -> usize {
        0
    }

    fn encode(&self, cursor: &mut EncodeCursor) {
        cursor.base(1)[0] = *self;
    }
}

impl<'a> Decode<'a> for u8 {
    fn decode(cursor: &DecodeCursor<'a>) -> DecodeResult<Self> {
        Ok(cursor.base(1)[0])
    }
}

impl BaseLen for i8 {
    const BASE_LEN: usize = 1;
}

impl Encode for i8 {
    fn scratch_len(&self) -> usize {
        0
    }

    fn encode(&self, cursor: &mut EncodeCursor) {
        cursor.base(1)[0] = *self as u8;
    }
}

impl<'a> Decode<'a> for i8 {
    fn decode(cursor: &DecodeCursor<'a>) -> DecodeResult<Self> {
        Ok(cursor.base(1)[0] as i8)
    }
}

macro_rules! integer_primitive_encoding_impl {
    ($t:ty) => {
        impl BaseLen for $t {
            const BASE_LEN: usize = core::mem::size_of::<$t>();
        }

        impl Encode for $t {
            fn scratch_len(&self) -> usize {
                0
            }

            fn encode(&self, cursor: &mut EncodeCursor) {
                cursor
                    .base(<$t>::BASE_LEN)
                    .copy_from_slice(&self.to_le_bytes());
            }
        }

        impl<'a> Decode<'a> for $t {
            fn decode(cursor: &DecodeCursor<'a>) -> DecodeResult<Self> {
                Ok(<$t>::from_le_bytes(
                    cursor
                        .base(<$t>::BASE_LEN)
                        .try_into()
                        .map_err(|_| DecodeError)?,
                ))
            }
        }
    };
}

integer_primitive_encoding_impl!(u16);
integer_primitive_encoding_impl!(u32);
integer_primitive_encoding_impl!(u64);
integer_primitive_encoding_impl!(u128);
integer_primitive_encoding_impl!(i16);
integer_primitive_encoding_impl!(i32);
integer_primitive_encoding_impl!(i64);
integer_primitive_encoding_impl!(i128);
integer_primitive_encoding_impl!(f32);
integer_primitive_encoding_impl!(f64);
