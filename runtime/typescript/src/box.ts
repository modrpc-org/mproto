import { DecodeCursor, Decoder, EncodeCursor, Encoder } from './index';

export class BoxEncoder<T> implements Encoder<T>, Decoder<T> {
  private innerEncoder: Encoder<T> & Decoder<T>;

  constructor(innerEncoder: Encoder<T> & Decoder<T>) {
    this.innerEncoder = innerEncoder;
  }

  baseLength = () => 4;

  scratchLength(value: T): number {
    return this.innerEncoder.baseLength() + this.innerEncoder.scratchLength(value);
  }

  encode(cursor: EncodeCursor, value: T) {
    cursor.innerInScratch(
      this.innerEncoder.baseLength(),
      (innerCursor: EncodeCursor) => {
        this.innerEncoder.encode(innerCursor, value);
      }
    );
  }

  decode(cursor: DecodeCursor): T {
    let innerCursor = cursor.innerInScratch();
    return this.innerEncoder.decode(innerCursor);
  }
}

export const ProtoBox = <T>(innerEncoder: Encoder<T> & Decoder<T>) => new BoxEncoder(innerEncoder);

export class BoxLazyDecoder<T> implements Decoder<T> {
  private innerDecoder: Decoder<T>;

  constructor(innerDecoder: Decoder<T>) {
    this.innerDecoder = innerDecoder;
  }

  baseLength = () => 4;

  decode(cursor: DecodeCursor): T {
    let innerCursor = cursor.innerInScratch();
    return this.innerDecoder.decode(innerCursor);
  }
}

export const ProtoBoxLazy = <T>(innerDecoder: Decoder<T>) => new BoxLazyDecoder(innerDecoder);
