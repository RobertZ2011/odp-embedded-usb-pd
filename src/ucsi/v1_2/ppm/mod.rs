use crate::ucsi::v1_2::{cci, CommandHeaderRaw, CommandType, InvalidCommandType, COMMAND_LEN};
use crate::{GlobalPortId, LocalPortId, PortId};

pub mod ack_cc_ci;
pub mod cancel;
pub mod get_capability;
pub mod ppm_reset;
pub mod set_notification_enable;
pub mod state_machine;

/// Commands that only affect the PPM level and don't need to be sent to an LPM
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Command {
    PpmReset,
    Cancel,
    AckCcCi(ack_cc_ci::Args),
    SetNotificationEnable(set_notification_enable::Args),
    GetCapability,
}

impl Command {
    /// Length of a PPM command payload, the command minus its header
    pub const PAYLOAD_LEN: usize = COMMAND_LEN - size_of::<CommandHeaderRaw>();

    /// Returns the command type for this command
    pub const fn command_type(&self) -> CommandType {
        match self {
            Command::PpmReset => CommandType::PpmReset,
            Command::Cancel => CommandType::Cancel,
            Command::AckCcCi(_) => CommandType::AckCcCi,
            Command::SetNotificationEnable(_) => CommandType::SetNotificationEnable,
            Command::GetCapability => CommandType::GetCapability,
        }
    }

    /// Converts this command into its raw payload bytes
    pub fn to_payload(&self) -> [u8; Self::PAYLOAD_LEN] {
        match self {
            Command::PpmReset => bytemuck::must_cast(ppm_reset::ArgsRaw::from(ppm_reset::Args)),
            Command::Cancel => bytemuck::must_cast(cancel::ArgsRaw::from(cancel::Args)),
            Command::AckCcCi(args) => bytemuck::must_cast(ack_cc_ci::ArgsRaw::from(*args)),
            Command::SetNotificationEnable(args) => bytemuck::must_cast(set_notification_enable::ArgsRaw::from(*args)),
            Command::GetCapability => bytemuck::must_cast(get_capability::ArgsRaw::from(get_capability::Args)),
        }
    }

    /// Reconstructs a command from its command type and raw payload bytes
    ///
    /// Returns [`InvalidCommandType`] if `command_type` is not a PPM command.
    pub fn from_payload(
        command_type: CommandType,
        payload: [u8; Self::PAYLOAD_LEN],
    ) -> Result<Self, InvalidCommandType> {
        match command_type {
            CommandType::PpmReset => Ok(Command::PpmReset),
            CommandType::Cancel => Ok(Command::Cancel),
            CommandType::AckCcCi => Ok(Command::AckCcCi(
                bytemuck::must_cast::<_, ack_cc_ci::ArgsRaw>(payload).into(),
            )),
            CommandType::SetNotificationEnable => Ok(Command::SetNotificationEnable(
                bytemuck::must_cast::<_, set_notification_enable::ArgsRaw>(payload).into(),
            )),
            CommandType::GetCapability => Ok(Command::GetCapability),
            _ => Err(InvalidCommandType(command_type as u8)),
        }
    }
}

/// PPM command response data
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ResponseData {
    GetCapability(get_capability::ResponseData),
}

impl ResponseData {
    /// Maximum length in bytes of any PPM response data
    pub const MAX_LEN: usize = get_capability::ResponseDataRaw::LEN;

    /// Returns the command type that produces this response data
    pub const fn command_type(&self) -> CommandType {
        match self {
            ResponseData::GetCapability(_) => CommandType::GetCapability,
        }
    }

    /// Converts this response data into raw bytes
    ///
    /// Returns a [`Self::MAX_LEN`] sized buffer along with the number of valid
    /// bytes at its start.
    pub fn to_bytes(&self) -> ([u8; Self::MAX_LEN], usize) {
        match self {
            ResponseData::GetCapability(data) => (
                bytemuck::must_cast(get_capability::ResponseDataRaw::from(*data)),
                get_capability::ResponseDataRaw::LEN,
            ),
        }
    }

    /// Reconstructs response data from its command type and raw bytes
    ///
    /// Returns [`InvalidCommandType`] if `command_type` is not a PPM command
    /// that produces response data.
    pub fn from_bytes(command_type: CommandType, bytes: [u8; Self::MAX_LEN]) -> Result<Self, InvalidCommandType> {
        match command_type {
            CommandType::GetCapability => Ok(ResponseData::GetCapability(
                bytemuck::must_cast::<_, get_capability::ResponseDataRaw>(bytes).into(),
            )),
            _ => Err(InvalidCommandType(command_type as u8)),
        }
    }
}

/// PPM command response
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Response<T: PortId> {
    /// CCI is produced by every command
    pub cci: cci::Cci<T>,
    /// Response data for the command
    pub data: Option<ResponseData>,
}

pub type GlobalResponse = Response<GlobalPortId>;
pub type LocalResponse = Response<LocalPortId>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ppm_reset_payload() {
        let payload = [0u8; Command::PAYLOAD_LEN];

        assert_eq!(Command::PpmReset.to_payload(), payload);
        assert_eq!(
            Command::from_payload(CommandType::PpmReset, payload),
            Ok(Command::PpmReset)
        );
    }

    #[test]
    fn test_cancel_payload() {
        let payload = [0u8; Command::PAYLOAD_LEN];

        assert_eq!(Command::Cancel.to_payload(), payload);
        assert_eq!(Command::from_payload(CommandType::Cancel, payload), Ok(Command::Cancel));
    }

    #[test]
    fn test_ack_cc_ci_payload() {
        let mut payload = [0u8; Command::PAYLOAD_LEN];
        payload[0] = 0x2; // Ack command complete

        let expected = Command::AckCcCi(ack_cc_ci::Args {
            ack: ack_cc_ci::Ack::from(0x2),
        });

        assert_eq!(expected.to_payload(), payload);
        assert_eq!(Command::from_payload(CommandType::AckCcCi, payload), Ok(expected));
    }

    #[test]
    fn test_set_notification_enable_payload() {
        let mut payload = [0u8; Command::PAYLOAD_LEN];
        payload[0] = 0x1; // Enable command complete notification

        let expected = Command::SetNotificationEnable(set_notification_enable::Args {
            notification_enable: set_notification_enable::NotificationEnable::from(0x1),
        });

        assert_eq!(expected.to_payload(), payload);
        assert_eq!(
            Command::from_payload(CommandType::SetNotificationEnable, payload),
            Ok(expected)
        );
    }

    #[test]
    fn test_get_capability_payload() {
        let payload = [0u8; Command::PAYLOAD_LEN];

        assert_eq!(Command::GetCapability.to_payload(), payload);
        assert_eq!(
            Command::from_payload(CommandType::GetCapability, payload),
            Ok(Command::GetCapability)
        );
    }

    #[test]
    fn test_from_payload_non_ppm_command() {
        let payload = [0u8; Command::PAYLOAD_LEN];

        assert_eq!(
            Command::from_payload(CommandType::GetConnectorStatus, payload),
            Err(InvalidCommandType(CommandType::GetConnectorStatus as u8))
        );
    }

    #[test]
    fn test_response_data_bytes() {
        let data = get_capability::ResponseData {
            num_connectors: 1,
            ..Default::default()
        };
        let expected = ResponseData::GetCapability(data);

        let (bytes, len) = expected.to_bytes();
        assert_eq!(len, ResponseData::MAX_LEN);
        assert_eq!(
            ResponseData::from_bytes(CommandType::GetCapability, bytes),
            Ok(expected)
        );
    }

    #[test]
    fn test_response_data_from_bytes_invalid_command() {
        let bytes = [0u8; ResponseData::MAX_LEN];

        assert_eq!(
            ResponseData::from_bytes(CommandType::PpmReset, bytes),
            Err(InvalidCommandType(CommandType::PpmReset as u8))
        );
    }
}
