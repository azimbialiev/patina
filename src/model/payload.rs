use crate::model::reason_code::ReasonCode;
use crate::model::topic::TopicFilter;
use crate::model::variable_header::Property;

#[derive(Debug)]
#[derive(Clone)]
pub struct Payload {
    client_id: Option<String>,
    will_properties: Option<Vec<Property>>,
    will_topic: Option<String>,
    will_payload: Option<Vec<u8>>,
    username: Option<String>,
    password: Option<String>,
    topic_filters: Option<Vec<TopicFilter>>,
    reason_codes: Option<Vec<ReasonCode>>,
    data: Option<Vec<u8>>,
}

impl Payload {
    pub fn client_id_opt(&self) -> Option<&String> {
        self.client_id.as_ref()
    }
    pub fn client_id(&self) -> &String {
        self.client_id.as_ref().expect("client_id")
    }
    pub fn topic_filters(&self) -> &Vec<TopicFilter> {
        self.topic_filters.as_ref().unwrap()
    }
    pub fn reason_codes(&self) -> &Vec<ReasonCode> {
        self.reason_codes.as_ref().unwrap()
    }
    pub fn data(&self) -> &Vec<u8> {
        self.data.as_ref().unwrap()
    }
}

impl Payload {
    pub fn from_connect(client_id: Option<String>, will_properties: Option<Vec<Property>>, will_topic: Option<String>, will_payload: Option<Vec<u8>>, username: Option<String>, password: Option<String>) -> Self {
        Payload {
            client_id,
            will_properties,
            will_topic,
            will_payload,
            username,
            password,
            topic_filters: None,
            reason_codes: None,
            data: None,
        }
    }

    pub fn from_sub_unsub(topic_filters: Vec<TopicFilter>) -> Self {
        Payload {
            client_id: None,
            will_properties: None,
            will_topic: None,
            will_payload: None,
            username: None,
            password: None,
            topic_filters: Some(topic_filters),
            reason_codes: None,
            data: None,
        }
    }
    pub fn from_sub_unsub_ack(reason_codes: Option<Vec<ReasonCode>>) -> Self {
        Payload {
            client_id: None,
            will_properties: None,
            will_topic: None,
            will_payload: None,
            username: None,
            password: None,
            topic_filters: None,
            reason_codes,
            data: None,
        }
    }

    pub fn from_publish(data: Option<Vec<u8>>) -> Self {
        Payload {
            client_id: None,
            will_properties: None,
            will_topic: None,
            will_payload: None,
            username: None,
            password: None,
            topic_filters: None,
            reason_codes: None,
            data,
        }
    }
}