use core::fmt::Debug;

use crate::{
    decode_value, encoded_len, BaseLen, Compatible, Decode, DecodeCursor, DecodeResult, Encode,
    EncodeCursor, Owned,
};

#[cfg(any(feature = "std", feature = "alloc"))]
use crate::{encode_value_vec, BoxLazy, ListLazy};

fn encode_decode<E, D>(v: E)
where
    E: Encode + Debug + PartialEq<D>,
    D: for<'a> Decode<'a> + Debug,
{
    const SOME_BIG_BUFFER_SIZE: usize = 1024;
    let mut buf = [0u8; SOME_BIG_BUFFER_SIZE];
    let encoded_len = encoded_len(&v);
    assert!(SOME_BIG_BUFFER_SIZE > encoded_len);

    let mut cursor = EncodeCursor::new::<E>(&mut buf);
    v.encode(&mut cursor);

    assert_eq!(cursor.encoded_len(), encoded_len);

    let decoded: D = decode_value(&buf).unwrap();
    assert_eq!(v, decoded);
}

fn encode_decode_owned<E>(v: E)
where
    E: Owned + Debug + PartialEq<E>,
{
    encode_decode::<E, E>(v);
}

fn encode_decode_with_buf<'a, E>(buf: &'a mut [u8], v: &E)
where
    E: Encode + Decode<'a> + Debug + PartialEq<E> + ?Sized,
{
    let mut cursor = EncodeCursor::new::<E>(buf);
    v.encode(&mut cursor);

    let decoded: E = decode_value(buf).unwrap();
    assert_eq!(decoded, *v);
}

#[test]
fn test_integers() {
    encode_decode_owned::<u8>(76);
    encode_decode_owned::<u16>(12345);
    encode_decode_owned::<u32>(1234567890);
    encode_decode_owned::<u64>(1234567890123456);

    encode_decode_owned::<i8>(-123);
    encode_decode_owned::<i8>(76);
    encode_decode_owned::<i16>(-12345);
    encode_decode_owned::<i16>(30111);
    encode_decode_owned::<i32>(-123456789);
    encode_decode_owned::<i32>(1234567890);
    encode_decode_owned::<i64>(-6786483465783829387);
    encode_decode_owned::<i64>(4893747873423431);
}

#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn test_string() {
    encode_decode_owned::<String>("hi i am a string".to_owned());
    encode_decode::<&str, String>("string string string");
}

#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn test_vec_u8() {
    encode_decode_owned::<Vec<u8>>(b"gimme some bytes".to_vec());
    encode_decode::<&[u8], Vec<u8>>(b"gimme some bytes");
}

#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn test_vec_string() {
    encode_decode_owned::<Vec<String>>(vec!["hubba bubba".into(), "ka pow".into(), "12345".into()]);
}

#[test]
fn test_option() {
    encode_decode_owned::<Option<u64>>(None);
    encode_decode_owned::<Option<u8>>(Some(42));
    encode_decode_owned::<Option<u16>>(Some(54321));
    encode_decode_owned::<Option<u32>>(Some(1234567890));
    encode_decode_owned::<Option<u64>>(Some(1234567890123456));
}

#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn test_option_vec() {
    encode_decode_owned::<Option<Vec<i64>>>(Some(vec![-6786483465783829387, -123, 456, 987654321]));
}

#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn test_vec_option() {
    encode_decode_owned::<Vec<Option<u64>>>(vec![None, Some(42), None, Some(43)]);
}

#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn test_vec_from_list_lazy() {
    use std::convert::TryInto;

    let original_vec = vec![
        "hubba bubba".to_string(),
        "ka pow".to_string(),
        "12345".to_string(),
    ];
    let buf = encode_value_vec(&original_vec);

    let list_lazy: ListLazy<'_, String> = decode_value(&buf).unwrap();
    let vec: Vec<&str> = list_lazy.try_into().unwrap();

    assert_eq!(vec, original_vec);
}

#[test]
fn test_result() {
    encode_decode_owned::<Result<u8, ()>>(Ok(42));
    encode_decode_owned::<Result<u8, i16>>(Err(-12345));
}

#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn test_box() {
    encode_decode_owned::<Box<u8>>(Box::new(42));
    encode_decode_owned::<Box<Result<u8, i16>>>(Box::new(Err(-12345)));
}

#[test]
fn test_custom_struct() {
    #[derive(Debug, PartialEq)]
    pub struct CustomStruct<'a> {
        a: u32,
        some_str: &'a str,
        b: u32,
    }

    impl<'a> BaseLen for CustomStruct<'a> {
        const BASE_LEN: usize = u32::BASE_LEN + str::BASE_LEN + u32::BASE_LEN;
    }

    impl<'a> Encode for CustomStruct<'a> {
        fn scratch_len(&self) -> usize {
            self.some_str.scratch_len()
        }

        fn encode(&self, cursor: &mut EncodeCursor) {
            self.a.encode(cursor);
            self.some_str.encode(cursor);
            self.b.encode(cursor);
        }
    }

    impl<'a> Decode<'a> for CustomStruct<'a> {
        fn decode(cursor: &DecodeCursor<'a>) -> DecodeResult<Self> {
            let a = Decode::decode(cursor)?;
            let some_str = Decode::decode(cursor)?;
            let b = Decode::decode(cursor)?;

            Ok(CustomStruct { a, some_str, b })
        }
    }

    let custom_struct = CustomStruct {
        a: 123,
        some_str: "some string",
        b: 456,
    };
    const SOME_BIG_BUFFER_SIZE: usize = 1024;
    let mut buf = [0u8; SOME_BIG_BUFFER_SIZE];
    assert!(SOME_BIG_BUFFER_SIZE > encoded_len(&custom_struct));
    encode_decode_with_buf::<CustomStruct>(&mut buf, &custom_struct);
}

// Tests for the Compatible trait impls

// TODO So far I haven't been able to implement full bidirectional coverage for the Compatible
// trait without the unstable `marker_trait_attr` feature. Luckily in the real-world we don't
// really need it - we usually just need to constrain that some passed in type's value is
// Compatible with some Owned type.
//fn assert_compatible<T: Compatible<U>, U: Compatible<T>>() {}
fn assert_compatible<T, U: Compatible<T>>() {}

#[allow(unused)]
#[cfg(any(feature = "std", feature = "alloc"))]
fn test_compatibility() {
    assert_compatible::<u32, u32>();
    assert_compatible::<u32, &u32>();

    assert_compatible::<Box<u32>, BoxLazy<u32>>();

    assert_compatible::<String, String>();
    assert_compatible::<String, &str>();

    assert_compatible::<Vec<u32>, ListLazy<u32>>();
    assert_compatible::<Vec<u32>, &[u32]>();
    assert_compatible::<ListLazy<u32>, &[u32]>();
    assert_compatible::<Vec<Box<u32>>, Vec<BoxLazy<u32>>>();
    assert_compatible::<Vec<Vec<u32>>, Vec<ListLazy<u32>>>();
    assert_compatible::<Vec<Vec<u32>>, ListLazy<Vec<u32>>>();
    assert_compatible::<Vec<String>, ListLazy<String>>();
    assert_compatible::<Vec<&str>, ListLazy<String>>();

    assert_compatible::<Option<Option<u32>>, Option<Option<u32>>>();
    assert_compatible::<Option<Option<u32>>, Option<Option<&u32>>>();
    assert_compatible::<Option<Vec<Option<&str>>>, Option<ListLazy<Option<String>>>>();
}

#[allow(unused)]
fn test_result_compatibility_bidirectional<Ok1, Err1, Ok2, Err2>()
where
    Ok1: Compatible<Ok1> + Compatible<Ok2>,
    Err1: Compatible<Err1> + Compatible<Err2>,
    Ok2: Compatible<Ok1> + Compatible<Ok2>,
    Err2: Compatible<Err1> + Compatible<Err2>,
{
    assert_compatible::<Result<Ok1, Err1>, Result<Ok1, Err1>>();
    assert_compatible::<Result<Ok2, Err2>, Result<Ok2, Err2>>();

    assert_compatible::<Result<Ok1, Err1>, Result<Ok2, Err2>>();
    assert_compatible::<Result<Ok2, Err2>, Result<Ok1, Err1>>();

    assert_compatible::<Result<Ok1, Err2>, Result<Ok2, Err1>>();
    assert_compatible::<Result<Ok2, Err1>, Result<Ok1, Err2>>();
}
