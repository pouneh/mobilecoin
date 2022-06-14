// Copyright (c) 2018-2022 The MobileCoin Foundation

//! Error types used by this crate

pub use retry::Error as RetryError;

use crate::traits::AttestationError;
use displaydoc::Display;
use grpcio::Error as GrpcError;
use mc_blockchain_types::ConvertError;
use mc_consensus_api::{consensus_common::ProposeTxResult, ConversionError};
use mc_crypto_noise::CipherError;
use mc_transaction_core::validation::TransactionValidationError;
use std::{array::TryFromSliceError, convert::TryInto, result::Result as StdResult};

pub type Result<T> = StdResult<T, Error>;
pub type RetryResult<T> = StdResult<T, RetryError<Error>>;

/// An enumeration of errors which can be generated by a connection
#[derive(Debug, Display)]
pub enum Error {
    /// The requested range was too large
    RequestTooLarge,
    /// Not found
    NotFound,
    /// Could not convert gRPC type to working type: {0}
    Conversion(ConversionError),
    /// gRPC failure: {0}
    Grpc(GrpcError),
    /// Encryption/decryption failure: {0}
    Cipher(CipherError),
    /// Attestation failure: {0}
    Attestation(Box<dyn AttestationError + 'static>),
    /// Transaction validation failure: {0}
    TransactionValidation(TransactionValidationError),
    /// Other error: {0}
    Other(String),
}

impl Error {
    /// Policy decision, whether the call should be retried.
    pub fn should_retry(&self) -> bool {
        match self {
            Error::Grpc(_) => true,
            Error::Attestation(err) => err.should_retry(),
            _ => false,
        }
    }
}

impl<AE: AttestationError + 'static> From<AE> for Error {
    fn from(src: AE) -> Self {
        Error::Attestation(Box::new(src))
    }
}

impl From<CipherError> for Error {
    fn from(src: CipherError) -> Self {
        Error::Cipher(src)
    }
}

impl From<GrpcError> for Error {
    fn from(src: GrpcError) -> Self {
        Error::Grpc(src)
    }
}

impl TryInto<GrpcError> for Error {
    type Error = Error;

    fn try_into(self) -> Result<GrpcError> {
        match self {
            Error::Grpc(ge) => Ok(ge),
            error => Err(error),
        }
    }
}

impl From<ConversionError> for Error {
    fn from(src: ConversionError) -> Self {
        Error::Conversion(src)
    }
}

impl From<TryFromSliceError> for Error {
    fn from(_src: TryFromSliceError) -> Self {
        ConversionError::ArrayCastError.into()
    }
}

impl From<ProposeTxResult> for Error {
    fn from(src: ProposeTxResult) -> Self {
        src.try_into()
            .map(Self::TransactionValidation)
            .unwrap_or_else(|err| Error::Other(err.into()))
    }
}

impl From<ConvertError> for Error {
    fn from(_src: ConvertError) -> Self {
        ConversionError::ArrayCastError.into()
    }
}
