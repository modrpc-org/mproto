export class DecodeCursor {
  public buffer: DataView;
  public baseOffset: number;

  constructor(buffer: DataView, baseOffset: number = 0) {
    this.buffer = buffer;
    this.baseOffset = baseOffset;
  }

  public base(size: number): number {
    let index = this.baseOffset;
    this.baseOffset += size;
    return index;
  }

  public scratch(): number {
    return this.buffer.getUint32(this.base(4), true);
  }

  public innerInScratch(): DecodeCursor {
    let innerBaseOffset = this.scratch();

    let inner = new DecodeCursor(this.buffer, innerBaseOffset);
    return inner;
  }
}

