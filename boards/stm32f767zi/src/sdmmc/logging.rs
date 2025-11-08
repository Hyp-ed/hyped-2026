use dvida_serialize::{DvDeErr, DvDeSer, DvDeserialize, DvSerErr, DvSerialize, Endianness};

/// magic is "LOG DISC"
pub const MAGIC: [u8; 8] = [76, 79, 71, 32, 68, 69, 83, 67];
pub const LOG_ENTRY_DESCRIPTOR_SIZE: u32 = 128;
pub const MESSAGE_SIZE: u32 = 64;

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
    // reserved until 128
}

#[derive(DvDeSer)]
pub struct Message {
    pub time_sec: u64,
    pub time_milli: u64,
    pub time_micro: u64,
    pub message: [u8; 40],
}
