import { DecodeCursor, Decoder, EncodeCursor, Encoder } from './index';

export class VoidEncoder implements Encoder<void>, Decoder<void> {
  baseLength = () => 0;

  scratchLength(value: void): number { return 0; }

  encode(cursor: EncodeCursor, value: void) { }

  decode(cursor: DecodeCursor): void { }
}

export const ProtoVoid = new VoidEncoder();

export class BoolEncoder implements Encoder<boolean>, Decoder<boolean> {
  baseLength = () => 1;

  scratchLength(value: boolean): number { return 0; }

  encode(cursor: EncodeCursor, value: boolean) {
    cursor.buffer.setUint8(cursor.base(1), value ? 1 : 0);
  }

  decode(cursor: DecodeCursor): boolean {
    const b = cursor.buffer.getUint8(cursor.base(1));
    if (b > 1) {
      throw Error("Invalid Bool value.");
    }
    return b == 1;
  }
}

export const ProtoBool = new BoolEncoder();

export class Uint8Encoder implements Encoder<number>, Decoder<number> {
  baseLength = () => 1;

  scratchLength(value: number): number { return 0; }

  encode(cursor: EncodeCursor, value: number) {
    cursor.buffer.setUint8(cursor.base(1), value);
  }

  decode(cursor: DecodeCursor): number {
    return cursor.buffer.getUint8(cursor.base(1));
  }
}

export const ProtoUint8 = new Uint8Encoder();

export class Uint16Encoder implements Encoder<number>, Decoder<number> {
  baseLength = () => 2;

  scratchLength(value: number): number { return 0; }

  encode(cursor: EncodeCursor, value: number) {
    cursor.buffer.setUint16(cursor.base(2), value, true);
  }

  decode(cursor: DecodeCursor): number {
    return cursor.buffer.getUint16(cursor.base(2), true);
  }
}

export const ProtoUint16 = new Uint16Encoder();

export class Uint32Encoder implements Encoder<number>, Decoder<number> {
  baseLength = () => 4;

  scratchLength(value: number): number { return 0; }

  encode(cursor: EncodeCursor, value: number) {
    cursor.buffer.setUint32(cursor.base(4), value, true);
  }

  decode(cursor: DecodeCursor): number {
    return cursor.buffer.getUint32(cursor.base(4), true);
  }
}

export const ProtoUint32 = new Uint32Encoder();

export class Uint64Encoder implements Encoder<bigint>, Decoder<bigint> {
  baseLength = () => 8;

  scratchLength(value: bigint): number { return 0; }

  encode(cursor: EncodeCursor, value: bigint) {
    cursor.buffer.setBigUint64(cursor.base(8), value, true);
  }

  decode(cursor: DecodeCursor): bigint {
    return cursor.buffer.getBigUint64(cursor.base(8), true);
  }
}

export const ProtoUint64 = new Uint64Encoder();

export class Int8Encoder implements Encoder<number>, Decoder<number> {
  baseLength = () => 1;

  scratchLength(value: number): number { return 0; }

  encode(cursor: EncodeCursor, value: number) {
    cursor.buffer.setInt8(cursor.base(1), value);
  }

  decode(cursor: DecodeCursor): number {
    return cursor.buffer.getInt8(cursor.base(1));
  }
}

export const ProtoInt8 = new Int8Encoder();

export class Int16Encoder implements Encoder<number>, Decoder<number> {
  baseLength = () => 2;

  scratchLength(value: number): number { return 0; }

  encode(cursor: EncodeCursor, value: number) {
    cursor.buffer.setInt16(cursor.base(2), value, true);
  }

  decode(cursor: DecodeCursor): number {
    return cursor.buffer.getInt16(cursor.base(2), true);
  }
}

export const ProtoInt16 = new Int16Encoder();

export class Int32Encoder implements Encoder<number>, Decoder<number> {
  baseLength = () => 4;

  scratchLength(value: number): number { return 0; }

  encode(cursor: EncodeCursor, value: number) {
    cursor.buffer.setInt32(cursor.base(4), value, true);
  }

  decode(cursor: DecodeCursor): number {
    return cursor.buffer.getInt32(cursor.base(4), true);
  }
}

export const ProtoInt32 = new Int32Encoder();

export class Int64Encoder implements Encoder<bigint>, Decoder<bigint> {
  baseLength = () => 8;

  scratchLength(value: bigint): number { return 0; }

  encode(cursor: EncodeCursor, value: bigint) {
    cursor.buffer.setBigInt64(cursor.base(8), value);
  }

  decode(cursor: DecodeCursor): bigint {
    return cursor.buffer.getBigInt64(cursor.base(8));
  }
}

export const ProtoInt64 = new Int64Encoder();

export class Float32Encoder implements Encoder<number>, Decoder<number> {
  baseLength = () => 4;

  scratchLength(value: number): number { return 0; }

  encode(cursor: EncodeCursor, value: number) {
    cursor.buffer.setFloat32(cursor.base(4), value, true);
  }

  decode(cursor: DecodeCursor): number {
    return cursor.buffer.getFloat32(cursor.base(4), true);
  }
}

export const ProtoFloat32 = new Float32Encoder();

export class Float64Encoder implements Encoder<number>, Decoder<number> {
  baseLength = () => 8;

  scratchLength(value: number): number { return 0; }

  encode(cursor: EncodeCursor, value: number) {
    cursor.buffer.setFloat64(cursor.base(8), value, true);
  }

  decode(cursor: DecodeCursor): number {
    return cursor.buffer.getFloat64(cursor.base(8), true);
  }
}

export const ProtoFloat64 = new Float64Encoder();
