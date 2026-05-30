#![allow(
    non_camel_case_types,
    non_upper_case_globals,
    non_snake_case,
    dead_code
)]
// include!(concat!(env!("OUT_DIR"), "/nican.rs"));
include!("generator/nican.rs");

use crate::NiCanFrame;
use rs_can::{CanDirection, CanError, CanFrame, CanId, Timestamp, TimestampSource, MAX_FRAME_SIZE};

impl Into<NCTYPE_CAN_FRAME> for NiCanFrame {
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
            ArbitrationId: arb_id as NCTYPE_CAN_ARBID,
            IsRemote: if self.is_remote() { 1 } else { 0 },
            DataLength: data_len,
            Data: data.try_into().unwrap(),
        }
    }
}

impl TryInto<NiCanFrame> for NCTYPE_CAN_STRUCT {
    type Error = CanError;

    fn try_into(self) -> Result<NiCanFrame, Self::Error> {
        let is_remote_frame = self.FrameType == NC_FRMTYPE_REMOTE as u8;
        let is_error_frame = self.FrameType == NC_FRMTYPE_COMM_ERR as u8;
        let arb_id = self.ArbitrationId as u32;
        let is_extended = (arb_id & NC_FL_CAN_ARBID_XTD) > 0;
        let dlc = self.DataLength;
        let timestamp = (self.Timestamp.HighPart as u64) << 32 | (self.Timestamp.LowPart as u64);

        if is_error_frame {
            return Err(CanError::InvalidFrame(
                "NI-CAN communication error frame is not supported by NiCanFrame".into(),
            ));
        }

        let id = CanId::from_bits(arb_id, Some(is_extended))?;
        let mut msg = if is_remote_frame {
            NiCanFrame::new_remote(id, dlc)
        } else {
            NiCanFrame::new_can(id, &self.Data[..dlc as usize])
        }
        .map_err(|_| {
            CanError::InvalidFrame(format!("length of data is rather than {}", MAX_FRAME_SIZE))
        })?;

        let filetime_100ns: u128 = timestamp as u128;
        let epoch_diff_100ns: u128 = 11_644_473_600u128 * 10_000_000u128;
        let nanos = filetime_100ns.saturating_sub(epoch_diff_100ns) * 100u128;

        msg.set_direction(CanDirection::Receive)
            .set_timestamp(Some(Timestamp {
                nanos,
                source: TimestampSource::Hardware,
            }));

        Ok(msg)
    }
}
