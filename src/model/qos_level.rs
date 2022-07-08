#[derive(Debug)]
#[derive(Copy, Clone)]
#[derive(Eq, PartialEq)]
pub enum QoSLevel {
    AtMostOnce,
    AtLeastOnce,
    ExactlyOnce,
}

impl QoSLevel {
    pub fn from_u8(value: u8) -> Option<QoSLevel> {
        return match value {
            0 => {
                Option::from(QoSLevel::AtMostOnce)
            }
            1 => {
                Option::from(QoSLevel::AtLeastOnce)
            }
            2 => {
                Option::from(QoSLevel::ExactlyOnce)
            }
            _ => {
                None
            }
        };
    }

    pub fn to_bool(&self) -> (bool, bool) {
        return match self {
            QoSLevel::AtMostOnce => { (false, false) }
            QoSLevel::AtLeastOnce => { (false, true) }
            QoSLevel::ExactlyOnce => { (true, false) }
        };
    }
}