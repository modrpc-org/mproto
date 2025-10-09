use crate::{
    max, BaseLen, Compatible, Decode, DecodeCursor, DecodeError, DecodeResult, Encode,
    EncodeCursor, Lazy, Owned,
};

impl<O: Owned, E: Owned> Owned for Result<O, E> {
    type Lazy<'a> = Result<O::Lazy<'a>, E::Lazy<'a>>;

    fn lazy_to_owned(lazy: Self::Lazy<'_>) -> DecodeResult<Self> {
        match lazy {
            Ok(ok) => Ok(Ok(O::lazy_to_owned(ok)?)),
            Err(err) => Ok(Err(E::lazy_to_owned(err)?)),
        }
    }
}

impl<'a, O: Lazy<'a>, E: Lazy<'a>> Lazy<'a> for Result<O, E> {
    type Owned = Result<O::Owned, E::Owned>;
}

impl<T: BaseLen, E: BaseLen> BaseLen for Result<T, E> {
    const BASE_LEN: usize = 1 + max(T::BASE_LEN, E::BASE_LEN);
}

impl<T: Encode, E: Encode> Encode for Result<T, E> {
    fn scratch_len(&self) -> usize {
        match self {
            Ok(ok) => ok.scratch_len(),
            Err(err) => err.scratch_len(),
        }
    }

    fn encode(&self, cursor: &mut EncodeCursor) {
        match self {
            Ok(ok) => {
                cursor.base(1)[0] = 0;
                ok.encode(cursor);
                cursor.base(Self::BASE_LEN - 1 - T::BASE_LEN);
            }
            Err(err) => {
                cursor.base(1)[0] = 1;
                err.encode(cursor);
                cursor.base(Self::BASE_LEN - 1 - E::BASE_LEN);
            }
        }
    }
}

impl<'a, T: Decode<'a>, E: Decode<'a>> Decode<'a> for Result<T, E> {
    fn decode(cursor: &DecodeCursor<'a>) -> DecodeResult<Self> {
        let variant = cursor.base(1)[0];
        if variant > 1 {
            return Err(DecodeError);
        }

        let is_ok = variant == 0;
        if is_ok {
            let ok = T::decode(cursor)?;
            cursor.advance(Self::BASE_LEN - 1 - T::BASE_LEN);
            Ok(Ok(ok))
        } else {
            let err = E::decode(cursor)?;
            cursor.advance(Self::BASE_LEN - 1 - E::BASE_LEN);
            Ok(Err(err))
        }
    }
}

impl<Ok1, Err1, Ok2, Err2> Compatible<Result<Ok2, Err2>> for Result<Ok1, Err1>
where
    Ok1: Compatible<Ok2>,
    Err1: Compatible<Err2>,
    Ok2: Encode,
    Err2: Encode,
{
}
