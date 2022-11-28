/*
 * ‌
 * Hedera Rust SDK
 * ​
 * Copyright (C) 2022 - 2023 Hedera Hashgraph, LLC
 * ​
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * ‍
 */

use std::fmt::{
    self,
    Debug,
    Display,
    Formatter,
};
use std::str::FromStr;

use itertools::Itertools;

use crate::evm_address::EvmAddress;
use crate::Error;

/// The ID of an entity on the Hedera network.
#[derive(Hash, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "ffi", derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr))]
pub struct EntityId {
    /// A non-negative number identifying the shard containing this entity.
    pub shard: u64,

    /// A non-negative number identifying the realm within the shard containing this entity.
    pub realm: u64,

    /// A non-negative number identifying the entity within the realm containing this entity.
    pub num: u64,
}

impl EntityId {
    pub(crate) fn from_solidity_address(address: &str) -> crate::Result<Self> {
        EvmAddress::from_str(address).map(Self::from)
    }

    pub(crate) fn to_solidity_address(self) -> crate::Result<String> {
        EvmAddress::try_from(self).map(|it| it.to_string())
    }
}

impl Debug for EntityId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\"{self}\"")
    }
}

impl Display for EntityId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.shard, self.realm, self.num)
    }
}

impl From<u64> for EntityId {
    fn from(num: u64) -> Self {
        Self { num, shard: 0, realm: 0 }
    }
}

impl FromStr for EntityId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<u64> =
            s.splitn(3, '.').map(u64::from_str).try_collect().map_err(Error::basic_parse)?;

        match *parts.as_slice() {
            [num] => Ok(Self::from(num)),
            [shard, realm, num] => Ok(Self { shard, realm, num }),
            _ => Err(Error::basic_parse("expecting <shard>.<realm>.<num> (ex. `0.0.1001`)")),
        }
    }
}
