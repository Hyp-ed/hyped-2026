extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{self, Generics};

#[proc_macro_derive(HypedUart)]
pub fn hyped_uart_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_hyped_uart(&ast)
}

fn impl_hyped_uart(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generics: &Generics = &ast.generics;
    let (impl_generics, ty_generics, _) = generics.split_for_impl();
    let gen = quote! {
        impl #impl_generics HypedUart for #name #ty_generics {
            async fn read(&mut self, buffer: &mut [u8]) -> Result<(), UartErr> {
                self.uart
                    .lock()
                    .await
                    .read(buffer)
                    .await
                    .map_err(Self::to_hyped_uart_err)
            }

            async fn write(&mut self, buffer: &[u8]) -> Result<(), UartErr> {
                self.uart
                    .lock()
                    .await
                    .write(buffer)
                    .await
                    .map_err(Self::to_hyped_uart_err)
            }

            async fn flush(&mut self) -> Result<(), UartErr> {
                self.uart
                    .lock()
                    .await
                    .blocking_flush()
                    .map_err(Self::to_hyped_uart_err)
            }
        }

        impl #impl_generics #name #ty_generics {
            pub fn new(
                uart: &'d embassy_sync::mutex::Mutex<CriticalSectionRawMutex, Uart<'static, Async>>,
            ) -> Self {
                Self { uart }
            }

            fn to_hyped_uart_err(e: embassy_stm32::usart::Error) -> UartErr {
                match e {
                    embassy_stm32::usart::Error::Noise => UartErr::Noise,
                    embassy_stm32::usart::Error::Overrun => UartErr::Overrun,
                    embassy_stm32::usart::Error::Parity => UartErr::Parity,
                    embassy_stm32::usart::Error::BufferTooLong => UartErr::BufferTooLong,
                    embassy_stm32::usart::Error::Framing => UartErr::Framing,
                    _ => UartErr::Unknown,
                }
            }
        }
    };
    gen.into()
}
