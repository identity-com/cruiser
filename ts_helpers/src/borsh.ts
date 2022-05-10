import { PublicKey } from "@solana/web3.js";

export type NonFunctionPropertyNames<T> = {
  // eslint-disable-next-line @typescript-eslint/ban-types
  [K in keyof T]: T[K] extends Function ? never : K;
}[keyof T];

export interface Deserialize<T> {
  read(buffer: Buffer, offset: { offset: number }): T;
}
export interface Serialize {
  write(buffer: Buffer, offset: { offset: number }): void;
  serializedSize(): number;
}

export interface StaticSize {
  staticSize(): number;
}

export interface Equals<T> {
  equals(other: T): boolean;
}

export interface Account<D, T> {
  discriminant(): D;
}
export interface Instruction<D extends Serialize, T extends Serialize> {
  discriminant(): D;
}

export class SerializablePublicKey
  implements Serialize, Equals<SerializablePublicKey>, Equals<PublicKey>
{
  constructor(public key: PublicKey) {}

  write(buffer: Buffer, offset: { offset: number }): void {
    this.key.toBuffer().copy(buffer, offset.offset);
    offset.offset += 32;
  }
  serializedSize(): number {
    return 32;
  }

  static read(
    buffer: Buffer,
    offset: { offset: number }
  ): SerializablePublicKey {
    const key = new PublicKey(buffer.slice(offset.offset, offset.offset + 32));
    offset.offset += 32;
    return new SerializablePublicKey(key);
  }

  static staticSize(): number {
    return 32;
  }

  equals(other: SerializablePublicKey | PublicKey): boolean {
    if (other instanceof SerializablePublicKey) {
      return this.key.equals(other.key);
    } else {
      return this.key.equals(other);
    }
  }
}

export type Cons<T> = new (...args: any[]) => T;
export function readAccount<D extends Equals<D>, T>(
  dCons: Cons<D> & Deserialize<D>,
  tCons: Cons<T> & Deserialize<T> & Account<D, T>,
  buffer: Buffer,
  offset: { offset: number } = { offset: 0 }
): T | null {
  const discriminant = dCons.read(buffer, offset);
  if (discriminant.equals(tCons.discriminant())) {
    return tCons.read(buffer, offset);
  } else {
    return null;
  }
}

export function serializeInstruction<D extends Serialize, T extends Serialize>(
  tCons: Cons<T> & Instruction<D, T>,
  instruction: T
): Buffer {
  const discriminant = tCons.discriminant();
  const buffer = Buffer.alloc(
    discriminant.serializedSize() + instruction.serializedSize()
  );
  const offset = { offset: 0 };
  discriminant.write(buffer, offset);
  instruction.write(buffer, offset);
  return buffer;
}
