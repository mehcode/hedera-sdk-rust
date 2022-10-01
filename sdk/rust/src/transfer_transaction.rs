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

use std::ops::Not;

use async_trait::async_trait;
use hedera_proto::services;
use hedera_proto::services::crypto_service_client::CryptoServiceClient;
use tonic::transport::Channel;

use crate::transaction::{
    AnyTransactionData,
    ToTransactionDataProtobuf,
    TransactionExecute,
};
use crate::{
    AccountId,
    Hbar,
    NftId,
    ToProtobuf,
    TokenId,
    Transaction,
};

/// Transfers cryptocurrency among two or more accounts by making the desired adjustments to their
/// balances.
///
/// Each transfer list can specify up to 10 adjustments. Each negative amount is withdrawn
/// from the corresponding account (a sender), and each positive one is added to the corresponding
/// account (a receiver). The amounts list must sum to zero.
///
pub type TransferTransaction = Transaction<TransferTransactionData>;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct TransferTransactionData {
    transfers: Vec<Transfer>,
    token_transfers: Vec<TokenTransfer>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Transfer {
    account_id: AccountId,

    #[serde(default)]
    amount: i64,

    #[serde(default)]
    is_approval: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenTransfer {
    token_id: TokenId,

    #[serde(default)]
    transfers: Vec<Transfer>,

    #[serde(default)]
    nft_transfers: Vec<NftTransfer>,

    #[serde(default)]
    expected_decimals: Option<u32>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct NftTransfer {
    sender_account_id: AccountId,
    receiver_account_id: AccountId,

    #[serde(default)]
    serial_number: u64,

    #[serde(default)]
    is_approval: bool,
}

impl TransferTransaction {
    fn _token_transfer(
        &mut self,
        token_id: TokenId,
        account_id: AccountId,
        amount: i64,
        approved: bool,
        expected_decimals: Option<u32>,
    ) -> &mut Self {
        let transfer = Transfer { account_id, amount, is_approval: approved };

        if let Some(tt) =
            self.body.data.token_transfers.iter_mut().find(|tt| tt.token_id == token_id)
        {
            tt.expected_decimals = expected_decimals;
            tt.transfers.push(transfer);
        } else {
            self.body.data.token_transfers.push(TokenTransfer {
                token_id,
                expected_decimals,
                nft_transfers: Vec::new(),
                transfers: vec![transfer],
            });
        }

        self
    }

    /// Add a non-approved token transfer to the transaction.
    pub fn token_transfer(
        &mut self,
        token_id: TokenId,
        account_id: AccountId,
        amount: i64,
    ) -> &mut Self {
        self._token_transfer(token_id, account_id, amount, false, None)
    }

    /// Add an approved token transfer to the transaction.
    pub fn approved_token_transfer(
        &mut self,
        token_id: TokenId,
        account_id: AccountId,
        amount: i64,
    ) -> &mut Self {
        self._token_transfer(token_id, account_id, amount, true, None)
    }

    /// Add a non-approved token transfer with decimals to the transaction.
    pub fn token_transfer_with_decimals(
        &mut self,
        token_id: TokenId,
        account_id: AccountId,
        amount: i64,
        expected_decimals: u32,
    ) -> &mut Self {
        self._token_transfer(token_id, account_id, amount, false, Some(expected_decimals))
    }

    /// Add an approved token transfer with decimals to the transaction.
    pub fn approved_token_transfer_with_decimals(
        &mut self,
        token_id: TokenId,
        account_id: AccountId,
        amount: i64,
        expected_decimals: u32,
    ) -> &mut Self {
        self._token_transfer(token_id, account_id, amount, true, Some(expected_decimals))
    }

    fn _nft_transfer(
        &mut self,
        nft_id: NftId,
        sender_account_id: AccountId,
        receiver_account_id: AccountId,
        approved: bool,
    ) -> &mut Self {
        let NftId { token_id, serial_number } = nft_id;
        let transfer = NftTransfer {
            serial_number,
            sender_account_id,
            receiver_account_id,
            is_approval: approved,
        };

        if let Some(tt) =
            self.body.data.token_transfers.iter_mut().find(|tt| tt.token_id == token_id)
        {
            tt.nft_transfers.push(transfer);
        } else {
            self.body.data.token_transfers.push(TokenTransfer {
                token_id,
                expected_decimals: None,
                transfers: Vec::new(),
                nft_transfers: vec![transfer],
            });
        }

        self
    }

    /// Add an approved nft transfer to the transaction.
    pub fn approved_nft_transfer(
        &mut self,
        nft_id: impl Into<NftId>,
        sender_account_id: AccountId,
        receiver_account_id: AccountId,
    ) -> &mut Self {
        self._nft_transfer(nft_id.into(), sender_account_id, receiver_account_id, true)
    }

    /// Add a non-approved nft transfer to the transaction.
    pub fn nft_transfer(
        &mut self,
        nft_id: impl Into<NftId>,
        sender_account_id: AccountId,
        receiver_account_id: AccountId,
    ) -> &mut Self {
        self._nft_transfer(nft_id.into(), sender_account_id, receiver_account_id, false)
    }

    fn _hbar_transfer(&mut self, account_id: AccountId, amount: Hbar, approved: bool) -> &mut Self {
        self.body.data.transfers.push(Transfer {
            account_id,
            amount: amount.to_tinybars(),
            is_approval: approved,
        });

        self
    }

    /// Add a non-approved hbar transfer to the transaction.
    pub fn hbar_transfer(&mut self, account_id: AccountId, amount: Hbar) -> &mut Self {
        self._hbar_transfer(account_id, amount, false)
    }

    /// Add an approved hbar transfer to the transaction.
    pub fn approved_hbar_transfer(&mut self, account_id: AccountId, amount: Hbar) -> &mut Self {
        self._hbar_transfer(account_id, amount, true)
    }
}

#[async_trait]
impl TransactionExecute for TransferTransactionData {
    // noinspection DuplicatedCode
    async fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> Result<tonic::Response<services::TransactionResponse>, tonic::Status> {
        CryptoServiceClient::new(channel).crypto_transfer(request).await
    }
}

impl ToProtobuf for Transfer {
    type Protobuf = services::AccountAmount;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::AccountAmount {
            amount: self.amount,
            account_id: Some(self.account_id.to_protobuf()),
            is_approval: self.is_approval,
        }
    }
}

impl ToProtobuf for TokenTransfer {
    type Protobuf = services::TokenTransferList;

    fn to_protobuf(&self) -> Self::Protobuf {
        let transfers = self.transfers.iter().map(Transfer::to_protobuf).collect();
        let nft_transfers = self.nft_transfers.iter().map(NftTransfer::to_protobuf).collect();

        services::TokenTransferList {
            token: Some(self.token_id.to_protobuf()),
            transfers,
            nft_transfers,
            expected_decimals: self.expected_decimals,
        }
    }
}

impl ToProtobuf for NftTransfer {
    type Protobuf = services::NftTransfer;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::NftTransfer {
            sender_account_id: Some(self.sender_account_id.to_protobuf()),
            receiver_account_id: Some(self.receiver_account_id.to_protobuf()),
            serial_number: self.serial_number as i64,
            is_approval: self.is_approval,
        }
    }
}

impl ToTransactionDataProtobuf for TransferTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        _node_account_id: crate::AccountId,
        _transaction_id: &crate::TransactionId,
    ) -> services::transaction_body::Data {
        let transfers = self.transfers.is_empty().not().then(|| services::TransferList {
            account_amounts: self.transfers.iter().map(Transfer::to_protobuf).collect(),
        });

        let token_transfers = self.token_transfers.iter().map(TokenTransfer::to_protobuf).collect();

        services::transaction_body::Data::CryptoTransfer(services::CryptoTransferTransactionBody {
            transfers,
            token_transfers,
        })
    }
}

impl From<TransferTransactionData> for AnyTransactionData {
    fn from(transaction: TransferTransactionData) -> Self {
        Self::Transfer(transaction)
    }
}

// hack(sr): these tests currently don't compile due to `payer_account_id`
#[cfg(feature = "false")]
mod tests {
    use assert_matches::assert_matches;

    use crate::transaction::{
        AnyTransaction,
        AnyTransactionData,
    };
    use crate::{
        AccountId,
        TransferTransaction,
    };

    // language=JSON
    const TRANSFER_HBAR: &str = r#"{
  "$type": "transfer",
  "transfers": [
    {
      "accountId": "0.0.1001",
      "amount": 20,
      "isApproval": false
    },
    {
      "accountId": "0.0.1002",
      "amount": -20,
      "isApproval": false
    }
  ],
  "tokenTransfers": [],
  "payerAccountId": "0.0.6189"
}"#;

    #[test]
    fn it_should_serialize() -> anyhow::Result<()> {
        let mut transaction = TransferTransaction::new();
        transaction
            .hbar_transfer(AccountId::from(1001), 20)
            .hbar_transfer(AccountId::from(1002), -20)
            .payer_account_id(AccountId::from(6189));

        let s = serde_json::to_string_pretty(&transaction)?;
        assert_eq!(s, TRANSFER_HBAR);

        Ok(())
    }

    #[test]
    fn it_should_deserialize() -> anyhow::Result<()> {
        let transaction: AnyTransaction = serde_json::from_str(TRANSFER_HBAR)?;

        let data = assert_matches!(transaction.body.data, AnyTransactionData::Transfer(transaction) => transaction);

        assert_eq!(data.transfers.len(), 2);

        assert_eq!(data.transfers[0].amount, 20);
        assert_eq!(data.transfers[1].amount, -20);

        Ok(())
    }
}
