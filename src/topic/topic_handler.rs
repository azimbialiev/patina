use std::sync::Arc;
use dashmap::{DashMap, DashSet};

use log::{debug, trace};

#[derive(Debug)]
pub struct TopicHandler {
    topic2subscribers: Arc<DashMap<String, DashSet<String>>>,
}

impl Default for TopicHandler {
    fn default() -> Self {
        Self { topic2subscribers: Arc::new(DashMap::new()) }
    }
}

impl TopicHandler {
    pub fn subscribe(&self, client_id: &String, topic_filter: &String) {
        trace!("Adding subscriber {:?} to {:?}", client_id, topic_filter);

        if let Some(mut subscribers) = self.topic2subscribers.get_mut(topic_filter) {
            subscribers.insert(client_id.to_owned());
        } else {
            let mut subscribers = DashSet::new();
            subscribers.insert(client_id.to_owned());
            self.topic2subscribers.insert(topic_filter.to_owned(), subscribers);
        }
    }

    pub fn unsubscribe(&self, client_id: &String, topic_filter: &String) {
        self.topic2subscribers.retain(|topic, mut subscribers| {
            if topic.eq(topic_filter) {
                trace!("Unsubscribing client {:?} from topic {:?}", client_id, topic_filter);
                subscribers.retain(|s| { s.ne(client_id) });
            }
            true
        });
    }

    pub fn unsubscribe_all(&self, client_id: &String) {
        self.topic2subscribers.retain(|topic, subscribers| {
            trace!("Unsubscribing client {:?} from topic {:?}", client_id, topic);
            subscribers.retain(|s| s.ne(client_id));
            true
        });
    }

    pub fn find_subscribers(&self, topic_filter: &String) -> Vec<String> {
        trace!("Finding subscribers for topic {:?} ", topic_filter);
        if let Some(subscribers) = self.topic2subscribers.get(topic_filter) {
            trace!("Found {:?} subscribers for topic {:?}", subscribers, topic_filter);
            subscribers.clone().into_iter()
                .map(|s| { s.clone() })
                .collect::<Vec<String>>()
        } else {
            Vec::with_capacity(0)
        }
    }
}