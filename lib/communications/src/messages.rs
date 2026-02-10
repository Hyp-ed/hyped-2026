use hyped_can::HypedCanFrame;
//use hyped_state_machine::states::State;

use crate::{boards::Board, emergency::Reason};

use super::{
    can_id::CanId,
    data::{CanData, CanDataType},
    heartbeat::Heartbeat,
    measurements::MeasurementReading,
    message_identifier::MessageIdentifier,
    //state_transition::StateTransitionRequest,
};

#[derive(PartialEq, Debug, Clone, defmt::Format)]
pub enum CanMessage {
    MeasurementReading(MeasurementReading),
    //StateTransitionCommand(StateTransitionCommand),
    //StateTransitionRequest(StateTransitionRequest),
    Heartbeat(Heartbeat),
    Emergency(Board, Reason),
}

// Converts a CanMessage into a HypedCanFrame ready to be sent over the CAN bus
impl From<CanMessage> for HypedCanFrame {
    fn from(val: CanMessage) -> Self {
        match val {
            CanMessage::MeasurementReading(measurement_reading) => {
                let message_identifier =
                    MessageIdentifier::Measurement(measurement_reading.measurement_id);
                let can_id = CanId::new(
                    measurement_reading.board,
                    measurement_reading.reading.into(),
                    message_identifier,
                );
                HypedCanFrame::new(can_id.into(), measurement_reading.reading.into())
            }
            // CanMessage::StateTransitionCommand(state_transition) => {
            //     let can_id = CanId::new(
            //         state_transition.from_board,
            //         CanDataType::State,
            //         MessageIdentifier::StateTransitionCommand,
            //     );
            //     HypedCanFrame::new(
            //         can_id.into(),
            //         CanData::State(state_transition.to_state.into()).into(),
            //     )
            // }
            // CanMessage::StateTransitionRequest(state_transition) => {
            //     let can_id = CanId::new(
            //         state_transition.requesting_board,
            //         CanDataType::State,
            //         MessageIdentifier::StateTransitionRequest,
            //     );
            //     HypedCanFrame::new(
            //         can_id.into(),
            //         CanData::State(state_transition.to_state.into()).into(),
            //     )
            // }
            CanMessage::Heartbeat(heartbeat) => {
                let can_id = CanId::new(
                    heartbeat.from,
                    CanDataType::Heartbeat,
                    MessageIdentifier::Heartbeat,
                );
                HypedCanFrame::new(can_id.into(), CanData::Heartbeat(heartbeat.to).into())
            }
            CanMessage::Emergency(board, reason) => {
                let can_id =
                    CanId::new(board, CanDataType::Emergency, MessageIdentifier::Emergency);
                HypedCanFrame::new(can_id.into(), CanData::Emergency(reason).into())
            }
        }
    }
}

// Converts an incoming HypedCanFrame read from the CAN bus into a CanMessage
impl From<HypedCanFrame> for CanMessage {
    fn from(frame: HypedCanFrame) -> Self {
        let can_id: CanId = frame.can_id.into();
        let message_identifier = can_id.message_identifier;
        let board = can_id.board;

        match message_identifier {
            MessageIdentifier::Measurement(measurement_id) => {
                let reading: CanData = frame.data.into();
                let measurement_reading = MeasurementReading {
                    reading,
                    board,
                    measurement_id,
                };
                CanMessage::MeasurementReading(measurement_reading)
            }
            // MessageIdentifier::StateTransitionCommand => {
            //     let reading: CanData = frame.data.into();
            //     match reading {
            //         CanData::State(state) => {
            //             //let to_state: State = state.try_into().expect("Invalid State!");
            //             //let state_transition = StateTransitionCommand::new(board, to_state);
            //             //CanMessage::StateTransitionCommand(state_transition)
            //         }
            //         _ => panic!("Invalid CanData for StateTransition"),
            //     }
            // }
            // // MessageIdentifier::StateTransitionRequest => {
            //     let reading: CanData = frame.data.into();
            //     match reading {
            //         CanData::State(state) => {
            //             //let to_state: State = state.try_into().expect("Invalid State!");
            //             //let state_transition = StateTransitionRequest::new(board, to_state);
            //             //CanMessage::StateTransitionRequest(state_transition)
            //         }
            //         _ => panic!("Invalid CanData for StateTransitionRequest"),
            //     }
            // }
            MessageIdentifier::Heartbeat => {
                let reading: CanData = frame.data.into();
                match reading {
                    CanData::Heartbeat(to) => {
                        let heartbeat = Heartbeat::new(to, board);
                        CanMessage::Heartbeat(heartbeat)
                    }
                    _ => panic!("Invalid CanData for Heartbeat"),
                }
            }
            MessageIdentifier::Emergency => {
                let reading: CanData = frame.data.into();
                match reading {
                    CanData::Emergency(reason) => CanMessage::Emergency(board, reason),
                    _ => panic!("Invalid CanData for Emergency"),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use hyped_can::HypedCanFrame;
    use hyped_core::config::MeasurementId;

    use crate::{
        boards::Board, data::CanData, emergency::Reason, heartbeat::Heartbeat,
        measurements::MeasurementReading, messages::CanMessage,
    };

    #[test]
    fn can_message_round_trip_measurement() {
        let measurement_reading = MeasurementReading::new(
            CanData::F32(1.25),
            Board::Telemetry,
            MeasurementId::Acceleration,
        );
        let can_message = CanMessage::MeasurementReading(measurement_reading);

        let can_frame: HypedCanFrame = can_message.clone().into();
        let decoded: CanMessage = can_frame.into();

        assert_eq!(can_message, decoded)
    }

    #[test]
    fn can_message_round_trip_heartbeat() {
        let heartbeat = CanMessage::Heartbeat(Heartbeat::new(Board::Mqtt, Board::Telemetry));
        let can_frame: HypedCanFrame = heartbeat.clone().into();
        let decoded: CanMessage = can_frame.into();
        assert_eq!(heartbeat, decoded)
    }

    #[test]
    fn can_message_round_trip_emergency() {
        let message = CanMessage::Emergency(Board::Navigation, Reason::MissingHeartbeat);
        let can_frame: HypedCanFrame = message.clone().into();
        let decoded: CanMessage = can_frame.into();
        assert_eq!(message, decoded)
    }
}
