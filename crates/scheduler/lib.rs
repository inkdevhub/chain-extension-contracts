#![cfg_attr(not(feature = "std"), no_std)]

use ink::prelude::vec::Vec;

use ink::env::{DefaultEnvironment, Environment};
use scale::{Decode, Encode};
use sp_weights::Weight;

type Balance = <DefaultEnvironment as Environment>::Balance;
type AccountId = <DefaultEnvironment as Environment>::AccountId;
type BlockNumber = <DefaultEnvironment as Environment>::BlockNumber;

pub struct SchedulerExtension;

impl SchedulerExtension {
    pub fn schedule(
        origin: Origin,
        when: BlockNumber,
        maybe_periodic: Option<(BlockNumber, u32)>,
        priority: u8,
        call_input: ContractCallInput,
    ) -> Result<(), SchedulerError> {
        ::ink::env::chain_extension::ChainExtensionMethod::build(0x30001)
            .input::<(
                Origin,
                BlockNumber,
                Option<(BlockNumber, u32)>,
                u8,
                ContractCallInput,
            )>()
            .output::<Result<(), SchedulerError>, true>()
            .handle_error_code::<SchedulerError>()
            .call(&(origin, when, maybe_periodic, priority, call_input))
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum SchedulerError {
    /// Failed to schedule a call
    FailedToSchedule = 1,
    /// Cannot find the scheduled call.
    NotFound = 2,
    /// Given target block number is in the past.
    TargetBlockNumberInPast = 3,
    /// Reschedule failed because it does not change scheduled time.
    RescheduleNoChange = 4,
    /// Attempt to use a non-named function on a named task.
    Named = 5,
    /// Origin Caller is not supported
    OriginCannotBeCaller = 98,
    /// Unknown error
    RuntimeError = 99,
    /// Unknow status code
    UnknownStatusCode,
    /// Encountered unexpected invalid SCALE encoding
    InvalidScaleEncoding,
}

impl ink::env::chain_extension::FromStatusCode for SchedulerError {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(Self::FailedToSchedule),
            2 => Err(Self::NotFound),
            3 => Err(Self::TargetBlockNumberInPast),
            4 => Err(Self::RescheduleNoChange),
            5 => Err(Self::Named),
            98 => Err(Self::OriginCannotBeCaller),
            99 => Err(Self::RuntimeError),
            _ => Err(Self::UnknownStatusCode),
        }
    }
}
impl From<scale::Error> for SchedulerError {
    fn from(_: scale::Error) -> Self {
        SchedulerError::InvalidScaleEncoding
    }
}

#[derive(Clone, Copy, Decode, Encode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub enum Origin {
    Caller,
    Address,
}

impl Default for Origin {
    fn default() -> Self {
        Self::Address
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct ContractCallInput {
    pub dest: AccountId,
    pub data: Vec<u8>,
    pub gas_limit: Weight,
    pub storage_deposit_limit: Option<Balance>,
    pub value: Balance,
    pub max_weight: u64,
}
