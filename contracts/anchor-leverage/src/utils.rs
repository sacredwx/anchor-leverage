use std::ops::Mul;

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::Decimal;

use crate::contract::DECIMAL_FRACTIONAL;

pub fn get_opposite_ratio(ratio: Decimal) -> Decimal {
    Decimal::from_ratio(DECIMAL_FRACTIONAL, ratio.mul(DECIMAL_FRACTIONAL.into()))
}

pub fn calculate_borrow(borrow_limit: Uint256, already_borrowed: Uint256) -> Uint256 {
    borrow_limit.mul(Decimal256::percent(70)) - already_borrowed
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_opposite_ratio() {
        assert_eq!(
            get_opposite_ratio(Decimal::from_str("0.999834039454456203").unwrap()),
            Decimal::from_str("1.000165988093018268").unwrap()
        );
        assert_eq!(
            get_opposite_ratio(Decimal::from_str("1.000165988093018268").unwrap()),
            Decimal::from_str("0.999834039454456203").unwrap()
        );
        assert_eq!(
            get_opposite_ratio(Decimal::from_str("1").unwrap()),
            Decimal::from_str("1").unwrap()
        );
    }

    #[test]
    fn test_calculate_borrow() {
        assert_eq!(
            calculate_borrow(
                Uint256::from(16511228606u128),
                Uint256::from(8394109000u128)
            ),
            Uint256::from(3163751024u128)
        );
    }
}
