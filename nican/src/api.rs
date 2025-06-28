#![allow(
    non_camel_case_types,
    non_upper_case_globals,
    non_snake_case,
    dead_code
)]
// include!(concat!(env!("OUT_DIR"), "/nican.rs"));
include!("generator/nican.rs");

use crate::CanMessage;
use rs_can::{CanDirect, CanError, CanFrame, CanId, MAX_FRAME_SIZE};

impl Into<NCTYPE_CAN_FRAME> for CanMessage {
    fn into(self) -> NCTYPE_CAN_FRAME {
        let mut arb_id = self.id().as_raw();
        if self.is_extended() {
            arb_id |= NC_FL_CAN_ARBID_XTD;
        }

        let data_len = self.data().len() as u8;
        let mut data = self.data().to_vec();
        if data.len() < MAX_FRAME_SIZE {
            data.resize(MAX_FRAME_SIZE, Default::default());
        }

        NCTYPE_CAN_FRAME {
            ArbitrationId: arb_id,
            IsRemote: if self.is_remote() { 1 } else { 0 },
            DataLength: data_len,
            Data: data.try_into().unwrap(),
        }
    }
}

impl TryInto<CanMessage> for NCTYPE_CAN_STRUCT {
    type Error = CanError;

    fn try_into(self) -> Result<CanMessage, Self::Error> {
        let is_remote_frame = self.FrameType == NC_FRMTYPE_REMOTE as u8;
        let is_error_frame = self.FrameType == NC_FRMTYPE_COMM_ERR as u8;
        let arb_id = self.ArbitrationId;
        let is_extended = (arb_id & NC_FL_CAN_ARBID_XTD) > 0;
        let dlc = self.DataLength;
        let timestamp = (self.Timestamp.HighPart as u64) << 32 | (self.Timestamp.LowPart as u64);

        let mut msg = if is_remote_frame {
            CanMessage::new_remote(CanId::from_bits(arb_id, Some(is_extended)), dlc as usize)
        } else {
            CanMessage::new(
                CanId::from_bits(arb_id, Some(is_extended)),
                self.Data.as_slice(),
            )
        }
        .ok_or(CanError::OtherError(format!(
            "length of data is rather than {}",
            MAX_FRAME_SIZE
        )))?;

        msg.set_direct(CanDirect::Receive)
            .set_timestamp(Some(
                (1000. * (timestamp as f64 / 10000000. - 11644473600.)) as u64,
            ))
            .set_error_frame(is_error_frame);

        Ok(msg)
    }
}
