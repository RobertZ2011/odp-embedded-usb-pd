use core::future::Future;

use embedded_hal_async::delay::DelayNs;

use crate::Error;

pub trait PdController {
    type BusError;

    fn reset(&mut self, delay: &mut impl DelayNs) -> impl Future<Output = Result<(), Error<Self::BusError>>>;
}
