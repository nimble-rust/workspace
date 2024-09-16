/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/nimble-rust/workspace
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use nimble_protocol::Nonce;
use std::{fmt, io};

#[derive(Eq, Debug, PartialEq)]
pub enum ErrorLevel {
    Info,     // Informative, can be ignored
    Warning,  // Should be logged, but recoverable
    Critical, // Requires immediate attention, unrecoverable
}

#[derive(Debug)]
pub enum ClientError {
    Single(ClientErrorKind),
    Multiple(Vec<ClientErrorKind>),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Single(error) => std::fmt::Display::fmt(&error, f),
            Self::Multiple(errors) => {
                writeln!(f, "Multiple errors occurred:")?;

                for (index, error) in errors.iter().enumerate() {
                    writeln!(f, "{}: {}", index + 1, error)?;
                }

                Ok(())
            }
        }
    }
}

#[derive(Debug)]
pub enum ClientErrorKind {
    Unexpected,
    IoErr(io::Error),
    WrongConnectResponseNonce(Nonce),
    WrongDownloadRequestId,
    DownloadResponseWasUnexpected,
}

impl ClientErrorKind {
    pub fn error_level(&self) -> ErrorLevel {
        match self {
            Self::IoErr(_) => ErrorLevel::Critical,
            Self::WrongConnectResponseNonce(_) => ErrorLevel::Info,
            Self::WrongDownloadRequestId => ErrorLevel::Warning,
            Self::DownloadResponseWasUnexpected => ErrorLevel::Info,
            Self::Unexpected => ErrorLevel::Critical,
        }
    }
}

impl fmt::Display for ClientErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unexpected => {
                write!(f, "Unexpected")
            }
            Self::IoErr(io_err) => {
                write!(f, "io:err {:?}", io_err)
            }
            Self::WrongConnectResponseNonce(nonce) => {
                write!(f, "wrong nonce in reply to connect {:?}", nonce)
            }
            Self::WrongDownloadRequestId => {
                write!(f, "WrongDownloadRequestId")
            }
            Self::DownloadResponseWasUnexpected => {
                write!(f, "DownloadResponseWasUnexpected")
            }
        }
    }
}

impl std::error::Error for ClientErrorKind {} // it implements Debug and Display
impl std::error::Error for ClientError {} // it implements Debug and Display
