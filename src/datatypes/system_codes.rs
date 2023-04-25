use std::fmt::Display;
use serde::{Serialize, Serializer};
use crate::datatypes::response_codes::ResponseCodes;
use logger::{ErrorTypes, Locations, MyError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemErrorCodes {
    Unknown,// Supports older versions where InternalResponseCodes was not present, do not use
    UnknownReason,
    UnReachable(u8), // 11
    ClosedChannel(u8), // 4
    BufferFull,
    DbNoConn(u8), // 14
    DbQuery(u8), // 36
    DbStmt(u8), // 12
    DbTransaction(u8), // 3
    DbRollback(u8), // 2
    DbCommit(u8), // 3
    NoInsertId(u8), // 7
    BadFormat,
    DecimalToF64,
    StringParse(u8), // 2
    JsonParse(u8), // 5
    ExchangeConfiguration,
    MissingCreditCurrency(u8), // 2
    MissingDebitCurrency(u8), // 2
    MissingFromCurrency(u8), // 1
    MissingToCurrency(u8), // 1
    ScriptError(u8), // 5
    MissingKey(u8), // 5
    InvalidCipherResponse,
    CipherError,
    TcpConn,
    InvalidProductId(u8), // 2
    InvalidEntityId,
    InvalidFraudRuleGroup,
    TransactionGroupNotFound(u8), // 5
    TransactionLogNotFound,
    UnknownBlocksId(u8), // 4
    UnknownOperation,
    NoBalancesLock(u8), // 5
    UnknownWalletsId,
    InconsistentTransactionGroup(u8),
    Encryption,
    RequestError,
    LastLogIdChanged,
    NoCollectingBalances,
}

impl std::fmt::Display for SystemErrorCodes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!( f, "{:?}", self)
    }
}

impl SystemErrorCodes {
    pub fn code(&self) -> u16 {
        match self {
            Self::Unknown => 1000,
            Self::UnknownReason => 1100,
            Self::UnReachable(v) => 1200+*v as u16,
            Self::ClosedChannel(v) => 1300+*v as u16,
            Self::BufferFull => 1400,
            Self::DbNoConn(v) => 2000+*v as u16,
            Self::DbQuery(v) => 2100+*v as u16,
            Self::DbStmt(v) => 2200+*v as u16,
            Self::DbTransaction(v) => 2300+*v as u16,
            Self::DbRollback(v) => 2400+*v as u16,
            Self::DbCommit(v) => 2500+*v as u16,
            Self::NoInsertId(v) => 2600+*v as u16,
            Self::BadFormat => 3000,
            Self::DecimalToF64 => 3100,
            Self::StringParse(v) => 3200+*v as u16,
            Self::JsonParse(v) => 3300+*v as u16,
            Self::ExchangeConfiguration => 4000,
            Self::MissingCreditCurrency(v) => 4100+*v as u16,
            Self::MissingDebitCurrency(v) => 4200+*v as u16,
            Self::MissingFromCurrency(v) => 4300+*v as u16,
            Self::MissingToCurrency(v) => 4400+*v as u16,
            Self::ScriptError(v) => 5000+*v as u16,
            Self::MissingKey(v) => 5100+*v as u16,
            Self::InvalidCipherResponse => 5200,
            Self::CipherError => 5300,
            Self::TcpConn => 5400,
            Self::InvalidProductId(v) => 6000+*v as u16,
            Self::InvalidEntityId => 6100,
            Self::InvalidFraudRuleGroup => 6200,
            Self::TransactionGroupNotFound(v) => 6300+*v as u16,
            Self::TransactionLogNotFound => 6400,
            Self::UnknownBlocksId(v) => 6500+*v as u16,
            Self::UnknownOperation => 6600,
            Self::NoBalancesLock(v) => 6700+*v as u16,
            Self::UnknownWalletsId => 6800,
            Self::InconsistentTransactionGroup(v) => 6900 + *v as u16,
            Self::Encryption => 7000,
            Self::RequestError => 7100,
            Self::LastLogIdChanged => 7200,
            Self::NoCollectingBalances => 7300
        }
    }
    pub fn as_response_code(&self) -> ResponseCodes {
        ResponseCodes::SystemError
    }
    pub fn from_u16(code: u16) -> Option<Self> {
        match code / 100 {
            10 => Some(Self::Unknown),
            11 => Some(Self::UnknownReason),
            12 => Some(Self::UnReachable((code % 100) as u8)),
            13 => Some(Self::ClosedChannel((code % 100) as u8)),
            14 => Some(Self::BufferFull),
            20 => Some(Self::DbNoConn((code % 100) as u8)),
            21 => Some(Self::DbQuery((code % 100) as u8)),
            22 => Some(Self::DbStmt((code % 100) as u8)),
            23 => Some(Self::DbTransaction((code % 100) as u8)),
            24 => Some(Self::DbRollback((code % 100) as u8)),
            25 => Some(Self::DbCommit((code % 100) as u8)),
            26 => Some(Self::NoInsertId((code % 100) as u8)),
            30 => Some(Self::BadFormat),
            31 => Some(Self::DecimalToF64),
            32 => Some(Self::StringParse((code % 100) as u8)),
            33 => Some(Self::JsonParse((code % 100) as u8)),
            40 => Some(Self::ExchangeConfiguration),
            41 => Some(Self::MissingCreditCurrency((code % 100) as u8)),
            42 => Some(Self::MissingDebitCurrency((code % 100) as u8)),
            43 => Some(Self::MissingFromCurrency((code % 100) as u8)),
            44 => Some(Self::MissingToCurrency((code % 100) as u8)),
            50 => Some(Self::ScriptError((code % 100) as u8)),
            51 => Some(Self::MissingKey((code % 100) as u8)),
            52 => Some(Self::InvalidCipherResponse),
            53 => Some(Self::CipherError),
            54 => Some(Self::TcpConn),
            60 => Some(Self::InvalidProductId((code % 100) as u8)),
            61 => Some(Self::InvalidEntityId),
            62 => Some(Self::InvalidFraudRuleGroup),
            63 => Some(Self::TransactionGroupNotFound((code % 100) as u8)),
            64 => Some(Self::TransactionLogNotFound),
            65 => Some(Self::UnknownBlocksId((code % 100) as u8)),
            66 => Some(Self::UnknownOperation),
            67 => Some(Self::NoBalancesLock((code % 100) as u8)),
            68 => Some(Self::UnknownWalletsId),
            69 => Some(Self::InconsistentTransactionGroup((code % 100) as u8)),
            70 => Some(Self::Encryption),
            71 => Some(Self::RequestError),
            72 => Some(Self::LastLogIdChanged),
            73 => Some(Self::NoCollectingBalances),
            _ => None,
        }
    }
}

impl Serialize for SystemErrorCodes {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
        S: Serializer {
        serializer.serialize_u16(self.code())
    }
}

impl From<&SystemErrorCodes> for ErrorTypes {
    fn from(s: &SystemErrorCodes) -> Self {
        match s {
            SystemErrorCodes::Unknown => ErrorTypes::Generic,
            SystemErrorCodes::UnknownReason => ErrorTypes::Generic,
            SystemErrorCodes::UnReachable(_) => ErrorTypes::UnReachable,
            SystemErrorCodes::ClosedChannel(_) => ErrorTypes::ClosedChannel,
            SystemErrorCodes::BufferFull => ErrorTypes::BufferFull,
            SystemErrorCodes::DbNoConn(_) => ErrorTypes::DbNoConn,
            SystemErrorCodes::DbQuery(_) => ErrorTypes::DbQuery,
            SystemErrorCodes::DbStmt(_) => ErrorTypes::DbStmt,
            SystemErrorCodes::DbTransaction(_) => ErrorTypes::DbTransaction,
            SystemErrorCodes::DbRollback(_) => ErrorTypes::DbRollback,
            SystemErrorCodes::DbCommit(_) => ErrorTypes::DbCommit,
            SystemErrorCodes::NoInsertId(_) => ErrorTypes::NoInsertId,
            SystemErrorCodes::BadFormat => ErrorTypes::BadFormat,
            SystemErrorCodes::DecimalToF64 => ErrorTypes::DecimalToF64,
            SystemErrorCodes::StringParse(_) => ErrorTypes::StringParse,
            SystemErrorCodes::JsonParse(_) => ErrorTypes::JsonParse,
            SystemErrorCodes::ExchangeConfiguration => ErrorTypes::ExchangeConfiguration,
            SystemErrorCodes::MissingCreditCurrency(_) => ErrorTypes::MissingCreditCurrency,
            SystemErrorCodes::MissingDebitCurrency(_) => ErrorTypes::MissingDebitCurrency,
            SystemErrorCodes::MissingFromCurrency(_) => ErrorTypes::Generic,
            SystemErrorCodes::MissingToCurrency(_) => ErrorTypes::Generic,
            SystemErrorCodes::ScriptError(_) => ErrorTypes::ScriptError,
            SystemErrorCodes::MissingKey(_) => ErrorTypes::MissingKey,
            SystemErrorCodes::InvalidCipherResponse => ErrorTypes::InvalidCipherResponse,
            SystemErrorCodes::CipherError => ErrorTypes::CipherError,
            SystemErrorCodes::TcpConn => ErrorTypes::TcpConn,
            SystemErrorCodes::InvalidProductId(_) => ErrorTypes::InvalidProductId,
            SystemErrorCodes::InvalidEntityId => ErrorTypes::InvalidEntityId,
            SystemErrorCodes::InvalidFraudRuleGroup => ErrorTypes::InvalidFraudRuleGroup,
            SystemErrorCodes::TransactionGroupNotFound(_) => ErrorTypes::TransactionNotFound,
            SystemErrorCodes::TransactionLogNotFound => ErrorTypes::TransactionNotFound,
            SystemErrorCodes::UnknownBlocksId(_) => ErrorTypes::UnknownBlocksId,
            SystemErrorCodes::UnknownOperation => ErrorTypes::UnknownOperation,
            SystemErrorCodes::NoBalancesLock(_) => ErrorTypes::NoBalancesLock,
            SystemErrorCodes::UnknownWalletsId => ErrorTypes::UnknownWalletsId,
            SystemErrorCodes::InconsistentTransactionGroup(_) => ErrorTypes::InconsistentTransactionGroup,
            SystemErrorCodes::Encryption => ErrorTypes::Encryption,
            SystemErrorCodes::RequestError => ErrorTypes::CoreRequest,
            SystemErrorCodes::LastLogIdChanged => ErrorTypes::NoBalancesLock,
            SystemErrorCodes::NoCollectingBalances => ErrorTypes::MissingCollectingBalance,
        }
    }
}

pub trait MySystemError {
    fn system_error<T,U>(msg: T, at: U, system_error: SystemErrorCodes) -> Self
        where T:Display, U:Display;
    fn new<T, U>(location: T, detail: U, error_code: ErrorTypes) -> Self
        where T: Display, U: Display;
}

impl MySystemError for crate::utils::CoreError {
    fn system_error<T,U>(msg: T, at: U, system_error: SystemErrorCodes) -> Self
        where T:Display, U:Display
    {
        MyError {
            location: Locations::Core(at.to_string()),
            detail: msg.to_string(),
            error_code: ErrorTypes::from(&system_error),
            critical: false,
            system_error
        }
    }

    fn new<T, U>(location: T, detail: U, error_code: ErrorTypes) -> Self
        where T: Display, U: Display
    {
        Self {
            location: Locations::Core(location.to_string()),
            detail: detail.to_string(),
            error_code,
            critical: false,
            system_error: SystemErrorCodes::Unknown
        }
    }
}