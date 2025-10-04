import { DecodeCursor, Decoder, EncodeCursor, Encoder } from './index';

const textEncoder = new TextEncoder();
const textDecoder = new TextDecoder();

function stringLengthInBytes(s: string) {
  var lengthInBytes = 0;
  for (var i = 0; i < s.length; i++) {
    let c = s.charCodeAt(i);
    lengthInBytes +=
      c < (1 <<  7) ? 1 :
      c < (1 << 11) ? 2 :
      c < (1 << 16) ? 3 :
      c < (1 << 21) ? 4 :
      c < (1 << 26) ? 5 :
      c < (1 << 31) ? 6 :
      Number.NaN;
  }
  return lengthInBytes;
}

export class StringEncoder implements Encoder<string>, Decoder<string> {
  baseLength = () => 8;

  scratchLength(value: string): number { return stringLengthInBytes(value); }

  encode(cursor: EncodeCursor, value: string) {
    let strLength = stringLengthInBytes(value);
    cursor.buffer.setUint32(cursor.base(4), strLength, true);
    let strScratchIndex = cursor.scratch(strLength);
    textEncoder.encodeInto(
      value,
      new Uint8Array(cursor.buffer.buffer, cursor.buffer.byteOffset + strScratchIndex, strLength)
    );
  }

  decode(cursor: DecodeCursor): string {
    let length = cursor.buffer.getUint32(cursor.base(4), true);
    let index = cursor.scratch();
    return textDecoder.decode(
      new Uint8Array(cursor.buffer.buffer, cursor.buffer.byteOffset + index, length)
    );
  }
}

export const ProtoString = new StringEncoder();
