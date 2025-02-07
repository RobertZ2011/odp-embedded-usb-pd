//! Request data object as defined in the USB PD specification 6.4.2
use bitfield::bitfield;

use super::{ApdoKind, Common, PdoKind};
use crate::pdo::*;

/// Request data object type
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Rdo {
    /// Fixed
    Fixed(FixedVarData),
    /// Variable
    Variable(FixedVarData),
    /// Battery
    Battery(BatteryData),
    /// PPS
    Pps(PpsData),
    /// AVS
    Avs(AvsData),
}

impl Rdo {
    /// Create a new RDO from the raw data and the corresponding PDO
    pub fn for_pdo(rdo: u32, pdo: impl Common) -> Self {
        match pdo.kind() {
            PdoKind::Fixed => Rdo::Fixed(FixedVarRaw(rdo).into()),
            PdoKind::Variable => Rdo::Variable(FixedVarRaw(rdo).into()),
            PdoKind::Battery => Rdo::Battery(BatteryRaw(rdo).into()),
            PdoKind::Augmented => match pdo.apdo_kind().unwrap() {
                ApdoKind::SprPps => Rdo::Pps(PpsRaw(rdo).into()),
                ApdoKind::EprAvs => Rdo::Pps(PpsRaw(rdo).into()),
                ApdoKind::SprAvs => Rdo::Avs(AvsRaw(rdo).into()),
            },
        }
    }
}

bitfield! {
    /// Fixed and variable RDO raw data
    #[derive(Copy, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub struct FixedVarRaw(u32);
    impl Debug;

    /// Object position
    pub u8, object_position, set_object_position: 31, 28;
    /// Capability mismatch
    pub u8, capability_mismatch, set_capability_mismatch: 26, 26;
    /// USB communications capable
    pub u8, usb_comm_capable, set_usb_comm_capable: 25, 25;
    /// No USB suspend
    pub u8, no_usb_suspend, set_no_usb_suspend: 24, 24;
    /// Unchunked extended messages supported
    pub u8, unchunked_extended_messages_support, set_unchunked_extended_messages_support: 23, 23;
    /// EPR capable
    pub u8, epr_capable, set_epr_capable: 22, 22;
    /// Operating current in 10mA units
    pub u16, operating_current, set_operating_current: 19, 10;
    /// Max operating current in 10mA units
    pub u16, max_operating_current, set_max_operating_current: 9, 0;
}

/// Fixed and variable RDO data
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FixedVarData {
    /// Object position
    pub object_position: u8,
    /// Capability mismatch
    pub capability_mismatch: bool,
    /// USB communications capable
    pub usb_comm_capable: bool,
    /// No USB suspend
    pub no_usb_suspend: bool,
    /// Unchunked extended messages supported
    pub unchunked_extended_messages_support: bool,
    /// EPR capable
    pub epr_capable: bool,
    /// Operating current in mA
    pub operating_current_ma: u16,
    /// Max operating current in mA
    pub max_operating_current_ma: u16,
}

impl From<FixedVarRaw> for FixedVarData {
    fn from(raw: FixedVarRaw) -> Self {
        FixedVarData {
            object_position: raw.object_position(),
            capability_mismatch: raw.capability_mismatch() != 0,
            usb_comm_capable: raw.usb_comm_capable() != 0,
            no_usb_suspend: raw.no_usb_suspend() != 0,
            unchunked_extended_messages_support: raw.unchunked_extended_messages_support() != 0,
            epr_capable: raw.epr_capable() != 0,
            operating_current_ma: raw.operating_current() * MA10_UNIT,
            max_operating_current_ma: raw.max_operating_current() * MA10_UNIT,
        }
    }
}

bitfield! {
    /// Battery RDO raw data
    #[derive(Copy, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub struct BatteryRaw(u32);
    impl Debug;

    /// Object position
    pub u8, object_position, set_object_position: 31, 28;
    /// Capability mismatch
    pub u8, capability_mismatch, set_capability_mismatch: 26, 26;
    /// USB communications capable
    pub u8, usb_comm_capable, set_usb_comm_capable: 25, 25;
    /// No USB suspend
    pub u8, no_usb_suspend, set_no_usb_suspend: 24, 24;
    /// Unchunked extended messages supported
    pub u8, unchunked_extended_messages_support, set_unchunked_extended_messages_support: 23, 23;
    /// EPR capable
    pub u8, epr_capable, set_epr_capable: 22, 22;
    /// Operating power in 250mW units
    pub u16, operating_power, set_operating_power: 19, 10;
    /// Max operating power in 250mW units
    pub u16, max_operating_power, set_max_operating_power: 9, 0;
}

/// Battery RDO data
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BatteryData {
    /// Object position
    pub object_position: u8,
    /// Capability mismatch
    pub capability_mismatch: bool,
    /// USB communications capable
    pub usb_comm_capable: bool,
    /// No USB suspend
    pub no_usb_suspend: bool,
    /// Unchunked extended messages supported
    pub unchunked_extended_messages_support: bool,
    /// EPR capable
    pub epr_capable: bool,
    /// Operating power in mW
    pub operating_power_mw: u16,
    /// Max operating power in mW
    pub max_operating_power_mw: u16,
}

impl From<BatteryRaw> for BatteryData {
    fn from(raw: BatteryRaw) -> Self {
        BatteryData {
            object_position: raw.object_position(),
            capability_mismatch: raw.capability_mismatch() != 0,
            usb_comm_capable: raw.usb_comm_capable() != 0,
            no_usb_suspend: raw.no_usb_suspend() != 0,
            unchunked_extended_messages_support: raw.unchunked_extended_messages_support() != 0,
            epr_capable: raw.epr_capable() != 0,
            operating_power_mw: raw.operating_power() * MW250_UNIT,
            max_operating_power_mw: raw.max_operating_power() * MW250_UNIT,
        }
    }
}

bitfield! {
    /// PPS RDO raw data
    #[derive(Copy, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub struct PpsRaw(u32);
    impl Debug;

    /// Object position
    pub u8, object_position, set_object_position: 31, 28;
    /// Capability mismatch
    pub u8, capability_mismatch, set_capability_mismatch: 26, 26;
    /// USB communications capable
    pub u8, usb_comm_capable, set_usb_comm_capable: 25, 25;
    /// No USB suspend
    pub u8, no_usb_suspend, set_no_usb_suspend: 24, 24;
    /// Unchunked extended messages supported
    pub u8, unchunked_extended_messages_support, set_unchunked_extended_messages_support: 23, 23;
    /// EPR capable
    pub u8, epr_capable, set_epr_capable: 22, 22;
    /// Output voltage in 20mV units
    pub u16, output_voltage, set_output_voltage: 20, 9;
    /// Operating current in 50mA units
    pub u16, operating_current, set_operating_current: 6, 0;
}

/// PPS RDO data
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PpsData {
    /// Object position
    pub object_position: u8,
    /// Capability mismatch
    pub capability_mismatch: bool,
    /// USB communications capable
    pub usb_comm_capable: bool,
    /// No USB suspend
    pub no_usb_suspend: bool,
    /// Unchunked extended messages supported
    pub unchunked_extended_messages_support: bool,
    /// EPR capable
    pub epr_capable: bool,
    /// Output voltage in mV
    pub output_voltage_mv: u16,
    /// Operating current in mA
    pub operating_current_ma: u16,
}

impl From<PpsRaw> for PpsData {
    fn from(raw: PpsRaw) -> Self {
        PpsData {
            object_position: raw.object_position(),
            capability_mismatch: raw.capability_mismatch() != 0,
            usb_comm_capable: raw.usb_comm_capable() != 0,
            no_usb_suspend: raw.no_usb_suspend() != 0,
            unchunked_extended_messages_support: raw.unchunked_extended_messages_support() != 0,
            epr_capable: raw.epr_capable() != 0,
            output_voltage_mv: raw.output_voltage() * MV20_UNIT,
            operating_current_ma: raw.operating_current() * MA50_UNIT,
        }
    }
}

bitfield! {
    /// AVS RDO raw data
    #[derive(Copy, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub struct AvsRaw(u32);
    impl Debug;

    /// Object position
    pub u8, object_position, set_object_position: 31, 28;
    /// Capability mismatch
    pub u8, capability_mismatch, set_capability_mismatch: 26, 26;
    /// USB communications capable
    pub u8, usb_comm_capable, set_usb_comm_capable: 25, 25;
    /// No USB suspend
    pub u8, no_usb_suspend, set_no_usb_suspend: 24, 24;
    /// Unchunked extended messages supported
    pub u8, unchunked_extended_messages_support, set_unchunked_extended_messages_support: 23, 23;
    /// EPR capable
    pub u8, epr_capable, set_epr_capable: 22, 22;
    /// Output voltage in 20mV units
    pub u16, output_voltage, set_output_voltage: 20, 9;
    /// Operating current in 50mA units
    pub u16, operating_current, set_operating_current: 6, 0;
}

/// AVS RDO data
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AvsData {
    /// Object position
    pub object_position: u8,
    /// Capability mismatch
    pub capability_mismatch: bool,
    /// USB communications capable
    pub usb_comm_capable: bool,
    /// No USB suspend
    pub no_usb_suspend: bool,
    /// Unchunked extended messages supported
    pub unchunked_extended_messages_support: bool,
    /// EPR capable
    pub epr_capable: bool,
    /// Output voltage in mV
    pub output_voltage_mv: u16,
    /// Operating current in mA
    pub operating_current_ma: u16,
}

impl From<AvsRaw> for AvsData {
    fn from(raw: AvsRaw) -> Self {
        AvsData {
            object_position: raw.object_position(),
            capability_mismatch: raw.capability_mismatch() != 0,
            usb_comm_capable: raw.usb_comm_capable() != 0,
            no_usb_suspend: raw.no_usb_suspend() != 0,
            unchunked_extended_messages_support: raw.unchunked_extended_messages_support() != 0,
            epr_capable: raw.epr_capable() != 0,
            output_voltage_mv: raw.output_voltage() * MV20_UNIT,
            operating_current_ma: raw.operating_current() * MA50_UNIT,
        }
    }
}
