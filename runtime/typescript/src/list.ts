import { DecodeCursor, Decoder, EncodeCursor, Encoder } from './index';

export class ListEncoder<T> implements Encoder<T[]>, Decoder<T[]> {
  private itemEncoder: Encoder<T> & Decoder<T>;

  constructor(itemEncoder: Encoder<T> & Decoder<T>) {
    this.itemEncoder = itemEncoder;
  }

  baseLength = () => 8;

  scratchLength(value: T[]): number {
    var length = value.length * this.itemEncoder.baseLength();
    for (let item of value) {
      length += this.itemEncoder.scratchLength(item);
    }
    return length;
  }

  encode(cursor: EncodeCursor, value: T[]) {
    cursor.buffer.setUint32(cursor.base(4), value.length, true);

    cursor.innerInScratch(
      this.itemEncoder.baseLength() * value.length,
      (itemCursor: EncodeCursor) => {
        for (let item of value) {
          this.itemEncoder.encode(itemCursor, item);
        }
      }
    );
  }

  decode(cursor: DecodeCursor): T[] {
    let length = cursor.buffer.getUint32(cursor.base(4), true);

    let itemCursor = cursor.innerInScratch();

    var list: T[] = [];
    for (var i = 0; i < length; i++) {
      list.push(this.itemEncoder.decode(itemCursor));
    }

    return list;
  }
}

export const ProtoList = <T>(itemEncoder: Encoder<T> & Decoder<T>) => new ListEncoder(itemEncoder);

export class ListLazy<T> {
  private buffer: DataView;
  private offset: number;
  private length: number;
  private itemDecoder: Decoder<T>;

  constructor(buffer: DataView, offset: number, length: number, itemDecoder: Decoder<T>) {
    this.buffer = buffer;
    this.offset = offset;
    this.length = length;
    this.itemDecoder = itemDecoder;
  }

  public getItem(index: number): T {
    if (index > this.length) {
      throw Error("Index out of range in mproto.ListLazy");
    }
    let cursor = new DecodeCursor(this.buffer, this.offset + index * this.itemDecoder.baseLength());
    return this.itemDecoder.decode(cursor);
  }
}

export class ListLazyEncoder<T> implements Decoder<ListLazy<T>> {
  private itemEncoder: Decoder<T>;

  constructor(itemEncoder: Decoder<T>) {
    this.itemEncoder = itemEncoder;
  }

  baseLength = () => 8;

  decode(cursor: DecodeCursor): ListLazy<T> {
    let length = cursor.buffer.getUint32(cursor.base(4), true);
    let index = cursor.scratch();

    return new ListLazy(cursor.buffer, index, length, this.itemEncoder);
  }
}

export const ProtoListLazy = <T>(itemEncoder: Decoder<T>) => new ListLazyEncoder(itemEncoder);
