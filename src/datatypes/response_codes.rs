use std::fmt;
use std::fmt::Formatter;
use serde::{Serialize, Serializer, Deserialize, Deserializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseCodes {
    Approved = 0,
    ReferToCardIssuer = 1,
    InvalidMerchant = 3,
    CaptureCard = 4,
    DoNotHonor = 5,
    HonorWithId = 8,
    PartialApproval = 10,
    InvalidTransaction = 12,
    InvalidAmount = 13,
    InvalidCardNumber = 14,
    InvalidIssuer = 15,
    FormatError = 30,
    LostCard = 41,
    StolenCard = 43,
    InsufficientFunds = 51,
    ExpiredCard = 54,
    InvalidPin = 55,
    TransactionNotPermittedToCardholder = 57,
    TransactionNotPermittedToTerminal = 58,
    ExceedsWithdrawalAmountLimit = 61,
    RestrictedCard = 62,
    ExceedsWithdrawalCountLimit = 65,
    // CaptureCard = 67, Not used, for some reason it appears twice
    AllowableNumberOfPinTriesExceeded = 75,
    DeactivatedCard = 78,
    InvalidNonexistentAccountSpecified = 79,
    DoNotHonorSwitch = 80,
    CancelledCard = 81,
    AuthorizationPlatformOrIssuerSystemInoperative = 84,
    NotDeclined = 85,
    PinValidationNotPossible = 86,
    ApprovedPurchaseAmountOnlyNoCashBackAllowed = 87,
    CryptographicFailure = 88,
    InvalidCryptogram = 89,
    AuthorizationSystemOrIssuerSystemInoperative = 91,
    DuplicateTransmissionDetected = 94,
    SystemError = 96,
}

impl fmt::Display for ResponseCodes {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        format!("{:?}({})", self, *self as u8).fmt(f)
    }
}

impl Serialize for ResponseCodes {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
        S: Serializer {
        serializer.serialize_u8(*self as u8)
    }
}

impl Default for ResponseCodes {
    fn default() -> Self {
        Self::SystemError
    }
}

impl ResponseCodes {
    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            x if x == ResponseCodes::Approved as u8 => Some(ResponseCodes::Approved),
            x if x == ResponseCodes::ReferToCardIssuer as u8 => Some(ResponseCodes::ReferToCardIssuer),
            x if x == ResponseCodes::InvalidMerchant as u8 => Some(ResponseCodes::InvalidMerchant),
            x if x == ResponseCodes::CaptureCard as u8 => Some(ResponseCodes::CaptureCard),
            x if x == ResponseCodes::DoNotHonor as u8 => Some(ResponseCodes::DoNotHonor),
            x if x == ResponseCodes::HonorWithId as u8 => Some(ResponseCodes::HonorWithId),
            x if x == ResponseCodes::PartialApproval as u8 => Some(ResponseCodes::PartialApproval),
            x if x == ResponseCodes::InvalidTransaction as u8 => Some(ResponseCodes::InvalidTransaction),
            x if x == ResponseCodes::InvalidAmount as u8 => Some(ResponseCodes::InvalidAmount),
            x if x == ResponseCodes::InvalidCardNumber as u8 => Some(ResponseCodes::InvalidCardNumber),
            x if x == ResponseCodes::InvalidIssuer as u8 => Some(ResponseCodes::InvalidIssuer),
            x if x == ResponseCodes::FormatError as u8 => Some(ResponseCodes::FormatError),
            x if x == ResponseCodes::ExpiredCard as u8 => Some(ResponseCodes::ExpiredCard),
            x if x == ResponseCodes::LostCard as u8 => Some(ResponseCodes::LostCard),
            x if x == ResponseCodes::StolenCard as u8 => Some(ResponseCodes::StolenCard),
            x if x == ResponseCodes::InsufficientFunds as u8 => Some(ResponseCodes::InsufficientFunds),
            x if x == ResponseCodes::InvalidPin as u8 => Some(ResponseCodes::InvalidPin),
            x if x == ResponseCodes::TransactionNotPermittedToCardholder as u8 => Some(ResponseCodes::TransactionNotPermittedToCardholder),
            x if x == ResponseCodes::TransactionNotPermittedToTerminal as u8 => Some(ResponseCodes::TransactionNotPermittedToTerminal),
            x if x == ResponseCodes::ExceedsWithdrawalAmountLimit as u8 => Some(ResponseCodes::ExceedsWithdrawalAmountLimit),
            x if x == ResponseCodes::RestrictedCard as u8 => Some(ResponseCodes::RestrictedCard),
            x if x == ResponseCodes::ExceedsWithdrawalCountLimit as u8 => Some(ResponseCodes::ExceedsWithdrawalCountLimit),
            x if x == ResponseCodes::AllowableNumberOfPinTriesExceeded as u8 => Some(ResponseCodes::AllowableNumberOfPinTriesExceeded),
            x if x == ResponseCodes::DeactivatedCard as u8 => Some(ResponseCodes::DeactivatedCard),
            x if x == ResponseCodes::InvalidNonexistentAccountSpecified as u8 => Some(ResponseCodes::InvalidNonexistentAccountSpecified),
            x if x == ResponseCodes::DoNotHonorSwitch as u8 => Some(ResponseCodes::DoNotHonorSwitch),
            x if x == ResponseCodes::CancelledCard as u8 => Some(ResponseCodes::CancelledCard),
            x if x == ResponseCodes::AuthorizationPlatformOrIssuerSystemInoperative as u8 => Some(ResponseCodes::AuthorizationPlatformOrIssuerSystemInoperative),
            x if x == ResponseCodes::NotDeclined as u8 => Some(ResponseCodes::NotDeclined),
            x if x == ResponseCodes::PinValidationNotPossible as u8 => Some(ResponseCodes::PinValidationNotPossible),
            x if x == ResponseCodes::CryptographicFailure as u8 => Some(ResponseCodes::CryptographicFailure),
            x if x == ResponseCodes::InvalidCryptogram as u8 => Some(ResponseCodes::InvalidCryptogram),
            x if x == ResponseCodes::AuthorizationSystemOrIssuerSystemInoperative as u8 => Some(ResponseCodes::AuthorizationSystemOrIssuerSystemInoperative),
            x if x == ResponseCodes::DuplicateTransmissionDetected as u8 => Some(ResponseCodes::DuplicateTransmissionDetected),
            x if x == ResponseCodes::SystemError as u8 => Some(ResponseCodes::SystemError),
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for ResponseCodes {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where
        D: Deserializer<'de> {
        Ok(ResponseCodes::from_code(u8::deserialize(deserializer)?).unwrap_or_default())
    }
}
