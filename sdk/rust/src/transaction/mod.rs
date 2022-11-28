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

use std::fmt;
use std::fmt::{
    Debug,
    Formatter,
};

use time::Duration;

use crate::execute::execute;
use crate::signer::Signer;
use crate::{
    AccountId,
    ArbitrarySigner,
    Client,
    Hbar,
    PrivateKey,
    PublicKey,
    TransactionId,
    TransactionResponse,
};

mod any;
mod execute;
mod protobuf;

#[cfg(feature = "ffi")]
pub use any::AnyTransaction;
#[cfg(feature = "ffi")]
pub(crate) use any::AnyTransactionBody;
pub(crate) use any::AnyTransactionData;
pub(crate) use execute::TransactionExecute;
pub(crate) use protobuf::ToTransactionDataProtobuf;

const DEFAULT_TRANSACTION_VALID_DURATION: Duration = Duration::seconds(120);

/// A transaction that can be executed on the Hedera network.
#[cfg_attr(feature = "ffi", derive(serde::Serialize))]
pub struct Transaction<D>
where
    D: TransactionExecute,
{
    #[cfg_attr(feature = "ffi", serde(flatten))]
    pub(crate) body: TransactionBody<D>,

    #[cfg_attr(feature = "ffi", serde(skip))]
    pub(crate) signers: Vec<Signer>,
}

#[cfg_attr(feature = "ffi", serde_with::skip_serializing_none)]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "ffi", derive(serde::Serialize))]
#[cfg_attr(feature = "ffi", serde(rename_all = "camelCase"))]
// fires because of `serde_as`
#[allow(clippy::type_repetition_in_bounds)]
pub(crate) struct TransactionBody<D>
where
    D: TransactionExecute,
{
    #[cfg_attr(feature = "ffi", serde(flatten))]
    #[cfg_attr(
        feature = "ffi",
        serde(with = "serde_with::As::<serde_with::FromInto<AnyTransactionData>>")
    )]
    pub(crate) data: D,

    pub(crate) node_account_ids: Option<Vec<AccountId>>,

    #[cfg_attr(
        feature = "ffi",
        serde(with = "serde_with::As::<Option<serde_with::DurationSeconds<i64>>>")
    )]
    pub(crate) transaction_valid_duration: Option<Duration>,

    pub(crate) max_transaction_fee: Option<Hbar>,

    #[cfg_attr(feature = "ffi", serde(skip_serializing_if = "String::is_empty"))]
    pub(crate) transaction_memo: String,

    pub(crate) payer_account_id: Option<AccountId>,

    pub(crate) transaction_id: Option<TransactionId>,
}

impl<D> Default for Transaction<D>
where
    D: Default + TransactionExecute,
{
    fn default() -> Self {
        Self {
            body: TransactionBody {
                data: D::default(),
                node_account_ids: None,
                transaction_valid_duration: None,
                transaction_memo: String::new(),
                max_transaction_fee: None,
                payer_account_id: None,
                transaction_id: None,
            },
            signers: Vec::new(),
        }
    }
}

impl<D> Debug for Transaction<D>
where
    D: Debug + TransactionExecute,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Transaction").field("body", &self.body).finish()
    }
}

impl<D> Transaction<D>
where
    D: Default + TransactionExecute,
{
    /// Create a new default transaction.
    ///
    /// Does the same thing as [`default`](Self::default)
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<D> Transaction<D>
where
    D: TransactionExecute,
{
    /// Set the account IDs of the nodes that this transaction may be submitted to.
    ///
    /// Defaults to the full list of nodes configured on the client.
    pub fn node_account_ids(&mut self, ids: impl IntoIterator<Item = AccountId>) -> &mut Self {
        self.body.node_account_ids = Some(ids.into_iter().collect());
        self
    }

    /// Sets the duration that this transaction is valid for, once finalized and signed.
    ///
    /// Defaults to 120 seconds (or two minutes).
    pub fn transaction_valid_duration(&mut self, duration: Duration) -> &mut Self {
        self.body.transaction_valid_duration = Some(duration);
        self
    }

    /// Set the maximum transaction fee the paying account is willing to pay.
    pub fn max_transaction_fee(&mut self, fee: Hbar) -> &mut Self {
        self.body.max_transaction_fee = Some(fee);
        self
    }

    /// Set a note or description that should be recorded in the transaction record (maximum length
    /// of 100 characters).
    pub fn transaction_memo(&mut self, memo: impl AsRef<str>) -> &mut Self {
        self.body.transaction_memo = memo.as_ref().to_owned();
        self
    }

    /// Set an explicit transaction ID to use to identify this transaction.
    ///
    /// Overrides payer account defined on this transaction or on the client.
    pub fn transaction_id(&mut self, id: TransactionId) -> &mut Self {
        self.body.transaction_id = Some(id);
        self
    }

    /// Sign the transaction.
    pub fn sign(&mut self, private_key: PrivateKey) -> &mut Self {
        self.sign_signer(Signer::PrivateKey(private_key))
    }

    /// Sign the transaction.
    pub fn sign_with<F>(&mut self, public_key: PublicKey, signer: ArbitrarySigner) -> &mut Self {
        self.sign_signer(Signer::Arbitrary(public_key, signer))
    }

    pub(crate) fn sign_signer(&mut self, signer: Signer) -> &mut Self {
        self.signers.push(signer);
        self
    }
}

impl<D> Transaction<D>
where
    D: TransactionExecute,
{
    /// Execute this transaction against the provided client of the Hedera network.
    // todo:
    #[allow(clippy::missing_errors_doc)]
    pub async fn execute(&mut self, client: &Client) -> crate::Result<TransactionResponse> {
        execute(client, self, None).await
    }

    #[cfg(feature = "ffi")]
    pub(crate) async fn execute_with_optional_timeout(
        &mut self,
        client: &Client,
        timeout: Option<std::time::Duration>,
    ) -> crate::Result<TransactionResponse> {
        execute(client, self, timeout).await
    }

    /// Execute this transaction against the provided client of the Hedera network.
    // todo:
    #[allow(clippy::missing_errors_doc)]
    pub async fn execute_with_timeout(
        &mut self,
        client: &Client,
        // fixme: be consistent with `time::Duration`? Except `tokio::time` is `std::time`, and we depend on tokio.
        timeout: std::time::Duration,
    ) -> crate::Result<TransactionResponse> {
        execute(client, self, timeout).await
    }
}
