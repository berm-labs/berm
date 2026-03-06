//! `berm-keeper` -- the off-chain parametric trigger daemon.
//!
//! In production this binary opens an RPC subscription, builds a
//! [`MarketSnapshot`] every slot from oracle and protocol telemetry, runs the
//! [`Keeper`] over the policy book, and submits triggered settlements to the
//! `claim-resolver` program. To keep the binary deterministic and dependency-free
//! it ships with a `--demo` mode that replays a recorded snapshot (the USDC 2023
//! depeg) so operators can validate the pipeline end-to-end without live RPC.
//!
//! Network wiring (RPC endpoint, signer) is supplied through environment
//! variables read at startup; see `--help`.

use std::env;
use std::process::ExitCode;

use berm_cover_engine::cover_type::{CoverParams, DepegParams};
use berm_cover_engine::policy::{Policy, PolicyStatus};
use berm_cover_engine::trigger::{MarketSnapshot, PegReading};
use berm_cover_engine::Keeper;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return ExitCode::SUCCESS;
    }

    let capital = env::var("BERM_POOL_CAPITAL_CENTS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(100_000_000); // $1,000,000.00 in cents

    if args.iter().any(|a| a == "--demo") {
        return run_demo(capital);
    }

    eprintln!(
        "berm-keeper: live mode requires BERM_RPC_URL and a signer; \
         run with --demo to validate the trigger pipeline offline."
    );
    match env::var("BERM_RPC_URL") {
        Ok(url) if !url.is_empty() => {
            println!("berm-keeper: would attach to RPC {url} (live loop not run in this build)");
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("error: BERM_RPC_URL is not set");
            ExitCode::from(2)
        }
    }
}

fn print_help() {
    println!(
        "berm-keeper -- parametric trigger daemon\n\n\
         USAGE:\n  berm-keeper [--demo]\n\n\
         ENV:\n  BERM_RPC_URL             Solana RPC endpoint (live mode)\n  \
         BERM_POOL_CAPITAL_CENTS  Free pool capital in USD cents (default 1,000,000.00)\n\n\
         FLAGS:\n  --demo   Replay the USDC 2023 depeg snapshot through the engine\n  \
         --help   Show this message"
    );
}

/// Replay the USDC March-2023 depeg (price reached ~0.88) through the engine and
/// print the settlement the protocol would have produced.
fn run_demo(capital: u64) -> ExitCode {
    let keeper = Keeper::new(capital);

    let policies = vec![Policy {
        id: 1,
        holder: "DemoHolder1111111111111111111111111111111".into(),
        subject: "USDC/USD".into(),
        coverage: 100_000_000, // $1,000,000.00 cover in cents
        params: CoverParams::Depeg(DepegParams::default()),
        start_slot: 0,
        end_slot: u64::MAX,
        status: PolicyStatus::Active,
    }];

    let mut snap = MarketSnapshot::at(184_900_000);
    snap.peg.insert(
        "USDC/USD".into(),
        PegReading {
            price: 88_000_000, // 0.88 at expo -8
            expo: -8,
            out_of_band_slots: 5_000,
        },
    );

    let tick = keeper.tick(&policies, &snap);
    println!("{}", Keeper::tick_to_json(&tick));
    if tick.outcomes.is_empty() {
        eprintln!("demo: no trigger fired (unexpected)");
        return ExitCode::from(1);
    }
    println!(
        "demo: DepegCover auto-triggered, settling {} cents to {} holder(s)",
        tick.settlement.total_net,
        tick.settlement.len()
    );
    ExitCode::SUCCESS
}
