export class EncodeCursor {
  buffer: DataView;
  baseOffset: number;
  scratchOffset: number;

  constructor(buffer: DataView, baseLength: number = 0) {
    this.buffer = buffer;
    this.baseOffset = 0;
    this.scratchOffset = baseLength;
  }

  public base(size: number): number {
    let index = this.baseOffset;
    this.baseOffset += size;
    return index;
  }

  public scratch(size: number): number {
    let index = this.scratchOffset;
    this.scratchOffset += size;

    this.buffer.setUint32(this.base(4), index, true);

    return index;
  }

  public innerInScratch(baseLength: number, f: (cursor: EncodeCursor) => void) {
    let innerBaseOffset = this.scratch(baseLength);

    let inner = new EncodeCursor(this.buffer);
    inner.baseOffset = innerBaseOffset;
    inner.scratchOffset = this.scratchOffset;

    f(inner)

    this.scratchOffset = inner.scratchOffset;
  }
}
