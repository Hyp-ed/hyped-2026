#![no_std]

use core::future::Future;

pub use hyped_uart_derive::HypedUart;

#[derive(defmt::Format, Debug)]
pub enum UartErr {
    Framing,
    Noise,
    Overrun,
    Parity,
    BufferTooLong,
    Unknown,
    CrcError,
    BufferOverflow,
}

pub trait HypedUart: Sized {
    fn write(&mut self, buffer: &[u8]) -> impl Future<Output = Result<(), UartErr>> + Send;
    fn read(&mut self, buffer: &mut [u8]) -> impl Future<Output = Result<(), UartErr>> + Send;
    fn flush(&mut self) -> impl Future<Output = Result<(), UartErr>> + Send;
}
//
// pub mod mock_uart {
//     /// A mock UART instance which can be used for testing
//     pub struct MockUart {}
//
//     impl crate::HypedUart for MockUart {
//         async fn write(_data: &[u8]) -> Result<(), crate::UartErr> {
//             todo!()
//         }
//
//         async fn read(_buffer: &mut [u8]) -> Result<(), crate::UartErr> {
//             todo!()
//         }
//
//         async fn flush() -> Result<(), crate::UartErr> {
//             todo!()
//         }
//     }
//
//     impl MockUart {
//         #[allow(clippy::new_without_default)]
//         pub fn new() -> Self {
//             Self {}
//         }
//     }
// }
