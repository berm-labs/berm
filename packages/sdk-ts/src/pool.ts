import {
  Connection,
  PublicKey,
  SystemProgram,
  TransactionInstruction,
} from "@solana/web3.js";
import BN from "bn.js";
import { BERM_PROGRAM_ID, CoverType, TOKEN_2022_PROGRAM_ID } from "./constants";
import { CoverPoolAccount } from "./types";
import {
  accountDiscriminator,
  instructionDiscriminator,
  encodeU8,
  encodeU16,
  encodeU64,
  readU16,
  readU32,
  readU64,
} from "./coding";
import { coverPoolPda, poolVaultPda } from "./pda";

/**
 * Read and write cover pools. A pool is created per cover type and covered
 * asset; liquidity providers underwrite it and earn premium.
 */
export class CoverPool {
  constructor(
    private readonly connection: Connection,
    private readonly programId: PublicKey = BERM_PROGRAM_ID
  ) {}

  /** Decode a raw cover pool account buffer. */
  static decode(address: PublicKey, data: Buffer): CoverPoolAccount {
    let o = 8; // skip discriminator
    const coverType = data.readUInt8(o) as CoverType;
    o += 1;
    const coveredAsset = new PublicKey(data.subarray(o, o + 32));
    o += 32;
    const totalCapital = readU64(data, o);
    o += 8;
    const totalCoverOutstanding = readU64(data, o);
    o += 8;
    const premiumAccrued = readU64(data, o);
    o += 8;
    const activePositions = readU32(data, o);
    o += 4;
    const thresholdBps = readU16(data, o);
    o += 2;
    const windowSlots = readU16(data, o);
    o += 2;
    const bump = data.readUInt8(o);
    return {
      address,
      coverType,
      coveredAsset,
      totalCapital,
      totalCoverOutstanding,
      premiumAccrued,
      activePositions,
      thresholdBps,
      windowSlots,
      bump,
    };
  }

  /** Fetch a single cover pool by cover type and covered asset. */
  async fetch(
    coverType: CoverType,
    coveredAsset: PublicKey
  ): Promise<CoverPoolAccount | null> {
    const [pool] = coverPoolPda(coverType, coveredAsset, this.programId);
    const info = await this.connection.getAccountInfo(pool);
    if (!info) return null;
    return CoverPool.decode(pool, info.data);
  }

  /** Fetch every cover pool owned by the program. */
  async fetchAll(): Promise<CoverPoolAccount[]> {
    const disc = accountDiscriminator("CoverPool");
    const accounts = await this.connection.getProgramAccounts(this.programId, {
      filters: [{ memcmp: { offset: 0, bytes: disc.toString("base64"), encoding: "base64" } }],
    });
    return accounts.map((a) => CoverPool.decode(a.pubkey, a.account.data));
  }

  /** Pool utilization: outstanding cover over underwritten capital. */
  static utilization(pool: CoverPoolAccount): number {
    if (pool.totalCapital.isZero()) return 0;
    return (
      pool.totalCoverOutstanding.mul(new BN(10000)).div(pool.totalCapital).toNumber() /
      10000
    );
  }

  /** Build an instruction to initialize a cover pool. */
  createPoolIx(params: {
    authority: PublicKey;
    coverType: CoverType;
    coveredAsset: PublicKey;
    capitalMint: PublicKey;
    thresholdBps: number;
    windowSlots: number;
  }): TransactionInstruction {
    const [pool] = coverPoolPda(
      params.coverType,
      params.coveredAsset,
      this.programId
    );
    const [vault] = poolVaultPda(pool, this.programId);
    const data = Buffer.concat([
      instructionDiscriminator("create_cover_pool"),
      encodeU8(params.coverType),
      params.coveredAsset.toBuffer(),
      encodeU16(params.thresholdBps),
      encodeU16(params.windowSlots),
    ]);
    return new TransactionInstruction({
      programId: this.programId,
      keys: [
        { pubkey: params.authority, isSigner: true, isWritable: true },
        { pubkey: pool, isSigner: false, isWritable: true },
        { pubkey: vault, isSigner: false, isWritable: true },
        { pubkey: params.capitalMint, isSigner: false, isWritable: false },
        { pubkey: TOKEN_2022_PROGRAM_ID, isSigner: false, isWritable: false },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      data,
    });
  }

  /** Build an instruction to underwrite (provide liquidity to) a pool. */
  provideLiquidityIx(params: {
    provider: PublicKey;
    pool: PublicKey;
    providerTokenAccount: PublicKey;
    amount: BN;
  }): TransactionInstruction {
    const [vault] = poolVaultPda(params.pool, this.programId);
    const data = Buffer.concat([
      instructionDiscriminator("provide_liquidity"),
      encodeU64(params.amount),
    ]);
    return new TransactionInstruction({
      programId: this.programId,
      keys: [
        { pubkey: params.provider, isSigner: true, isWritable: true },
        { pubkey: params.pool, isSigner: false, isWritable: true },
        { pubkey: vault, isSigner: false, isWritable: true },
        { pubkey: params.providerTokenAccount, isSigner: false, isWritable: true },
        { pubkey: TOKEN_2022_PROGRAM_ID, isSigner: false, isWritable: false },
      ],
      data,
    });
  }
}
