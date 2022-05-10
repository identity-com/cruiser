import { Equals, Serialize } from "./borsh";

export type Bounds<T> = {
  min: T;
  max: T;
};

export class i8 implements Serialize, Equals<i8>, Equals<number> {
  static readonly bounds: Bounds<number> = {
    min: -128,
    max: 127,
  };
  constructor(public value: number) {
    if (value < i8.bounds.min || value > i8.bounds.max) {
      throw new Error(
        `${value} is not in range ${i8.bounds.min}..${i8.bounds.max}`
      );
    }
  }

  write(buffer: Buffer, offset: { offset: number }): void {
    buffer.writeInt8(this.value, offset.offset);
    offset.offset += 1;
  }
  serializedSize(): number {
    return i8.staticSize();
  }

  static read(buffer: Buffer, offset: { offset: number }): i8 {
    const value = buffer.readInt8(offset.offset);
    offset.offset += 1;
    return new i8(value);
  }

  static staticSize(): number {
    return 1;
  }

  equals(other: i8 | number): boolean {
    if (other instanceof i8) {
      return this.value === other.value;
    } else {
      return this.value === other;
    }
  }
}

export class i16 implements Serialize, Equals<i16>, Equals<number> {
  static readonly bounds: Bounds<number> = {
    min: -32768,
    max: 32767,
  };
  constructor(public value: number) {
    if (value < i16.bounds.min || value > i16.bounds.max) {
      throw new Error(
        `${value} is not in range ${i16.bounds.min}..${i16.bounds.max}`
      );
    }
  }

  write(buffer: Buffer, offset: { offset: number }): void {
    buffer.writeInt16LE(this.value, offset.offset);
    offset.offset += 2;
  }

  serializedSize(): number {
    return i16.staticSize();
  }

  static read(buffer: Buffer, offset: { offset: number }): i16 {
    const value = buffer.readInt16LE(offset.offset);
    offset.offset += 2;
    return new i16(value);
  }

  static staticSize(): number {
    return 2;
  }

  equals(other: i16 | number): boolean {
    if (other instanceof i16) {
      return this.value === other.value;
    } else {
      return this.value === other;
    }
  }
}

export class i32 implements Serialize, Equals<i32>, Equals<number> {
  static readonly bounds: Bounds<number> = {
    min: -2147483648,
    max: 2147483647,
  };
  constructor(public value: number) {
    if (value < i32.bounds.min || value > i32.bounds.max) {
      throw new Error(
        `${value} is not in range ${i32.bounds.min}..${i32.bounds.max}`
      );
    }
  }

  write(buffer: Buffer, offset: { offset: number }): void {
    buffer.writeInt32LE(this.value, offset.offset);
    offset.offset += 4;
  }
  serializedSize(): number {
    return i32.staticSize();
  }

  static read(buffer: Buffer, offset: { offset: number }): i32 {
    const value = buffer.readInt32LE(offset.offset);
    offset.offset += 4;
    return new i32(value);
  }

  static staticSize(): number {
    return 4;
  }

  equals(other: i32 | number): boolean {
    if (other instanceof i32) {
      return this.value === other.value;
    } else {
      return this.value === other;
    }
  }
}

export class i64 implements Serialize, Equals<i64>, Equals<bigint> {
  static readonly bounds: Bounds<bigint> = {
    min: -9223372036854775808n,
    max: 9223372036854775807n,
  };

  constructor(public value: bigint) {
    if (value < i64.bounds.min || value > i64.bounds.max) {
      throw new Error(
        `${value} is not in range ${i64.bounds.min}..${i64.bounds.max}`
      );
    }
  }

  write(buffer: Buffer, offset: { offset: number }): void {
    buffer.writeBigInt64LE(this.value, offset.offset);
    offset.offset += 8;
  }
  serializedSize(): number {
    return i64.staticSize();
  }

  static read(buffer: Buffer, offset: { offset: number }): i64 {
    const value = buffer.readBigInt64LE(offset.offset);
    offset.offset += 8;
    return new i64(value);
  }

  static staticSize(): number {
    return 8;
  }

  equals(other: i64 | bigint): boolean {
    if (other instanceof i64) {
      return this.value === other.value;
    } else {
      return this.value === other;
    }
  }
}

export class u8 implements Serialize, Equals<u8>, Equals<number> {
  static readonly bounds: Bounds<number> = {
    min: 0,
    max: 255,
  };
  constructor(public value: number) {
    if (value < u8.bounds.min || value > u8.bounds.max) {
      throw new Error(
        `${value} is not in range ${u8.bounds.min}..${u8.bounds.max}`
      );
    }
  }

  write(buffer: Buffer, offset: { offset: number }): void {
    buffer.writeUInt8(this.value, offset.offset);
    offset.offset += 1;
  }

  serializedSize(): number {
    return u8.staticSize();
  }

  static read(buffer: Buffer, offset: { offset: number }): u8 {
    const value = buffer.readUInt8(offset.offset);
    offset.offset += 1;
    return new u8(value);
  }

  static staticSize(): number {
    return 1;
  }

  equals(other: u8 | number): boolean {
    if (other instanceof u8) {
      return this.value === other.value;
    } else {
      return this.value === other;
    }
  }
}

export class u16 implements Serialize, Equals<u16>, Equals<number> {
  static readonly bounds: Bounds<number> = {
    min: 0,
    max: 65535,
  };
  constructor(public value: number) {
    if (value < u16.bounds.min || value > u16.bounds.max) {
      throw new Error(
        `${value} is not in range ${u16.bounds.min}..${u16.bounds.max}`
      );
    }
  }

  write(buffer: Buffer, offset: { offset: number }): void {
    buffer.writeUInt16LE(this.value, offset.offset);
    offset.offset += 1;
  }

  serializedSize(): number {
    return u16.staticSize();
  }

  static read(buffer: Buffer, offset: { offset: number }): u16 {
    const value = buffer.readUInt16LE(offset.offset);
    offset.offset += 1;
    return new u16(value);
  }

  static staticSize(): number {
    return 1;
  }

  equals(other: u16 | number): boolean {
    if (other instanceof u16) {
      return this.value === other.value;
    } else {
      return this.value === other;
    }
  }
}

export class u32 implements Serialize, Equals<u32>, Equals<number> {
  static readonly bounds: Bounds<number> = {
    min: 0,
    max: 4294967295,
  };
  constructor(public value: number) {
    if (value < u32.bounds.min || value > u32.bounds.max) {
      throw new Error(
        `${value} is not in range ${u32.bounds.min}..${u32.bounds.max}`
      );
    }
  }

  write(buffer: Buffer, offset: { offset: number }): void {
    buffer.writeUint32LE(this.value, offset.offset);
    offset.offset += 1;
  }

  serializedSize(): number {
    return u32.staticSize();
  }

  static read(buffer: Buffer, offset: { offset: number }): u32 {
    const value = buffer.readUint32LE(offset.offset);
    offset.offset += 1;
    return new u32(value);
  }

  static staticSize(): number {
    return 1;
  }

  equals(other: u32 | number): boolean {
    if (other instanceof u32) {
      return this.value === other.value;
    } else {
      return this.value === other;
    }
  }
}

export class u64 implements Serialize, Equals<u64>, Equals<bigint> {
  static readonly bounds: Bounds<bigint> = {
    min: 0n,
    max: 0xffff_ffff_ffff_ffffn,
  };
  constructor(public value: bigint) {
    if (value < u64.bounds.min || value > u64.bounds.max) {
      throw new Error(
        `${value} is not in range ${u64.bounds.min}..${u64.bounds.max}`
      );
    }
  }

  write(buffer: Buffer, offset: { offset: number }): void {
    buffer.writeBigUint64LE(this.value, offset.offset);
    offset.offset += 1;
  }

  serializedSize(): number {
    return u64.staticSize();
  }

  static read(buffer: Buffer, offset: { offset: number }): u64 {
    const value = buffer.readBigUInt64LE(offset.offset);
    offset.offset += 1;
    return new u64(value);
  }

  static staticSize(): number {
    return 1;
  }

  equals(other: u64 | bigint): boolean {
    if (other instanceof u64) {
      return this.value === other.value;
    } else {
      return this.value === other;
    }
  }
}
