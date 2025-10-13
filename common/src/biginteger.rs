use bigdecimal::{FromPrimitive, Zero};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::ops::{Add, AddAssign, Deref, Rem, Sub, SubAssign};
use std::str::FromStr;

#[derive(Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct BigInt(bigdecimal::num_bigint::BigInt);

impl BigInt {
    pub fn from_str(value: &str) -> Result<Self, std::io::Error> {
        let value = bigdecimal::num_bigint::BigInt::from_str(value)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))?;
        Ok(Self(value))
    }

    pub fn from_bigint(bigint: bigdecimal::num_bigint::BigInt) -> Self {
        BigInt(bigint)
    }

    pub fn from_u64(value: u64) -> Option<BigInt> {
        if let Some(value) = bigdecimal::num_bigint::BigInt::from_u64(value) {
            Some(BigInt(value))
        } else {
            None
        }
    }

    pub fn zero() -> Self {
        Self(bigdecimal::num_bigint::BigInt::zero())
    }
}

impl Deref for BigInt {
    type Target = bigdecimal::num_bigint::BigInt;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Add for BigInt {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for BigInt {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl SubAssign for BigInt {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Sub for BigInt {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Rem for BigInt {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Self(self.0 % rhs.0)
    }
}

impl Serialize for BigInt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'a> Deserialize<'a> for BigInt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(Self(
            bigdecimal::num_bigint::BigInt::from_str(&value).map_err(serde::de::Error::custom)?,
        ))
    }
}
