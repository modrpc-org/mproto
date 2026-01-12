const test = require('tape');
const {
  decodeValue, encodeValue,
  ProtoUint8, ProtoUint16, ProtoUint32, ProtoUint64,
  ProtoInt8, ProtoInt16, ProtoInt32, ProtoInt64,
  ProtoFloat32, ProtoFloat64,
  ProtoBox, ProtoList, ProtoString, ProtoVoid,
  ProtoOption,
  ProtoResult, Result,
} = require('../dist/index');

function testEncodeDecode(t, ty, v) {
  let buffer = encodeValue(ty, v);
  let decoded = decodeValue(ty, buffer, 0);
  t.deepEqual(decoded, v);
}

test("encode void", t => {
  t.plan(1);
  testEncodeDecode(t, ProtoVoid, undefined);
});

test("encode u8", t => {
  t.plan(2);
  testEncodeDecode(t, ProtoUint8, 42);
  testEncodeDecode(t, ProtoUint8, 43);
});

test("encode u16", t => {
  t.plan(2);
  testEncodeDecode(t, ProtoUint16, 1234);
  testEncodeDecode(t, ProtoUint16, 20000);
});

test("encode u32", t => {
  t.plan(2);
  testEncodeDecode(t, ProtoUint32, 20000);
  testEncodeDecode(t, ProtoUint32, 1234000001);
});

test("encode u64", t => {
  t.plan(4);
  testEncodeDecode(t, ProtoUint64, 24n);
  testEncodeDecode(t, ProtoUint64, 20000n);
  testEncodeDecode(t, ProtoUint64, 1234000001n);
  testEncodeDecode(t, ProtoUint64, 8394722983749283740n);
});

test("encode i8", t => {
  t.plan(2);
  testEncodeDecode(t, ProtoInt8, 42);
  testEncodeDecode(t, ProtoInt8, -43);
});

test("encode i16", t => {
  t.plan(2);
  testEncodeDecode(t, ProtoInt16, 1234);
  testEncodeDecode(t, ProtoInt16, -20000);
});

test("encode i32", t => {
  t.plan(2);
  testEncodeDecode(t, ProtoInt32, -20000);
  testEncodeDecode(t, ProtoInt32, 1234000001);
});

test("encode i64", t => {
  t.plan(4);
  testEncodeDecode(t, ProtoInt64, 24n);
  testEncodeDecode(t, ProtoInt64, -20000n);
  testEncodeDecode(t, ProtoInt64, 1234000001n);
  testEncodeDecode(t, ProtoInt64, -8394722983749283740n);
});

test("encode f32", t => {
  function testEncodeDecodeFloat32(t, ty, v) {
    let buffer = encodeValue(ty, v);
    let decoded = decodeValue(ty, buffer, 0);
    t.assert(Math.abs(decoded - v) < Math.abs(v * 0.000001));
  }

  t.plan(4);
  testEncodeDecodeFloat32(t, ProtoFloat32, 1.23);
  testEncodeDecodeFloat32(t, ProtoFloat32, -4.56);
  testEncodeDecodeFloat32(t, ProtoFloat32, 0.0003);
  testEncodeDecodeFloat32(t, ProtoFloat32, 100000000.3);
});

test("encode f64", t => {
  t.plan(4);
  testEncodeDecode(t, ProtoFloat64, 1.23);
  testEncodeDecode(t, ProtoFloat64, -4.56);
  testEncodeDecode(t, ProtoFloat64, 0.000314159);
  testEncodeDecode(t, ProtoFloat64, 10000000000.456);
});

test("encode string", t => {
  t.plan(4);
  testEncodeDecode(t, ProtoString, "");
  testEncodeDecode(t, ProtoString, "b");
  testEncodeDecode(t, ProtoString, "asdf 1234");
  testEncodeDecode(t, ProtoString, "Hello world ❤️");
});

test("encode list", t => {
  t.plan(6);
  testEncodeDecode(t, ProtoList(ProtoUint8), []);
  testEncodeDecode(t, ProtoList(ProtoUint8), [1, 2, 3, 4]);
  testEncodeDecode(t, ProtoList(ProtoUint64), [1000n, 2000000n, 3000000000n, 43274983748738n]);
  testEncodeDecode(t, ProtoList(ProtoString), [""]);
  testEncodeDecode(t, ProtoList(ProtoString), ["a", "asdf 1234", "Hello world ❤️"]);
  testEncodeDecode(t, ProtoList(ProtoList(ProtoString)), [
    ["a", "asdf 1234", "Hello world ❤️"],
    [],
    ["hello"],
    ["hello", "hello", "hello"],
  ]);
});

test("encode option", t => {
  t.plan(6);
  testEncodeDecode(t, ProtoOption(ProtoUint32), 42);
  testEncodeDecode(t, ProtoOption(ProtoUint32), 43);
  testEncodeDecode(t, ProtoOption(ProtoUint32), null);
  testEncodeDecode(t, ProtoOption(ProtoString), "something bad happened");
  testEncodeDecode(t, ProtoOption(ProtoString), "something very bad happened");
  testEncodeDecode(t, ProtoOption(ProtoString), null);
});

test("encode result", t => {
  t.plan(4);
  testEncodeDecode(t, ProtoResult(ProtoUint32, ProtoString), new Result.Ok(42));
  testEncodeDecode(t, ProtoResult(ProtoUint32, ProtoString), new Result.Ok(43));
  testEncodeDecode(t, ProtoResult(ProtoUint32, ProtoString), new Result.Err("something bad happened"));
  testEncodeDecode(t, ProtoResult(ProtoUint32, ProtoString), new Result.Err("something very bad happened"));
});

test("encode box", t => {
  t.plan(3);
  testEncodeDecode(t, ProtoBox(ProtoUint32), 42);
  testEncodeDecode(t, ProtoBox(ProtoResult(ProtoUint32, ProtoString)), new Result.Ok(43));
  testEncodeDecode(t, ProtoBox(ProtoResult(ProtoUint32, ProtoString)), new Result.Err("something bad happened"));
});

