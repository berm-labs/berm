import {
  Connection,
  PublicKey,
  SystemProgram,
  TransactionInstruction,
} from "@solana/web3.js";
import BN from "bn.js";
import { BERM_PROGRAM_ID, CoverType, TOKEN_2022_PROGRAM_ID } from "./constants";
import { ClaimStatus, CoverPositionAccount } from "./types";
import {
  accountDiscriminator,
  instructionDiscriminator,
  encodeU8,
  encodeU16,
  encodeU32,
  encodeU64,
  readU16,
  readU64,
} from "./coding";
import { coverPositionPda, poolVaultPda, claimPda } from "./pda";

/**
 * Read and write user cover positions: buy cover, query holdings, and burn
 * (close) an expired or settled position.
 */
export class CoverPosition {
  constructor(
    private readonly connection: Connection,
    private readonly programId: PublicKey = BERM_PROGRAM_ID
  ) {}

  /** Decode a raw cover position account buffer. */
  static decode(address: PublicKey, data: Buffer): CoverPositionAccount {
    let o = 8;
    const owner = new PublicKey(data.subarray(o, o + 32));
    o += 32;
    const pool = new PublicKey(data.subarray(o, o + 32));
    o += 32;
    const coverType = data.readUInt8(o) as CoverType;
    o += 1;
    const coverAmount = readU64(data, o);
    o += 8;
    const premiumPaid = readU64(data, o);
    o += 8;
    const coverRatioBps = readU16(data, o);
    o += 2;
    const startSlot = readU64(data, o);
    o += 8;
    const expirySlot = readU64(data, o);
    o += 8;
    const claimStatus = data.readUInt8(o) as ClaimStatus;
    o += 1;
    const bump = data.readUInt8(o);
    return {
      address,
      owner,
      pool,
      coverType,
      coverAmount,
      premiumPaid,
      coverRatioBps,
      startSlot,
      expirySlot,
      claimStatus,
      bump,
    };
  }

  /** Fetch a single position by pool, owner, and index. */
  async fetch(
    pool: PublicKey,
    owner: PublicKey,
    index: number
  ): Promise<CoverPositionAccount | null> {
    const [position] = coverPositionPda(pool, owner, index, this.programId);
    const info = await this.connection.getAccountInfo(position);
    if (!info) return null;
    return CoverPosition.decode(position, info.data);
  }

  /** Fetch all positions owned by a wallet across every pool. */
  async fetchByOwner(owner: PublicKey): Promise<CoverPositionAccount[]> {
    const disc = accountDiscriminator("CoverPosition");
    const accounts = await this.connection.getProgramAccounts(this.programId, {
      filters: [
        { memcmp: { offset: 0, bytes: disc.toString("base64"), encoding: "base64" } },
        { memcmp: { offset: 8, bytes: owner.toBase58() } },
      ],
    });
    return accounts.map((a) => CoverPosition.decode(a.pubkey, a.account.data));
  }

  /** True when a position is still within its active window. */
  static isActive(position: CoverPositionAccount, currentSlot: BN): boolean {
    return (
      currentSlot.gte(position.startSlot) &&
      currentSlot.lt(position.expirySlot) &&
      position.claimStatus !== ClaimStatus.Settled
    );
  }

  /** Build an instruction to buy cover. */
  buyCoverIx(params: {
    buyer: PublicKey;
    pool: PublicKey;
    index: number;
    buyerTokenAccount: PublicKey;
    coverAmount: BN;
    coverRatioBps: number;
    durationSlots: BN;
    premium: BN;
  }): TransactionInstruction {
    const [position] = coverPositionPda(
      params.pool,
      params.buyer,
      params.index,
      this.programId
    );
    const [vault] = poolVaultPda(params.pool, this.programId);
    const data = Buffer.concat([
      instructionDiscriminator("buy_cover"),
      encodeU32(params.index),
      encodeU64(params.coverAmount),
      encodeU16(params.coverRatioBps),
      encodeU64(params.durationSlots),
      encodeU64(params.premium),
    ]);
    return new TransactionInstruction({
      programId: this.programId,
      keys: [
        { pubkey: params.buyer, isSigner: true, isWritable: true },
        { pubkey: params.pool, isSigner: false, isWritable: true },
        { pubkey: position, isSigner: false, isWritable: true },
        { pubkey: vault, isSigner: false, isWritable: true },
        { pubkey: params.buyerTokenAccount, isSigner: false, isWritable: true },
        { pubkey: TOKEN_2022_PROGRAM_ID, isSigner: false, isWritable: false },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      data,
    });
  }

  /** Build an instruction to burn (close) a position and reclaim rent. */
  burnPositionIx(params: {
    owner: PublicKey;
    pool: PublicKey;
    position: PublicKey;
  }): TransactionInstruction {
    const [claim] = claimPda(params.position, this.programId);
    const data = instructionDiscriminator("burn_position");
    return new TransactionInstruction({
      programId: this.programId,
      keys: [
        { pubkey: params.owner, isSigner: true, isWritable: true },
        { pubkey: params.pool, isSigner: false, isWritable: true },
        { pubkey: params.position, isSigner: false, isWritable: true },
        { pubkey: claim, isSigner: false, isWritable: false },
      ],
      data,
    });
  }
}
