use crate::model::qos_level::QoSLevel;

#[derive(Debug)]
#[derive(Clone)]
#[derive(Eq, PartialEq)]
pub enum RetainHandling {
    SendRetainedMessagesOnSubscribe,
    SendRetainedMessagesOnNewSubscribe,
    DontSendRetainedMessages,
}

impl RetainHandling {
    pub fn from_u8(value: u8) -> Option<RetainHandling> {
        let retain_handling = match value {
            0 => { RetainHandling::SendRetainedMessagesOnSubscribe }
            1 => { RetainHandling::SendRetainedMessagesOnNewSubscribe }
            2 => { RetainHandling::DontSendRetainedMessages }
            _ => { return None; }
        };
        Some(retain_handling)
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct TopicFilter {
    topic_filter: String,
    maximum_qos: QoSLevel,
    no_local: bool,
    retain_as_published: bool,
    retain_handling: RetainHandling,
    reserved_bits: Vec<bool>,
}

impl TopicFilter {
    pub fn from_subscribe(topic_filter: String, maximum_qos: QoSLevel, no_local: bool, retain_as_published: bool, retain_handling: RetainHandling, reserved_bits: Vec<bool>) -> Self {
        TopicFilter { topic_filter, maximum_qos, no_local, retain_as_published, retain_handling, reserved_bits }
    }
    pub fn from_unsubscribe(topic_filter: String) -> Self {
        TopicFilter { topic_filter, maximum_qos: QoSLevel::ExactlyOnce, no_local: true, retain_as_published: false, retain_handling: RetainHandling::DontSendRetainedMessages, reserved_bits: vec![] }
    }
    pub fn topic_filter(&self) -> &String {
        return &self.topic_filter;
    }
}