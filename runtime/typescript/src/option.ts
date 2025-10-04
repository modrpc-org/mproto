import { DecodeCursor, Decoder, EncodeCursor, Encoder } from './index';

export type Option<T> = T | null;

export class OptionEncoder<T> implements Encoder<Option<T>>, Decoder<Option<T>> {
  private someEncoder: Encoder<T> & Decoder<T>;

  constructor(
    someEncoder: Encoder<T> & Decoder<T>,
  ) {
    this.someEncoder = someEncoder;
  }

  baseLength = () => 1 + this.someEncoder.baseLength();

  scratchLength(value: Option<T>): number {
    if (value !== null) {
      return this.someEncoder.scratchLength(value);
    } else {
      return 0;
    }
  }

  encode(cursor: EncodeCursor, value: Option<T>) {
    if (value !== null) {
      cursor.buffer.setUint8(cursor.base(1), 1);
      this.someEncoder.encode(cursor, value);
    } else {
      cursor.buffer.setUint8(cursor.base(1), 0);
    }
  }

  decode(cursor: DecodeCursor): Option<T> {
    let variant = cursor.buffer.getUint8(cursor.base(1));

    if (variant == 0) {
      return null;
    } else if (variant == 1) {
      return this.someEncoder.decode(cursor);
    } else {
      throw "Failed to decode Option - invalid variant tag";
    }
  }
}

export class OptionLazyDecoder<T> implements Decoder<Option<T>> {
  private someDecoder: Decoder<T>;

  constructor(
    someDecoder: Decoder<T>,
  ) {
    this.someDecoder = someDecoder;
  }

  baseLength = () => 1 + this.someDecoder.baseLength();

  decode(cursor: DecodeCursor): Option<T> {
    let variant = cursor.buffer.getUint8(cursor.base(1));

    if (variant == 0) {
      return null;
    } else if (variant == 1) {
      return this.someDecoder.decode(cursor);
    } else {
      throw "Failed to decode Option - invalid variant tag";
    }
  }
}

export const ProtoOption = <T>(someEncoder: Encoder<T> & Decoder<T>) => new OptionEncoder(someEncoder);
export const ProtoOptionLazy = <T>(someDecoder: Decoder<T>) => new OptionLazyDecoder(someDecoder);
