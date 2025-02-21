/// Type-C current
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Current {
    /// Default USB current
    #[default]
    UsbDefault,
    /// 1.5A
    Current1A5,
    /// 3A0
    Current3A0,
}

impl Current {
    /// Returns the current in mA
    pub fn to_ma(self, is_usb2: bool) -> u16 {
        match self {
            Current::UsbDefault => {
                if is_usb2 {
                    500
                } else {
                    900
                }
            }
            Current::Current1A5 => 1500,
            Current::Current3A0 => 3000,
        }
    }
}
