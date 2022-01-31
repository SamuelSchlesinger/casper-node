// TODO - remove once schemars stops causing warning.
#![allow(clippy::field_reassign_with_default)]

use std::{
    convert::TryFrom,
    fmt::{self, Display, Formatter},
};

use datasize::DataSize;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::cast::{FromPrimitive, ToPrimitive};
#[cfg(test)]
use rand::Rng;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use casper_types::{
    bytesrepr::{self, FromBytes, ToBytes, U8_SERIALIZED_LENGTH},
    EraId,
};

#[cfg(test)]
use crate::testing::TestRng;
use crate::types::Timestamp;

#[derive(FromPrimitive, ToPrimitive, Debug)]
enum ActivationPointTag {
    EraId = 0,
    Genesis = 1,
}

impl TryFrom<u8> for ActivationPointTag {
    type Error = bytesrepr::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        FromPrimitive::from_u8(value).ok_or(bytesrepr::Error::Formatting)
    }
}

/// The first era to which the associated protocol version applies.
#[derive(Copy, Clone, DataSize, PartialEq, Eq, Serialize, Deserialize, Debug, JsonSchema)]
#[serde(untagged)]
pub enum ActivationPoint {
    EraId(EraId),
    Genesis(Timestamp),
}

impl ActivationPoint {
    fn tag(&self) -> ActivationPointTag {
        match self {
            ActivationPoint::EraId(_) => ActivationPointTag::EraId,
            ActivationPoint::Genesis(_) => ActivationPointTag::Genesis,
        }
    }

    fn tag_byte(&self) -> u8 {
        self.to_u8()
            .expect("ActivationPointTag should be representable as a u8")
    }

    /// Returns whether we should upgrade the node due to the next era being at or after the upgrade
    /// activation point.
    pub(crate) fn should_upgrade(&self, era_being_deactivated: &EraId) -> bool {
        match self {
            ActivationPoint::EraId(era_id) => era_being_deactivated.successor() >= *era_id,
            ActivationPoint::Genesis(_) => false,
        }
    }

    /// Returns the Era ID if `self` is of `EraId` variant, or else 0 if `Genesis`.
    pub(crate) fn era_id(&self) -> EraId {
        match self {
            ActivationPoint::EraId(era_id) => *era_id,
            ActivationPoint::Genesis(_) => EraId::from(0),
        }
    }

    /// Returns the timestamp if `self` is of `Genesis` variant, or else `None`.
    pub(crate) fn genesis_timestamp(&self) -> Option<Timestamp> {
        match self {
            ActivationPoint::EraId(_) => None,
            ActivationPoint::Genesis(timestamp) => Some(*timestamp),
        }
    }

    /// Returns true if `self` is `Genesis`.
    pub(crate) fn is_genesis(&self) -> bool {
        match self {
            ActivationPoint::EraId(_) => false,
            ActivationPoint::Genesis(_) => true,
        }
    }
}

impl Display for ActivationPoint {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ActivationPoint::EraId(era_id) => write!(formatter, "activation point {}", era_id),
            ActivationPoint::Genesis(timestamp) => {
                write!(formatter, "activation point {}", timestamp)
            }
        }
    }
}

impl ToBytes for ActivationPoint {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        let mut buffer = vec![self.tag_byte()];
        match self {
            ActivationPoint::EraId(era_id) => {
                buffer.extend(era_id.to_bytes()?);
                Ok(buffer)
            }
            ActivationPoint::Genesis(timestamp) => {
                buffer.extend(timestamp.to_bytes()?);
                Ok(buffer)
            }
        }
    }

    fn serialized_length(&self) -> usize {
        U8_SERIALIZED_LENGTH
            + match self {
                ActivationPoint::EraId(era_id) => era_id.serialized_length(),
                ActivationPoint::Genesis(timestamp) => timestamp.serialized_length(),
            }
    }
}

impl FromBytes for ActivationPoint {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (tag, remainder) = u8::from_bytes(bytes)?;
        match TryFrom::try_from(tag)? {
            ActivationPointTag::EraId => {
                let (era_id, remainder) = EraId::from_bytes(remainder)?;
                Ok((ActivationPoint::EraId(era_id), remainder))
            }
            ActivationPointTag::Genesis => {
                let (timestamp, remainder) = Timestamp::from_bytes(remainder)?;
                Ok((ActivationPoint::Genesis(timestamp), remainder))
            }
        }
    }
}

#[cfg(test)]
impl ActivationPoint {
    /// Generates a random instance using a `TestRng`.
    pub fn random(rng: &mut TestRng) -> Self {
        if rng.gen() {
            ActivationPoint::EraId(rng.gen())
        } else {
            ActivationPoint::Genesis(Timestamp::random(rng))
        }
    }
}
