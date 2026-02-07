use embassy_stm32::sdmmc::DataBlock;

use crate::sdmmc::{
    logging::{
        DescriptorBlock, LogEntryDescriptor, Message, LOG_ENTRY_DESCRIPTOR_SIZE, MAGIC,
        MESSAGE_SIZE,
    },
    BlockIdx, HypedSdmmc, HypedSdmmcError, BLOCK_SIZE,
};

use dvida_serialize::{DvDeserialize, DvSerialize};

impl HypedSdmmc {
    pub async fn read_logs(&self) -> Result<(), HypedSdmmcError> {
        let descriptor_block = self.get_descriptor_block().await?;

        if descriptor_block.magic != MAGIC {
            return Err(HypedSdmmcError::MagicMismatch);
        }

        if descriptor_block.last_log_start == 0 {
            defmt::info!("No logs available");
            return Err(HypedSdmmcError::NoLogsAvailable);
        }

        defmt::info!("Reading logs from SD card...");
        defmt::info!("Current trial ID: {}", descriptor_block.current_trial_id);

        let mut buffer = [DataBlock([0u8; BLOCK_SIZE]); 1];
        let mut current_log_block = descriptor_block.last_log_start;
        let mut log_entry_count = 0;

        // Traverse the log chain backwards from last to first
        loop {
            // Read the log entry descriptor
            self.read(&mut buffer, BlockIdx(current_log_block)).await?;

            let log_entry =
                LogEntryDescriptor::deserialize(dvida_serialize::Endianness::Little, &buffer[0].0)
                    .map_err(|_| HypedSdmmcError::SerializationError)?
                    .0;

            log_entry_count += 1;
            defmt::info!(
                "--- Log Entry {} (Block {}) ---",
                log_entry_count,
                current_log_block
            );

            // Calculate how many blocks contain messages for this log entry
            let next_block = if log_entry.next_log_start == 0 {
                descriptor_block.first_free_block
            } else {
                log_entry.next_log_start
            };

            let total_blocks = next_block - current_log_block;

            // Read all message blocks for this log entry
            for block_offset in 0..total_blocks {
                buffer[0].0.fill(0);
                self.read(&mut buffer, BlockIdx(current_log_block + block_offset))
                    .await?;

                // First block starts after the descriptor, others start at 0
                let mut offset = if block_offset == 0 {
                    LOG_ENTRY_DESCRIPTOR_SIZE as usize
                } else {
                    0
                };

                // Parse messages from this block
                while offset + MESSAGE_SIZE as usize <= BLOCK_SIZE {
                    match Message::deserialize(
                        dvida_serialize::Endianness::Little,
                        &buffer[0].0[offset..],
                    ) {
                        Ok((msg, _)) => {
                            // Check if message is all zeros (unused space)
                            if msg.time_sec == 0 && msg.time_milli == 0 && msg.time_micro == 0 {
                                break;
                            }

                            defmt::info!(
                                "[{}.{:03}.{:03}] {:?}",
                                msg.time_sec,
                                msg.time_milli,
                                msg.time_micro,
                                msg.message
                            );
                        }
                        Err(_) => break,
                    }
                    offset += MESSAGE_SIZE as usize;
                }
            }

            // Move to previous log entry
            if log_entry.last_log_start == 0 {
                break;
            }
            current_log_block = log_entry.last_log_start;
        }

        defmt::info!("Finished reading {} log entries", log_entry_count);
        Ok(())
    }

    /// Clear all logs by resetting the descriptor block
    pub async fn clear_logs(&self) -> Result<(), HypedSdmmcError> {
        defmt::info!("Clearing all logs...");

        let mut buffer = [DataBlock([0u8; BLOCK_SIZE]); 1];

        // Read current descriptor to preserve trial ID
        self.read(&mut buffer, BlockIdx(0)).await?;

        let mut descriptor_block =
            DescriptorBlock::deserialize(dvida_serialize::Endianness::Little, &buffer[0].0)
                .map_err(|_| HypedSdmmcError::SerializationError)?
                .0;

        // Verify magic
        if descriptor_block.magic != MAGIC {
            return Err(HypedSdmmcError::MagicMismatch);
        }

        // Reset to initial state but keep trial ID
        descriptor_block.first_free_block = 1;
        descriptor_block.last_log_start = 0;

        buffer[0].0.fill(0);
        descriptor_block
            .serialize(dvida_serialize::Endianness::Little, &mut buffer[0].0)
            .map_err(|_| HypedSdmmcError::SerializationError)?;

        self.write(&buffer, BlockIdx(0)).await?;

        defmt::info!("Logs cleared successfully");
        Ok(())
    }
}
