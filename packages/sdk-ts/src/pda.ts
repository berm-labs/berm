import { PublicKey } from "@solana/web3.js";
import { BERM_PROGRAM_ID, SEEDS, CoverType } from "./constants";

/** Derive the cover pool PDA for a given cover type and covered asset. */
export function coverPoolPda(
  coverType: CoverType,
  coveredAsset: PublicKey,
  programId: PublicKey = BERM_PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from(SEEDS.coverPool),
      Buffer.from([coverType]),
      coveredAsset.toBuffer(),
    ],
    programId
  );
}

/** Derive the Token-2022 vault PDA that custodies a pool's capital. */
export function poolVaultPda(
  pool: PublicKey,
  programId: PublicKey = BERM_PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(SEEDS.poolVault), pool.toBuffer()],
    programId
  );
}

/** Derive a cover position PDA for an owner within a pool. */
export function coverPositionPda(
  pool: PublicKey,
  owner: PublicKey,
  index: number,
  programId: PublicKey = BERM_PROGRAM_ID
): [PublicKey, number] {
  const indexBuf = Buffer.alloc(4);
  indexBuf.writeUInt32LE(index);
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from(SEEDS.coverPosition),
      pool.toBuffer(),
      owner.toBuffer(),
      indexBuf,
    ],
    programId
  );
}

/** Derive the claim record PDA for a position. */
export function claimPda(
  position: PublicKey,
  programId: PublicKey = BERM_PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(SEEDS.claim), position.toBuffer()],
    programId
  );
}

/** Derive an underwriter account PDA. */
export function underwriterPda(
  authority: PublicKey,
  programId: PublicKey = BERM_PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(SEEDS.underwriter), authority.toBuffer()],
    programId
  );
}
