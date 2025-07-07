use rs_can::CanError;

pub enum ZLinMode {
    Slave = 0,
    Master = 1,
}

impl TryFrom<u8> for ZLinMode {
    type Error = CanError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Slave),
            1 => Ok(Self::Master),
            _ => Err(CanError::other_error("parameter not supported")),
        }
    }
}

pub enum ZLinDataType {
    TypeData = 0,
    TypeError = 1,
    TypeEvent = 2,
}

impl TryFrom<u8> for ZLinDataType {
    type Error = CanError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::TypeData),
            1 => Ok(Self::TypeError),
            2 => Ok(Self::TypeEvent),
            _ => Err(CanError::other_error("parameter not supported")),
        }
    }
}

pub enum ZLinEventType {
    Wakeup = 1,
    EnterSleep = 2,
    ExitSleep = 3,
}

impl TryFrom<u8> for ZLinEventType {
    type Error = CanError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Wakeup),
            1 => Ok(Self::EnterSleep),
            2 => Ok(Self::ExitSleep),
            _ => Err(CanError::other_error("parameter not supported")),
        }
    }
}

pub enum ZLinCheckSumMode {
    Classic = 1,
    Enhance = 2,
    Auto = 3,
}

impl TryFrom<u8> for ZLinCheckSumMode {
    type Error = CanError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Classic),
            1 => Ok(Self::Enhance),
            2 => Ok(Self::Auto),
            _ => Err(CanError::other_error("parameter not supported")),
        }
    }
}
