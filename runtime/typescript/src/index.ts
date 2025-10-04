import { DecodeCursor } from './decode_cursor';
import { EncodeCursor } from './encode_cursor';

export * from './box';
export { DecodeCursor } from './decode_cursor';
export { EncodeCursor } from './encode_cursor';
export * from './list';
export * from './option';
export * from './primitives';
export * from './result';
export * from './string';

export interface Encoder<T> {
  baseLength(): number;
  scratchLength(value: T): number;
  encode(cursor: EncodeCursor, value: T): void;
}

export interface Decoder<T> {
  baseLength(): number;
  decode(cursor: DecodeCursor): T;
}

export interface EncoderDecoder<T> extends Encoder<T>, Decoder<T> { }

export function encodeValue<T>(encoder: Encoder<T>, value: T): ArrayBuffer {
  let buffer = new ArrayBuffer(encoder.baseLength() + encoder.scratchLength(value));
  let dataView = new DataView(buffer);
  let cursor = new EncodeCursor(dataView, encoder.baseLength())
  encoder.encode(cursor, value);
  return buffer;
}

export function decodeValue<T>(decoder: Decoder<T>, buffer: ArrayBuffer, offset: number = 0): T {
  let cursor = new DecodeCursor(new DataView(buffer, offset), 0);
  return decoder.decode(cursor);
}

