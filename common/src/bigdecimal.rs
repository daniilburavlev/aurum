use crate::biginteger::BigInt;
use bigdecimal::num_bigint::ToBigInt;
use bigdecimal::{FromPrimitive, Zero};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::ops::{Add, AddAssign, Deref, Div, Mul, MulAssign, Sub, SubAssign};
use std::str::FromStr;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BigDecimal(pub bigdecimal::BigDecimal);

impl Deref for BigDecimal {
    type Target = bigdecimal::BigDecimal;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl BigDecimal {
    pub fn from_str(value: &str) -> Result<Self, std::io::Error> {
        let value = bigdecimal::BigDecimal::from_str(value)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))?;
        Ok(Self(value))
    }

    pub fn from_usize(value: usize) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self(bigdecimal::BigDecimal::from_usize(value).unwrap()))
    }

    pub fn to_bigint(&self) -> Option<BigInt> {
        if let Some(value) = self.0.to_bigint() {
            Some(BigInt::from_bigint(value))
        } else {
            None
        }
    }

    pub fn zero() -> Self {
        Self(bigdecimal::BigDecimal::zero())
    }
}

impl Add for BigDecimal {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for BigDecimal {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for BigDecimal {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for BigDecimal {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Mul for BigDecimal {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl MulAssign for BigDecimal {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0;
    }
}

impl Div for BigDecimal {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl Serialize for BigDecimal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_plain_string())
    }
}

impl<'a> Deserialize<'a> for BigDecimal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(Self(
            bigdecimal::BigDecimal::from_str(&value).map_err(serde::de::Error::custom)?,
        ))
    }
}

impl PartialOrd for BigDecimal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for BigDecimal {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}
