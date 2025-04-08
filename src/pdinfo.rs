//! Types for the `pdinfo` command to query the PD controller for information on the current port and status.

/// The status for the switches on the power path.
#[derive(Clone, Copy)]
pub struct PowerPathStatus(inner::PowerPathStatus);

impl PowerPathStatus {
    /// Creates a new [`PowerPathStatus`] instance with the given status bits.
    pub fn new(ext_vbus_sw_en: bool, int_vbus_sw_en: bool) -> Self {
        Self(inner::PowerPathStatus::new(
            ext_vbus_sw_en.into(),
            int_vbus_sw_en.into(),
        ))
    }

    /// Returns true if the external VBUS switch is enabled.
    pub fn ext_vbus_sw_en(&self) -> bool {
        self.0.ext_vbus_sw_en() == 1
    }

    /// Returns true if the internal VBUS switch is enabled.
    pub fn int_vbus_sw_en(&self) -> bool {
        self.0.int_vbus_sw_en() == 1
    }
}

impl core::fmt::Debug for PowerPathStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Implementing Debug manually to avoid the `.0: 123` field in the bitfield debug impl.
        f.debug_struct("PowerPathStatus")
            .field("ext_vbus_sw_en", &self.ext_vbus_sw_en())
            .field("int_vbus_sw_en", &self.int_vbus_sw_en())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for PowerPathStatus {
    fn format(&self, fmt: defmt::Formatter) {
        // Implementing Format manually to avoid the `.0: 123` field in the bitfield debug impl.
        defmt::write!(
            fmt,
            "PowerPathStatus {{ ext_vbus_sw_en: {}, int_vbus_sw_en: {} }}",
            self.ext_vbus_sw_en(),
            self.int_vbus_sw_en()
        )
    }
}

/// The Alternate Mode the controller is in.
///
/// Alternate Modes are vendor-defined therefore the meaning of the bits is not defined in the
/// USB-C spec. Each driver should define the meaning of the bits in the context of the driver, or
/// provide a way to convert this type to a more specific type.
#[derive(Clone, Copy)]
pub struct AltMode(inner::AltMode);

impl core::fmt::Debug for AltMode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Implementing Debug manually to avoid the `.0: 123` field in the bitfield debug impl.
        f.debug_struct("AltMode")
            .field("status1", &self.status1())
            .field("status2", &self.status2())
            .field("status3", &self.status3())
            .field("status4", &self.status4())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for AltMode {
    fn format(&self, fmt: defmt::Formatter) {
        // Implementing Format manually to avoid the `.0: 123` field in the bitfield debug impl.
        defmt::write!(
            fmt,
            "AltMode {{ status1: {}, status2: {}, status3: {}, status4: {} }}",
            self.status1(),
            self.status2(),
            self.status3(),
            self.status4()
        )
    }
}

impl AltMode {
    /// Creates a new [`AltMode`] instance with the given status bits.
    pub fn new(status1: bool, status2: bool, status3: bool, status4: bool) -> Self {
        Self(inner::AltMode::new(
            status1.into(),
            status2.into(),
            status3.into(),
            status4.into(),
        ))
    }

    /// Returns true if the first status bit is set.
    pub fn status1(&self) -> bool {
        self.0.status1() == 1
    }

    /// Returns true if the second status bit is set.
    pub fn status2(&self) -> bool {
        self.0.status2() == 1
    }

    /// Returns true if the third status bit is set.
    pub fn status3(&self) -> bool {
        self.0.status3() == 1
    }

    /// Returns true if the fourth status bit is set.
    pub fn status4(&self) -> bool {
        self.0.status4() == 1
    }
}

mod inner {
    use bitfield::bitfield;

    bitfield! {
        #[derive(Clone, Copy)]
        pub struct PowerPathStatus(u8);
        impl new;
        u8;
        pub ext_vbus_sw_en, set_ext_vbus_sw_en: 0, 0;
        pub int_vbus_sw_en, set_int_vbus_sw_en: 1, 1;
    }

    bitfield! {
        #[derive(Clone, Copy)]
        pub struct AltMode(u8);
        impl new;
        u8;
        pub status1, set_status1: 0, 0;
        pub status2, set_status2: 1, 1;
        pub status3, set_status3: 2, 2;
        pub status4, set_status4: 3, 3;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    extern crate std;

    #[test]
    fn power_path_status_debug_contains_type_and_field_names() {
        let status = PowerPathStatus(inner::PowerPathStatus(0b0000_0001));
        let debug = std::format!("{:?}", status);
        assert!(debug.contains("PowerPathStatus"));
        assert!(debug.contains("ext_vbus_sw_en:"));
        assert!(debug.contains("int_vbus_sw_en:"));
    }

    #[test]
    fn alt_mode_debug_contains_type_and_field_names() {
        let status = AltMode(inner::AltMode(0b0000_0001));
        let debug = std::format!("{:?}", status);
        assert!(debug.contains("AltMode"));
        assert!(debug.contains("status1:"));
        assert!(debug.contains("status2:"));
        assert!(debug.contains("status3:"));
        assert!(debug.contains("status4:"));
    }
}
