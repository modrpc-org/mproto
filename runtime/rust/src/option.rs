use crate::{
    BaseLen, Compatible, Decode, DecodeCursor, DecodeError, DecodeResult, Encode, EncodeCursor,
    Lazy, Owned,
};

impl<T: Owned> Owned for Option<T> {
    type Lazy<'a> = Option<T::Lazy<'a>>;

    fn lazy_to_owned(lazy: Self::Lazy<'_>) -> DecodeResult<Self> {
        match lazy {
            Some(lazy) => Ok(Some(T::lazy_to_owned(lazy)?)),
            None => Ok(None),
        }
    }
}

impl<'a, T: Lazy<'a>> Lazy<'a> for Option<T> {
    type Owned = Option<T::Owned>;
}

impl<T, U: Compatible<T>> Compatible<Option<T>> for Option<U> {}

impl<T: BaseLen> BaseLen for Option<T> {
    const BASE_LEN: usize = 1 + T::BASE_LEN;
}

impl<T: Encode> Encode for Option<T> {
    fn scratch_len(&self) -> usize {
        match self {
            Some(some) => some.scratch_len(),
            None => 0,
        }
    }

    fn encode(&self, cursor: &mut EncodeCursor) {
        match self {
            Some(some) => {
                cursor.base(1)[0] = 1;
                some.encode(cursor);
            }
            None => {
                cursor.base(1)[0] = 0;
                cursor.base(T::BASE_LEN);
            }
        }
    }
}

impl<'a, T: Decode<'a>> Decode<'a> for Option<T> {
    fn decode(cursor: &DecodeCursor<'a>) -> DecodeResult<Self> {
        let variant = cursor.base(1)[0];
        if variant > 1 {
            return Err(DecodeError);
        }

        let is_some = variant == 1;
        if is_some {
            Ok(Some(T::decode(cursor)?))
        } else {
            cursor.advance(T::BASE_LEN);
            Ok(None)
        }
    }
}
