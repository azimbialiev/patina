use std::borrow::Borrow;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::ops::{DerefMut, Index};
use std::str::Split;
use tokio::sync::{broadcast, RwLock};
use std::sync::{Arc, RwLockWriteGuard};
use log::{trace, info, debug, warn, error};
use dashmap::DashMap;
use dashmap::mapref::one::{Ref, RefMut};

lazy_static! {

    static ref topic2subscribers: DashMap<String, Vec<String>> = {
        let map = DashMap::new();
        map
    };
}

#[derive(Debug)]
pub struct TopicHandler {

}

impl TopicHandler {
    pub fn new() -> Self {
        TopicHandler {  }
    }
}

impl TopicHandler {
    pub fn find_subscribers(&self, topic_path: &String) -> Vec<String> {
        trace!("TopicHandler::find_subscribers");
        // let mut topic_levels = topic_path.split("/");
        let subscribers = topic2subscribers.get(topic_path);
        return match subscribers {
            None => { Vec::with_capacity(0) }
            Some(subscribers) => { subscribers.value().clone() }
        };
    }

    pub fn add_subscriber(&self, client_id: &String, topic_filter: &String) {
        trace!("TopicHandler::add_subscriber");

        let subscribers = topic2subscribers.get_mut(topic_filter);

        match topic2subscribers.contains_key(topic_filter) {
            true => {
                let mut subscribers = topic2subscribers.get_mut(topic_filter).unwrap();
                subscribers.push(client_id.clone());
            }
            false => {
                topic2subscribers.insert(topic_filter.clone(), vec![client_id.clone()]);
            }
        };
        debug!("Added subscriber {:?} to {:?}", client_id, topic_filter);
    }

    pub fn remove_subscriber(&self, client_id: &String, topic_filter: &String){
        trace!("TopicHandler::remove_subscriber");

        match topic2subscribers.get_mut(topic_filter){
            None => {}
            Some(mut subscribers) => {
                subscribers.retain(|subscriber| subscriber != client_id);
            }
        }
    }

    pub fn remove_all_subscriptions(&self, client_id: &String){
        trace!("TopicHandler::remove_all_subscriptions");

        topic2subscribers.alter_all(|key, mut value|{
            value.retain(|subscriber| subscriber != client_id);
            value
        });
    }

}