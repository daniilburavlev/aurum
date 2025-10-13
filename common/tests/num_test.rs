use common::bigdecimal::BigDecimal;
use common::biginteger::BigInt;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
struct Test {
    int: BigInt,
    decimal: BigDecimal,
}

#[test]
fn test_serialize_deserialize() {
    let test = Test {
        int: BigInt::from_str("1").unwrap(),
        decimal: BigDecimal::from_str("0.001").unwrap(),
    };
    let json = serde_json::to_string_pretty(&test).unwrap();
    let restored: Test = serde_json::from_str(&json).unwrap();
    assert_eq!(test, restored);
}

#[test]
fn test_add() {
    let expected = BigInt::from_str("3").unwrap();
    let result = BigInt::from_str("2").unwrap() + BigInt::from_str("1").unwrap();
    assert_eq!(expected, result);

    let expected = BigDecimal::from_str("0.00001").unwrap();
    let result =
        BigDecimal::from_str("0.000005").unwrap() + BigDecimal::from_str("0.000005").unwrap();
    assert_eq!(expected, result);
}

#[test]
fn test_subtract() {
    let expected = BigInt::from_str("1").unwrap();
    let result = BigInt::from_str("2").unwrap() - BigInt::from_str("1").unwrap();
    assert_eq!(expected, result);

    let expected = BigDecimal::from_str("0.00001").unwrap();
    let result =
        BigDecimal::from_str("0.00099").unwrap() - BigDecimal::from_str("0.00098").unwrap();
    assert_eq!(expected, result);
}
