import { createHash } from "crypto";
import BN from "bn.js";

/**
 * Compute the 8-byte Anchor instruction discriminator for a snake_case
 * instruction name: sha256("global:<name>")[0..8].
 */
export function instructionDiscriminator(name: string): Buffer {
  const hash = createHash("sha256").update(`global:${name}`).digest();
  return hash.subarray(0, 8);
}

/**
 * Compute the 8-byte Anchor account discriminator for an account struct:
 * sha256("account:<Name>")[0..8].
 */
export function accountDiscriminator(name: string): Buffer {
  const hash = createHash("sha256").update(`account:${name}`).digest();
  return hash.subarray(0, 8);
}

/** Encode a u64 as 8-byte little-endian. */
export function encodeU64(value: BN | number): Buffer {
  const bn = BN.isBN(value) ? value : new BN(value);
  return bn.toArrayLike(Buffer, "le", 8);
}

/** Encode a u32 as 4-byte little-endian. */
export function encodeU32(value: number): Buffer {
  const buf = Buffer.alloc(4);
  buf.writeUInt32LE(value);
  return buf;
}

/** Encode a u16 as 2-byte little-endian. */
export function encodeU16(value: number): Buffer {
  const buf = Buffer.alloc(2);
  buf.writeUInt16LE(value);
  return buf;
}

/** Encode a u8. */
export function encodeU8(value: number): Buffer {
  return Buffer.from([value & 0xff]);
}

/** Read a little-endian u64 from a buffer at offset, returning a BN. */
export function readU64(data: Buffer, offset: number): BN {
  return new BN(data.subarray(offset, offset + 8), "le");
}

/** Read a little-endian u32 from a buffer at offset. */
export function readU32(data: Buffer, offset: number): number {
  return data.readUInt32LE(offset);
}

/** Read a little-endian u16 from a buffer at offset. */
export function readU16(data: Buffer, offset: number): number {
  return data.readUInt16LE(offset);
}
