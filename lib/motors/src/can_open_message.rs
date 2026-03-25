/// Basic structure for a CanOpen Message
#[derive(Debug, PartialEq, Clone)]
pub struct CanOpenMessage {
    pub id: u32,
    pub index: u16,
    pub sub_index: u8,
    pub command: u8,
    pub data: u32,
}

pub mod config_messages {
    use super::CanOpenMessage;
    
    pub const TEST_STEPPER_FREQUENCY: CanOpenMessage = CanOpenMessage {
        id: 0x601,
        command: 0x2B,
        index: 0x2040,
        sub_index: 0x04,
        data: 0x00000014, // Data will be overwritten at runtime depending on the frequency desired
    };

    pub const TEST_STEPPER_ENABLE: CanOpenMessage = CanOpenMessage {
        id: 0x601,
        command: 0x2F,
        index: 0x2040,
        sub_index: 0x09,
        data: 0x00000001,
    };

    pub const RESET_TEST_MODE_COMMAND: CanOpenMessage = CanOpenMessage {
        id: 0x601,
        command: 0x2B,
        index: 0x2031,
        sub_index: 0x00,
        data: 0x00000000,
    };

    pub const TEST_MODE_COMMAND: CanOpenMessage = CanOpenMessage {
        id: 0x601,
        command: 0x2B,
        index: 0x2031,
        sub_index: 0x00,
        data: 0x00000000, // Data will be overwritten at runtime depending on the test mode command desired
    };

    pub const SET_MAX_CURRENT: CanOpenMessage = CanOpenMessage {
        id: 0x601,
        command: 0x23,
        index: 0x2050,
        sub_index: 0x00,
        data: 0x00005208,
    };

    pub const SECONDARY_CURRENT_PROTECTION: CanOpenMessage = CanOpenMessage {
        id: 0x601,
        command: 0x23,
        index: 0x2051,
        sub_index: 0x00,
        data: 0x000186A0,
    };

    pub const MOTOR_RATED_CURRENT: CanOpenMessage = CanOpenMessage {
        id: 0x601,
        command: 0x23,
        index: 0x6075,
        sub_index: 0x00,
        data: 0x00004E20,
    };

    pub const OVERVOLTAGE_LIMIT: CanOpenMessage = CanOpenMessage {
        id: 0x601,
        command: 0x2B,
        index: 0x2054,
        sub_index: 0x00,
        data: 0x000001A4,
    };

    pub const MODES_OF_OPERATION: CanOpenMessage = CanOpenMessage {
        id: 0x601,
        command: 0x2F,
        index: 0x6060,
        sub_index: 0x00,
        data: 0x000000FD,
    };

    pub const SENSOR_TYPE: CanOpenMessage = CanOpenMessage {
        id: 0x601,
        command: 0x2F,
        index: 0x2057,
        sub_index: 0x01,
        data: 0x00000000,
    };

    pub const UNDERVOLTAGE_LIMIT: CanOpenMessage = CanOpenMessage {
        id: 0x601,
        command: 0x2B,
        index: 0x2055,
        sub_index: 0x01,
        data: 0x0000001E,
    };

}

pub mod messages {
    use super::CanOpenMessage;

    pub const ENTER_STOP_STATE: CanOpenMessage = CanOpenMessage {
        id: 0x000,
        command: 0x02,
        index: 0x0000,
        sub_index: 0x00,
        data: 0x00000000,
    };

    pub const ENTER_PREOPERATIONAL_STATE: CanOpenMessage = CanOpenMessage {
        id: 0x000,
        command: 0x80,
        index: 0x0000,
        sub_index: 0x00,
        data: 0x00000000,
    };

    pub const RESET_NODE: CanOpenMessage = CanOpenMessage {
        id: 0x000,
        command: 0x81,
        index: 0x0000,
        sub_index: 0x00,
        data: 0x00000000,
    };

    pub const ENTER_OPERATIONAL_STATE: CanOpenMessage = CanOpenMessage {
        id: 0x000,
        command: 0x01,
        index: 0x0000,
        sub_index: 0x00,
        data: 0x00000000,
    };

    // Data will be overwritten at runtime depending on the frequency desired
    pub const SET_FREQUENCY: CanOpenMessage = CanOpenMessage {
        id: 0x601,
        command: 0x2B,
        index: 0x2040,
        sub_index: 0x04,
        data: 0x00000000,
    };

    pub const SHUTDOWN: CanOpenMessage = CanOpenMessage {
        id: 0x601,
        command: 0x2B,
        index: 0x6040,
        sub_index: 0x00,
        data: 0x00000006,
    };

    pub const SWITCH_ON: CanOpenMessage = CanOpenMessage {
        id: 0x601,
        command: 0x2B,
        index: 0x6040,
        sub_index: 0x00,
        data: 0x00000007,
    };

    pub const START_DRIVE: CanOpenMessage = CanOpenMessage {
        id: 0x601,
        command: 0x2B,
        index: 0x6040,
        sub_index: 0x00,
        data: 0x0000000F,
    };

    pub const QUICK_STOP: CanOpenMessage = CanOpenMessage {
        id: 0x601,
        command: 0x2B,
        index: 0x6040,
        sub_index: 0x00,
        data: 0x00000002,
    };
}
