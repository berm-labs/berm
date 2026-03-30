//! On-chain oracle reads (Pyth + Switchboard On-Demand).
//!
//! Price-based cover (Depeg, Oracle) needs a trustworthy price at purchase and
//! settlement time. This module reads both a Pyth price account and a
//! Switchboard On-Demand pull feed on chain, normalises each to a common
//! exponent, and exposes a divergence check -- the dual-oracle guard that backs
//! `OracleCover`. The Pyth account is parsed against the public v2 layout (the
//! `pyth-sdk-solana` crate is intentionally not linked here because it conflicts
//! with `switchboard-on-demand` on `borsh`); Switchboard is read through the
//! first-party `switchboard-on-demand` crate.

use anchor_lang::prelude::*;
use switchboard_on_demand::PullFeedAccountData;

use crate::error::CoverError;

/// Pyth v2 price-account field offsets (mirrors `berm_oracle_adapter::pyth`).
const PYTH_MAGIC: u32 = 0xa1b2_c3d4;
const EXPO_OFFSET: usize = 20;
const AGG_PRICE_OFFSET: usize = 208;
const AGG_CONF_OFFSET: usize = 216;

/// A price normalised to mantissa + base-10 exponent.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct OraclePrice {
    /// Signed price mantissa.
    pub price: i64,
    /// One-sigma confidence mantissa.
    pub conf: u64,
    /// Base-10 exponent.
    pub expo: i32,
}

impl OraclePrice {
    /// Rescale to a target exponent (used to compare two sources).
    pub fn rescale(&self, target_expo: i32) -> Option<OraclePrice> {
        if self.expo == target_expo {
            return Some(*self);
        }
        let diff = self.expo - target_expo;
        if diff > 0 {
            let f = 10i64.checked_pow(diff as u32)?;
            Some(OraclePrice {
                price: self.price.checked_mul(f)?,
                conf: self.conf.checked_mul(f as u64)?,
                expo: target_expo,
            })
        } else {
            let f = 10i64.checked_pow((-diff) as u32)?;
            Some(OraclePrice {
                price: self.price / f,
                conf: self.conf / f as u64,
                expo: target_expo,
            })
        }
    }
}

/// Read and validate a Pyth v2 price account.
pub fn read_pyth(account: &AccountInfo) -> Result<OraclePrice> {
    let data = account.try_borrow_data()?;
    require!(data.len() >= AGG_CONF_OFFSET + 8, CoverError::OracleParse);
    let magic = u32::from_le_bytes(slice4(&data, 0));
    require!(magic == PYTH_MAGIC, CoverError::OracleParse);
    let expo = i32::from_le_bytes(slice4(&data, EXPO_OFFSET));
    let price = i64::from_le_bytes(slice8(&data, AGG_PRICE_OFFSET));
    let conf = u64::from_le_bytes(slice8(&data, AGG_CONF_OFFSET));
    require!(price != 0, CoverError::OracleParse);
    Ok(OraclePrice { price, conf, expo })
}

/// Read a Switchboard On-Demand pull feed, validating staleness against `clock_slot`.
pub fn read_switchboard(account: &AccountInfo, clock_slot: u64) -> Result<OraclePrice> {
    let data = account.try_borrow_data()?;
    let feed = PullFeedAccountData::parse(data).map_err(|_| error!(CoverError::OracleParse))?;
    let value = feed
        .value(clock_slot)
        .map_err(|_| error!(CoverError::StaleOracle))?;
    let mantissa = value.mantissa();
    let scale = value.scale();
    let price = i64::try_from(mantissa).map_err(|_| error!(CoverError::OracleParse))?;
    Ok(OraclePrice {
        price,
        conf: 0,
        expo: -(scale as i32),
    })
}

/// Cross-source divergence in basis points at a common exponent.
pub fn divergence_bps(a: &OraclePrice, b: &OraclePrice, target_expo: i32) -> Result<u64> {
    let a = a.rescale(target_expo).ok_or(error!(CoverError::Overflow))?;
    let b = b.rescale(target_expo).ok_or(error!(CoverError::Overflow))?;
    let diff = (a.price as i128 - b.price as i128).unsigned_abs();
    let denom = (a.price.unsigned_abs().min(b.price.unsigned_abs())).max(1) as u128;
    Ok(((diff * 10_000) / denom) as u64)
}

/// Whether the price is outside the inclusive `[lower, upper]` peg band at `expo`.
pub fn is_out_of_band(price: &OraclePrice, lower: i64, upper: i64, expo: i32) -> Result<bool> {
    let p = price.rescale(expo).ok_or(error!(CoverError::Overflow))?;
    Ok(p.price < lower || p.price > upper)
}

fn slice4(data: &[u8], off: usize) -> [u8; 4] {
    let mut b = [0u8; 4];
    b.copy_from_slice(&data[off..off + 4]);
    b
}

fn slice8(data: &[u8], off: usize) -> [u8; 8] {
    let mut b = [0u8; 8];
    b.copy_from_slice(&data[off..off + 8]);
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rescale_widens_decimals() {
        let p = OraclePrice {
            price: 100,
            conf: 5,
            expo: -2,
        };
        let r = p.rescale(-4).unwrap();
        assert_eq!(r.price, 10_000);
        assert_eq!(r.conf, 500);
    }

    #[test]
    fn divergence_is_relative() {
        let a = OraclePrice {
            price: 100_000_000,
            conf: 0,
            expo: -8,
        };
        let b = OraclePrice {
            price: 101_000_000,
            conf: 0,
            expo: -8,
        };
        assert_eq!(divergence_bps(&a, &b, -8).unwrap(), 100); // 1%
    }

    #[test]
    fn band_check() {
        let p = OraclePrice {
            price: 90_000_000,
            conf: 0,
            expo: -8,
        };
        assert!(is_out_of_band(&p, 95_000_000, 105_000_000, -8).unwrap());
        let q = OraclePrice {
            price: 100_000_000,
            conf: 0,
            expo: -8,
        };
        assert!(!is_out_of_band(&q, 95_000_000, 105_000_000, -8).unwrap());
    }
}
