use std::borrow::BorrowMut;
use std::collections::HashSet;
use std::ops::Deref;
use std::sync::Arc;
use dashmap::{DashMap, DashSet};
use metered::{*};
use log::{debug, trace};

#[derive(Debug)]
pub struct TopicHandler {
    topic2subscribers: Arc<DashMap<String, HashSet<String>>>,
    pub(crate) metrics: TopicHandlerMetrics,
}

impl Default for TopicHandler {
    fn default() -> Self {
        Self { topic2subscribers: Arc::new(DashMap::new()), metrics: TopicHandlerMetrics::default() }
    }
}

#[metered(registry = TopicHandlerMetrics)]
impl TopicHandler {
    #[measure([HitCount, Throughput, InFlight, ResponseTime])]
    pub fn subscribe(&self, client_id: &String, topic_filter: &String) {
        trace!("Adding subscriber {:?} to {:?}", client_id, topic_filter);

        if self.topic2subscribers.contains_key(topic_filter) {
            let mut subscribers = self.topic2subscribers.get_mut(topic_filter).unwrap();
            if subscribers.iter().filter(|s: &&String| { s.to_owned().eq(client_id) }).count() == 0 {
                subscribers.insert(client_id.to_owned());
            }
        } else {
            let mut subscribers = HashSet::with_capacity(100);
            subscribers.insert(client_id.to_owned());
            self.topic2subscribers.insert(topic_filter.to_owned(), subscribers);
        }
    }

    #[measure([HitCount, Throughput, InFlight, ResponseTime])]
    pub fn unsubscribe(&self, client_id: &String, topic_filter: &String) {
        self.topic2subscribers.alter_all(|topic, subscribers| {
            if topic.eq(topic_filter) {
                trace!("Unsubscribing client {:?} from topic {:?}", client_id, topic_filter);
                subscribers.to_owned()
                    .into_iter()
                    .filter(|s| { s.deref().ne(client_id) })
                    .collect()
            } else {
                subscribers
            }
        });
    }

    #[measure([HitCount, Throughput, InFlight, ResponseTime])]
    pub fn unsubscribe_all(&self, client_id: &String) {
        self.topic2subscribers.alter_all(|topic, subscribers| {
            trace!("Unsubscribing client {:?} from topic {:?}", client_id, topic);
            subscribers
                .into_iter()
                .filter(|s: &String| s.to_owned().ne(client_id))
                .collect()
        });
    }

    #[measure([HitCount, Throughput, InFlight, ResponseTime])]
    pub fn find_subscribers(&self, topic_filter: &String) -> Vec<String> {
        trace!("Finding subscribers for topic {:?} ", topic_filter);
        if let Some(subscribers) = self.topic2subscribers.get(topic_filter) {
            trace!("Found {:?} subscribers for topic {:?}", subscribers, topic_filter);
            subscribers.iter()
                .map(|s| { s.clone() })
                .collect()
        } else {
            Vec::with_capacity(0)
        }
    }
}