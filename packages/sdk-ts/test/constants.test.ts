import { describe, it, expect } from "vitest";
import {
  BERM_PROGRAM_ID,
  POOL_VAULT_PROGRAM_ID,
  CLAIM_RESOLVER_PROGRAM_ID,
  DEFAULT_RPC_ENDPOINT,
  DEVNET_RPC_ENDPOINT,
} from "../src/constants";

describe("program identities", () => {
  it("targets the deployed devnet cover executor", () => {
    expect(BERM_PROGRAM_ID.toBase58()).toBe(
      "AMenBCW8sgtx2VriEYzdJkTCsUBF6FGQy8PhcNh9p7pH"
    );
  });

  it("targets the deployed devnet pool vault program", () => {
    expect(POOL_VAULT_PROGRAM_ID.toBase58()).toBe(
      "H4ifx5HYeHHvEuyJMdF1EpRSeNZJqRf3Vkhi4LT8N12T"
    );
  });

  it("targets the deployed devnet claim resolver program", () => {
    expect(CLAIM_RESOLVER_PROGRAM_ID.toBase58()).toBe(
      "GnS9Sii7PpELXQLyKwZRgrEpqma3GQwcSxtqNdCMmkk3"
    );
  });

  it("does not regress to the deploy authority pubkey", () => {
    const deployAuthority = "8swee4bbyHY6fF4frktzBkj2MSfcAUieMEz97qAPf9iq";
    expect(BERM_PROGRAM_ID.toBase58()).not.toBe(deployAuthority);
  });

  it("defaults to the devnet RPC endpoint", () => {
    expect(DEFAULT_RPC_ENDPOINT).toBe(DEVNET_RPC_ENDPOINT);
  });
});
