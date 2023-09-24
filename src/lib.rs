#![no_std]
#![feature(async_fn_in_trait)]

#[cfg(feature = "embassy-rp")]
pub mod dshot_embassy_rp;

#[cfg(feature = "rp2040-hal")]
pub mod dshot_rp2040_hal;

pub trait DshotPioTrait<const N: usize> {
    fn reverse(&mut self, reverse: [bool;N]);
    async fn throttle_clamp(&mut self, throttle: [u16;N]) -> [u32; N];
    fn throttle_minimum(&mut self);
}