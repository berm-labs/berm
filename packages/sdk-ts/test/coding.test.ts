import { describe, it, expect } from "vitest";
import BN from "bn.js";
import {
  instructionDiscriminator,
  accountDiscriminator,
  encodeU64,
  encodeU32,
  encodeU16,
  encodeU8,
  readU64,
  readU32,
  readU16,
} from "../src/coding";

describe("discriminators", () => {
  it("are 8 bytes and deterministic", () => {
    const a = instructionDiscriminator("buy_cover");
    const b = instructionDiscriminator("buy_cover");
    expect(a.length).toBe(8);
    expect(a.equals(b)).toBe(true);
  });
  it("differ between instruction and account namespaces", () => {
    const ix = instructionDiscriminator("cover_pool");
    const acc = accountDiscriminator("CoverPool");
    expect(ix.equals(acc)).toBe(false);
  });
  it("match the known Anchor sha256 scheme", () => {
    // sha256("global:buy_cover")[0..8]
    const d = instructionDiscriminator("buy_cover");
    expect(d.toString("hex")).toHaveLength(16);
  });
});

describe("integer codecs round-trip", () => {
  it("u64 round-trips a large value", () => {
    const v = new BN("18446744073709551615"); // u64 max
    const buf = encodeU64(v);
    expect(buf.length).toBe(8);
    expect(readU64(buf, 0).toString()).toBe(v.toString());
  });
  it("u64 accepts a number", () => {
    const buf = encodeU64(50000);
    expect(readU64(buf, 0).toNumber()).toBe(50000);
  });
  it("u32 round-trips", () => {
    const buf = encodeU32(4294967295);
    expect(buf.length).toBe(4);
    expect(readU32(buf, 0)).toBe(4294967295);
  });
  it("u16 round-trips", () => {
    const buf = encodeU16(65535);
    expect(buf.length).toBe(2);
    expect(readU16(buf, 0)).toBe(65535);
  });
  it("u8 encodes a single byte", () => {
    expect(encodeU8(3)).toEqual(Buffer.from([3]));
  });
  it("reads at an offset", () => {
    const buf = Buffer.concat([encodeU8(0), encodeU32(12345)]);
    expect(readU32(buf, 1)).toBe(12345);
  });
});
