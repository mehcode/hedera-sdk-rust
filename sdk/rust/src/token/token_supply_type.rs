use hedera_proto::services;
use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    FromProtobuf,
    ToProtobuf,
};

/// Possible token supply types.
/// Can be used to restrict supply to a set maximum.
/// Defaults to [`Infinite`].
#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
#[repr(C)]
pub enum TokenSupplyType {
    /// Indicates the token has a maximum supply of [`u64::MAX`].
    Infinite = 0,

    /// Indicates the token has a configurable maximum supply, provided on token creation.
    Finite = 1,
}

impl FromProtobuf for TokenSupplyType {
    type Protobuf = services::TokenSupplyType;

    fn from_protobuf(pb: Self::Protobuf) -> crate::Result<Self> {
        Ok(match pb {
            Self::Protobuf::Infinite => Self::Infinite,
            Self::Protobuf::Finite => Self::Finite,
        })
    }
}

impl ToProtobuf for TokenSupplyType {
    type Protobuf = services::TokenSupplyType;

    fn to_protobuf(&self) -> Self::Protobuf {
        match self {
            Self::Infinite => Self::Protobuf::Infinite,
            Self::Finite => Self::Protobuf::Finite,
        }
    }
}

#[cfg(test)]
mod tests {
    use hedera_proto::services;

    use crate::token::token_supply_type::TokenSupplyType;
    use crate::{
        FromProtobuf,
        ToProtobuf,
    };

    #[test]
    fn it_can_convert_to_protobuf() -> anyhow::Result<()> {
        let infinite_supply_type = TokenSupplyType::Infinite;
        let finite_supply_type = TokenSupplyType::Finite;

        let infinite_protobuf = infinite_supply_type.to_protobuf();
        let finite_protobuf = finite_supply_type.to_protobuf();

        assert_eq!(infinite_protobuf, services::TokenSupplyType::Infinite);
        assert_eq!(finite_protobuf, services::TokenSupplyType::Finite);

        Ok(())
    }

    #[test]
    fn it_can_be_created_from_protobuf() -> anyhow::Result<()> {
        let infinite_protobuf = services::TokenSupplyType::Infinite;
        let finite_protobuf = services::TokenSupplyType::Finite;

        let infinite_supply_type = TokenSupplyType::from_protobuf(infinite_protobuf)?;
        let finite_supply_type = TokenSupplyType::from_protobuf(finite_protobuf)?;

        assert_eq!(infinite_supply_type, TokenSupplyType::Infinite);
        assert_eq!(finite_supply_type, TokenSupplyType::Finite);

        Ok(())
    }
}