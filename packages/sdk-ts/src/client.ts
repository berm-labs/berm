import { Connection, PublicKey } from "@solana/web3.js";
import {
  BERM_PROGRAM_ID,
  DEFAULT_RPC_ENDPOINT,
  PYTH_HERMES_ENDPOINT,
} from "./constants";
import { CoverPool } from "./pool";
import { CoverPosition } from "./position";
import { ClaimResolver } from "./resolver";
import { RiskScorer } from "./risk";
import { OracleAdapter } from "./oracle";

/** Configuration for {@link BermClient}. */
export interface BermClientConfig {
  /** RPC endpoint; defaults to the public Solana mainnet RPC. */
  endpoint?: string;
  /** Existing connection to reuse instead of creating one. */
  connection?: Connection;
  /** Program id override; defaults to {@link BERM_PROGRAM_ID}. */
  programId?: PublicKey;
  /** Pyth Hermes endpoint override. */
  hermesEndpoint?: string;
}

/**
 * Top-level entry point to the BERM SDK. Holds the RPC connection and exposes
 * the cover pool, position, claim resolver, risk scorer, and oracle clients.
 *
 * @example
 * ```ts
 * const berm = new BermClient({ endpoint: "https://api.devnet.solana.com" });
 * const pools = await berm.pools.fetchAll();
 * ```
 */
export class BermClient {
  readonly connection: Connection;
  readonly programId: PublicKey;
  readonly pools: CoverPool;
  readonly positions: CoverPosition;
  readonly claims: ClaimResolver;
  readonly risk: RiskScorer;
  readonly oracle: OracleAdapter;

  constructor(config: BermClientConfig = {}) {
    this.connection =
      config.connection ??
      new Connection(config.endpoint ?? DEFAULT_RPC_ENDPOINT, "confirmed");
    this.programId = config.programId ?? BERM_PROGRAM_ID;
    this.pools = new CoverPool(this.connection, this.programId);
    this.positions = new CoverPosition(this.connection, this.programId);
    this.claims = new ClaimResolver(this.connection, this.programId);
    this.risk = new RiskScorer();
    this.oracle = new OracleAdapter(
      config.hermesEndpoint ?? PYTH_HERMES_ENDPOINT
    );
  }

  /** Current confirmed slot, useful for position activity checks. */
  async currentSlot(): Promise<number> {
    return this.connection.getSlot("confirmed");
  }
}
