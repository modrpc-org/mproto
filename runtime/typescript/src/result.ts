import { DecodeCursor, Decoder, EncodeCursor, Encoder } from './index';

export namespace Result {
  export class Ok<Ok, Err> {
    ok: Ok;
    constructor(ok: Ok) {
      this.ok = ok;
    }
    toString(): string {
      const stringified = this.ok.toString();
      if (stringified == "[object Object]") {
        return `Ok(${JSON.stringify(this.ok)})`;
      } else {
        return `Ok(${stringified})`;
      }
    }
  }
  export class Err<Ok, Err> {
    err: Err;
    constructor(err: Err) {
      this.err = err;
    }
    toString(): string {
      const stringified = this.err.toString();
      if (stringified == "[object Object]") {
        return `Err(${JSON.stringify(this.err)})`;
      } else {
        return `Err(${stringified})`;
      }
    }
  }
}

export type Result<Ok, Err> = Result.Ok<Ok, Err> | Result.Err<Ok, Err>;

export class ResultEncoder<Ok, Err> implements Encoder<Result<Ok, Err>>, Decoder<Result<Ok, Err>> {
  private okEncoder: Encoder<Ok> & Decoder<Ok>;
  private errEncoder: Encoder<Err> & Decoder<Err>;

  constructor(
    okEncoder: Encoder<Ok> & Decoder<Ok>,
    errEncoder: Encoder<Err> & Decoder<Err>,
  ) {
    this.okEncoder = okEncoder;
    this.errEncoder = errEncoder;
  }

  baseLength = () => 1 + Math.max(this.okEncoder.baseLength(), this.errEncoder.baseLength());

  scratchLength(value: Result<Ok, Err>): number {
    if (value instanceof Result.Ok) {
      return this.okEncoder.scratchLength(value.ok);
    } else if (value instanceof Result.Err) {
      return this.errEncoder.scratchLength(value.err);
    } else {
      throw "Failed to encode Result - value is not a Result";
    }
  }

  encode(cursor: EncodeCursor, value: Result<Ok, Err>) {
    if (value instanceof Result.Ok) {
      cursor.buffer.setUint8(cursor.base(1), 0);
      this.okEncoder.encode(cursor, value.ok);
    } else if (value instanceof Result.Err) {
      cursor.buffer.setUint8(cursor.base(1), 1);
      this.errEncoder.encode(cursor, value.err);
    } else {
      throw "Failed to encode Result - value is not a Result";
    }
  }

  decode(cursor: DecodeCursor): Result<Ok, Err> {
    let variant = cursor.buffer.getUint8(cursor.base(1));

    if (variant == 0) {
      return new Result.Ok(this.okEncoder.decode(cursor));
    } else if (variant == 1) {
      return new Result.Err(this.errEncoder.decode(cursor));
    } else {
      throw "Failed to decode Result - invalid variant tag";
    }
  }
}

export class ResultLazyDecoder<Ok, Err> implements Decoder<Result<Ok, Err>> {
  private okDecoder: Decoder<Ok>;
  private errDecoder: Decoder<Err>;

  constructor(
    okDecoder: Decoder<Ok>,
    errDecoder: Decoder<Err>,
  ) {
    this.okDecoder = okDecoder;
    this.errDecoder = errDecoder;
  }

  baseLength = () => 1 + Math.max(this.okDecoder.baseLength(), this.errDecoder.baseLength());

  decode(cursor: DecodeCursor): Result<Ok, Err> {
    let variant = cursor.buffer.getUint8(cursor.base(1));

    if (variant == 0) {
      return new Result.Ok(this.okDecoder.decode(cursor));
    } else if (variant == 1) {
      return new Result.Err(this.errDecoder.decode(cursor));
    } else {
      throw "Failed to decode Result - invalid variant tag";
    }
  }
}

export const ProtoResult = <Ok, Err>(okEncoder: Encoder<Ok> & Decoder<Ok>, errEncoder: Encoder<Err> & Decoder<Err>) => new ResultEncoder(okEncoder, errEncoder);
export const ProtoResultLazy = <Ok, Err>(okDecoder: Decoder<Ok>, errDecoder: Decoder<Err>) => new ResultLazyDecoder(okDecoder, errDecoder);
