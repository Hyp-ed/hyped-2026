use dvida_serialize::{DvDeErr, DvDeSer, DvDeserialize, DvSerErr, DvSerialize, Endianness};

/// magic is "LOG DISC"
pub const MAGIC: [u8; 8] = [76, 79, 71, 32, 68, 69, 83, 67];
pub const LOG_ENTRY_DESCRIPTOR_SIZE: u32 = 256;
pub const MESSAGE_SIZE: u32 = 256;
pub const MESSAGE_SIZE_RAW: usize = (MESSAGE_SIZE - 24) as usize;

#[derive(DvDeSer, Clone)]
pub struct DescriptorBlock {
    pub magic: [u8; 8],
    pub current_trial_id: u32,
    pub first_free_block: u32,
    /// set to 0 if there is none
    pub last_log_start: u32,
    // reserved until 512
}

impl Default for DescriptorBlock {
    fn default() -> Self {
        DescriptorBlock {
            magic: MAGIC,
            current_trial_id: 0,
            first_free_block: 1,
            last_log_start: 0,
        }
    }
}

#[derive(DvDeSer)]
pub struct LogEntryDescriptor {
    /// set to 0 if it's the first log entry
    pub last_log_start: u32,
    // set to 0 if it's the last log entry
    pub next_log_start: u32,
    // reserved until 256
}

#[derive(DvDeSer)]
pub struct Message {
    pub time_sec: u64,
    pub time_milli: u64,
    pub time_micro: u64,
    pub message: [u8; MESSAGE_SIZE_RAW],
}

pub struct LogBufWriter {
    pub ind: usize,
    pub buf: [u8; MESSAGE_SIZE_RAW],
}

impl Default for LogBufWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl LogBufWriter {
    pub fn new() -> Self {
        LogBufWriter {
            ind: 0,
            buf: [0; MESSAGE_SIZE_RAW],
        }
    }
}

impl core::fmt::Write for LogBufWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.as_bytes().iter().for_each(|i| {
            if self.ind >= MESSAGE_SIZE_RAW {
                return;
            }

            self.buf[self.ind] = *i;
            self.ind += 1;
        });

        Ok(())
    }
}

#[doc(hidden)]
#[allow(unused_unsafe, unused)]
pub fn _to_msg_buf(writer: &mut LogBufWriter, args: core::fmt::Arguments) {
    use core::fmt::Write;

    writer.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! send_log {
    ($sender:ident, $($arg:tt)*) => {
        if let Some(sender) = $sender {
            let mut writer = LogBufWriter::new();
            $crate::sdmmc::logging::_to_msg_buf(&mut writer, format_args!($($arg)*));
            sender.send(writer.buf).await;
        }
    };
}
