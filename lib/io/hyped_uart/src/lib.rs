#![no_std]

use core::future::Future;

pub enum UartErr {
    Framing,
    Noise,
    Overrun,
    Parity,
    BufferTooLong,
}

pub trait HypedUart: Sized {
    fn write(buffer: &[u8]) -> impl Future<Output = Result<(), UartErr>> + Send;
    fn read(buffer: &mut [u8]) -> impl Future<Output = Result<(), UartErr>> + Send;
    fn flush() -> impl Future<Output = Result<(), UartErr>> + Send;
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
