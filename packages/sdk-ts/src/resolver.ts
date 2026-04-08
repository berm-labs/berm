import {
  Connection,
  PublicKey,
  TransactionInstruction,
} from "@solana/web3.js";
import BN from "bn.js";
import { BERM_PROGRAM_ID, CoverType } from "./constants";
import { ClaimRecord, ClaimStatus, CoverPositionAccount } from "./types";
import { instructionDiscriminator, encodeU16, readU16, readU64 } from "./coding";
import { claimPda } from "./pda";
import {
  depegSeverity,
  exploitSeverity,
  oracleSeverity,
  payout,
} from "./severity";

/** Inputs describing an observed event for trigger evaluation. */
export interface TriggerInput {
  coverType: CoverType;
  /** TVL drop ratio for ExploitCover. */
  dropRatio?: number;
  /** Observed price for DepegCover. */
  price?: number;
  /** Oracle divergence fraction for OracleCover. */
  divergence?: number;
  /** Slashed/total stake for SlashingCover. */
  slashRatio?: number;
  /** Realized-loss/collateral for LiquidationCover. */
  liquidationRatio?: number;
  /** Notional exposure, base units. */
  notional: BN;
}

/** Outcome of a client-side trigger evaluation. */
export interface TriggerEvaluation {
  triggered: boolean;
  severity: number;
  estimatedPayout: BN;
}

/**
 * Reads claim records and evaluates whether a position's parametric trigger
 * condition is met. Settlement authority lives on-chain; this client mirrors
 * the predicate so callers can check eligibility before submitting.
 */
export class ClaimResolver {
  constructor(
    private readonly connection: Connection,
    private readonly programId: PublicKey = BERM_PROGRAM_ID
  ) {}

  /** Decode a raw claim record account buffer. */
  static decode(address: PublicKey, data: Buffer): ClaimRecord {
    let o = 8;
    const position = new PublicKey(data.subarray(o, o + 32));
    o += 32;
    const status = data.readUInt8(o) as ClaimStatus;
    o += 1;
    const severityBps = readU16(data, o);
    o += 2;
    const payoutAmount = readU64(data, o);
    o += 8;
    const triggerSlot = readU64(data, o);
    return {
      address,
      position,
      status,
      severityBps,
      payout: payoutAmount,
      triggerSlot,
    };
  }

  /** Fetch the claim record for a position, if one exists. */
  async fetchClaim(position: PublicKey): Promise<ClaimRecord | null> {
    const [claim] = claimPda(position, this.programId);
    const info = await this.connection.getAccountInfo(claim);
    if (!info) return null;
    return ClaimResolver.decode(claim, info.data);
  }

  /**
   * Evaluate whether a parametric trigger is met for a position and estimate
   * the resulting payout. Pure mirror of the on-chain predicate.
   */
  evaluate(
    position: CoverPositionAccount,
    input: TriggerInput
  ): TriggerEvaluation {
    let severity = 0;
    switch (input.coverType) {
      case CoverType.Exploit:
        severity = exploitSeverity(input.dropRatio ?? 0);
        break;
      case CoverType.Depeg:
        severity = depegSeverity(input.price ?? 1);
        break;
      case CoverType.Oracle:
        severity = oracleSeverity(input.divergence ?? 0);
        break;
      case CoverType.Slashing:
        severity = Math.max(0, Math.min(1, input.slashRatio ?? 0));
        break;
      case CoverType.Liquidation:
        severity = Math.max(0, Math.min(1, input.liquidationRatio ?? 0));
        break;
    }
    const estimatedPayout = payout(
      position.coverAmount,
      position.coverRatioBps,
      severity,
      input.notional
    );
    return {
      triggered: severity > 0,
      severity,
      estimatedPayout,
    };
  }

  /** Build an instruction to propose a parametric trigger for a position. */
  triggerClaimIx(params: {
    cranker: PublicKey;
    pool: PublicKey;
    position: PublicKey;
    severityBps: number;
  }): TransactionInstruction {
    const [claim] = claimPda(params.position, this.programId);
    const data = Buffer.concat([
      instructionDiscriminator("trigger_claim"),
      encodeU16(params.severityBps),
    ]);
    return new TransactionInstruction({
      programId: this.programId,
      keys: [
        { pubkey: params.cranker, isSigner: true, isWritable: true },
        { pubkey: params.pool, isSigner: false, isWritable: true },
        { pubkey: params.position, isSigner: false, isWritable: true },
        { pubkey: claim, isSigner: false, isWritable: true },
      ],
      data,
    });
  }
}
