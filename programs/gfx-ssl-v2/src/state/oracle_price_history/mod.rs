pub mod historical_price;
pub mod oracle_type;

use crate::PDAIdentifier;
use anchor_lang::{prelude::*, Discriminator};
use bytemuck::Zeroable;
#[cfg(feature = "no-entrypoint")]
use std::fmt::{Display, Formatter};
use std::io::Write;

use crate::SSLV2Error;
pub use historical_price::*;
pub use oracle_type::*;

/// Max capacity of the oracle's price history.
pub const NUM_HISTORICAL_PRICE_ENTRIES: usize = 256;

/// Scoped to a particular admin, mint, and oracle,
/// this account records the continually updated USD-price history
/// of that mint in a rotating array.
///
/// This account should be cranked every slot.
#[account(zero_copy)]
#[derive(Debug)]
#[repr(C)]
pub struct OraclePriceHistory {
    /// Whether the oracle is a Pyth oracle, Switchboardv2, etc.
    /// For [Pod] safety, stored directly as a [u8], and converted to/from an [OracleType]
    /// using `self.oracle_type()`.
    pub oracle_type: u8,
    /// Used if the oracle needs to be throttled so that the price updates aren't too close to each other.
    pub minimum_elapsed_slots: u8,
    /// Used to configure how many slots can pass before a price is considered stale
    pub max_slot_price_staleness: u8,
    pub _pad0: [u8; 5],
    /// The pool registry pubkey.
    pub pool_registry: Pubkey,
    /// The oracle pubkey itself.
    pub oracle_address: Pubkey,
    /// The mint that is being tracked.
    pub mint: Pubkey,
    /// Total number of updates that have executed.
    /// This is used to keep track of where the most recently updated value is.
    pub num_updates: u64,
    _space: [u8; 128],
    /// Historical record of price values.
    pub price_history: [HistoricalPrice; NUM_HISTORICAL_PRICE_ENTRIES],
}

impl Default for OraclePriceHistory {
    fn default() -> Self {
        Self::zeroed()
    }
}

#[cfg(feature = "no-entrypoint")]
impl Display for OraclePriceHistory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Pool Registry: {}", self.pool_registry)?;
        writeln!(f, "Mint: {}", self.mint)?;
        writeln!(f, "Oracle Address: {}", self.oracle_address)?;
        writeln!(f, "Oracle type: {}", OracleType::from(self.oracle_type))?;
        writeln!(f, "Minimum Elapsed Slots: {}", self.minimum_elapsed_slots)?;
        writeln!(
            f,
            "Maximum Price Slot Staleness: {}",
            self.max_slot_price_staleness
        )?;

        let price_iter = AccountHistoryIterator::from(self);
        for (idx, historical_price) in price_iter.enumerate() {
            if *historical_price != HistoricalPrice::default() {
                writeln!(f, "Historical Price {}", idx)?;
                writeln!(f, "{}", historical_price)?;
            }
        }
        Ok(())
    }
}

/// Compile-time size check.
const _: [u8; 6384] = [0u8; std::mem::size_of::<OraclePriceHistory>()];

impl PDAIdentifier for OraclePriceHistory {
    const IDENT: &'static [u8] = b"oracle_price_history";

    #[inline(always)]
    fn program_id() -> &'static Pubkey {
        &crate::ID
    }
}

impl OraclePriceHistory {
    pub fn default_pyth() -> Self {
        let mut act = Self::default();
        act.oracle_type = OracleType::Pyth.into();
        act
    }

    pub fn default_switchboard_v2() -> Self {
        let mut act = Self::default();
        act.oracle_type = OracleType::Switchboardv2.into();
        act
    }

    /// Create an instance with mock data.
    /// For unit testing.
    #[cfg(feature = "no-entrypoint")]
    pub fn from_test_data(test_data: Vec<i64>) -> Self {
        let mut price_history = Self {
            oracle_type: OracleType::Pyth.into(),
            minimum_elapsed_slots: 0,
            max_slot_price_staleness: u8::MAX,
            _pad0: [0; 5],
            pool_registry: Default::default(),
            oracle_address: Default::default(),
            mint: Default::default(),
            num_updates: NUM_HISTORICAL_PRICE_ENTRIES as u64,
            _space: [0; 128],
            price_history: [Default::default(); NUM_HISTORICAL_PRICE_ENTRIES],
        };
        test_data.into_iter().enumerate().for_each(|(slot, num)| {
            price_history.push(HistoricalPrice {
                price: HistoricalDecimal {
                    num,
                    scale: 6,
                    _pad0: [0; 4],
                },
                slot: slot as u64,
            })
        });
        price_history
    }

    /// Calculate an address based on admin and oracle address.
    pub fn address(pool_registry: &Pubkey, oracle_address: &Pubkey) -> Pubkey {
        Self::get_address(&[pool_registry.as_ref(), oracle_address.as_ref()])
    }

    /// Most recently modified index. Returns zero when there is no data.
    pub fn most_recent_index(&self) -> usize {
        self.num_updates as usize % NUM_HISTORICAL_PRICE_ENTRIES
    }

    /// Most recently added value. Returns zeroed bytes when there is no data.
    pub fn most_recent_entry(&self) -> &HistoricalPrice {
        &self.price_history[self.most_recent_index()]
    }

    /// Add a new historical price to the price history
    pub fn push(&mut self, historical_price: HistoricalPrice) {
        // Obtain a mutable slice of the byte portion to be overwritten
        let offset = (self.num_updates as usize + 1) % NUM_HISTORICAL_PRICE_ENTRIES;
        self.price_history[offset] = historical_price;
        // Increment the counter that keeps track of indexing
        self.num_updates += 1;
    }

    /// Converts the [u8] at rest into an [OracleType].
    pub fn oracle_type(&self) -> OracleType {
        OracleType::from(self.oracle_type)
    }

    /// The current price
    /// NOTE: This does not check for price staleness.
    pub fn latest_price(&self) -> Result<HistoricalPrice> {
        let price = *self.most_recent_entry();
        if price == HistoricalPrice::default() {
            return err!(SSLV2Error::PriceHistoryEmpty);
        }
        Ok(price)
    }
}

impl AccountSerialize for OraclePriceHistory {
    fn try_serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        let mut disc = Self::discriminator().to_vec();
        disc.append(&mut bytemuck::bytes_of(self).to_vec());
        writer.write_all(&disc)?;
        Ok(())
    }
}

/// Iterates from newest value to oldest.
#[derive(Debug)]
pub struct AccountHistoryIterator<'data> {
    inner: &'data OraclePriceHistory,
    counter: usize,
    index: usize,
}

impl<'data> Iterator for AccountHistoryIterator<'data> {
    type Item = &'data HistoricalPrice;

    fn next(&mut self) -> Option<Self::Item> {
        if self.counter < NUM_HISTORICAL_PRICE_ENTRIES
            && self.counter < self.inner.num_updates as usize
        {
            let slice = &self.inner.price_history[self.index];
            self.counter += 1;
            self.index = if self.index == 0 {
                NUM_HISTORICAL_PRICE_ENTRIES - 1
            } else {
                self.index - 1
            };
            Some(slice)
        } else {
            None
        }
    }
}

impl<'data> From<&'data OraclePriceHistory> for AccountHistoryIterator<'data> {
    fn from(value: &'data OraclePriceHistory) -> Self {
        let start = value.most_recent_index();
        Self {
            inner: &value,
            counter: 0,
            index: start,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify manual implementation of `AccountSerialize`
    #[test]
    fn serialization() {
        let mut price_history = OraclePriceHistory::default();
        let price = HistoricalPrice {
            price: HistoricalDecimal {
                num: 1000,
                scale: 2,
                _pad0: [0; 4],
            },
            slot: 1234,
        };
        price_history.push(price);
        let price = HistoricalPrice {
            price: HistoricalDecimal {
                num: 2000,
                scale: 2,
                _pad0: [0; 4],
            },
            slot: 1235,
        };
        price_history.push(price);
        let mut serialized = vec![];
        price_history.try_serialize(&mut serialized).unwrap();
        let deserialized: OraclePriceHistory =
            OraclePriceHistory::try_deserialize(&mut serialized.as_slice()).unwrap();
        let latest_price = deserialized.most_recent_entry();
        assert_eq!(*latest_price, price);
    }

    #[test]
    fn gets_most_recent_index() {
        let mut price_history = OraclePriceHistory::default();
        let price = HistoricalPrice {
            price: HistoricalDecimal {
                num: 1000,
                scale: 2,
                _pad0: [0; 4],
            },
            slot: 1,
        };
        price_history.push(price);
        let price = HistoricalPrice {
            price: HistoricalDecimal {
                num: 2000,
                scale: 2,
                _pad0: [0; 4],
            },
            slot: 2,
        };
        price_history.push(price);
        assert_eq!(2, price_history.most_recent_entry().slot);
    }

    #[test]
    fn iteration_starts_at_most_recent() {
        let mut output_token_history = OraclePriceHistory::default();
        let price = HistoricalPrice {
            price: HistoricalDecimal {
                num: 1000,
                scale: 2,
                _pad0: [0; 4],
            },
            slot: 1,
        };
        output_token_history.push(price);
        let price = HistoricalPrice {
            price: HistoricalDecimal {
                num: 2000,
                scale: 2,
                _pad0: [0; 4],
            },
            slot: 2,
        };
        output_token_history.push(price);

        let mut input_token_history = OraclePriceHistory::default();
        let price = HistoricalPrice {
            price: HistoricalDecimal {
                num: 3000,
                scale: 2,
                _pad0: [0; 4],
            },
            slot: 3,
        };
        input_token_history.push(price);
        let price = HistoricalPrice {
            price: HistoricalDecimal {
                num: 4000,
                scale: 2,
                _pad0: [0; 4],
            },
            slot: 4,
        };
        input_token_history.push(price);

        let iterated_prices =
            AccountHistoryIterator::from(&output_token_history).collect::<Vec<_>>();
        assert_eq!(2, iterated_prices.len());
        assert_eq!(2, iterated_prices[0].slot);
        assert_eq!(1, iterated_prices[1].slot);

        let iterated_prices = AccountHistoryIterator::from(&output_token_history)
            .zip(AccountHistoryIterator::from(&input_token_history))
            .collect::<Vec<_>>();
        assert_eq!(2, iterated_prices.len());
        assert_eq!(2, iterated_prices[0].0.slot);
        assert_eq!(4, iterated_prices[0].1.slot);
        assert_eq!(1, iterated_prices[1].0.slot);
        assert_eq!(3, iterated_prices[1].1.slot);
    }
}
