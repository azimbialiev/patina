use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use log::{debug, trace};
use tokio::sync::mpsc::Receiver;
use tokio::sync::Mutex;

#[derive(Debug)]
pub enum TopicCommand {
    Subscribe {
        client_id: String,
        topic_filter: String,
    },
    Unsubscribe {
        client_id: String,
        topic_filter: String,
    },
    UnsubscribeAll {
        client_id: String,
    },
    FindSubscribers {
        topic_filter: String,
        callback: tokio::sync::oneshot::Sender<Vec<String>>,
    },
}

#[derive(Debug)]
pub struct TopicHandler {}

impl TopicHandler {}

impl TopicHandler {
    pub async fn handle_topics(mut broker2topic_handler: Receiver<TopicCommand>) {
        debug!("TopicHandler::handle_topics");
        let topic2subscribers_mutex: Arc<Mutex<HashMap<String, HashSet<String>>>> = Arc::new(Mutex::new(HashMap::new()));
        loop {
            match broker2topic_handler.recv().await {
                None => {}
                Some(topic_command) => {
                    trace!("TopicCommand: {:?}", topic_command);
                    let mut topic2subscribers = topic2subscribers_mutex.lock().await;
                    match topic_command {
                        TopicCommand::Subscribe { client_id, topic_filter } => {
                            trace!("Adding subscriber {:?} to {:?}", client_id, topic_filter);

                            match topic2subscribers.contains_key(&topic_filter) {
                                true => {
                                    let subscribers = topic2subscribers.get_mut(&topic_filter).unwrap();
                                    subscribers.insert(client_id);
                                }
                                false => {
                                    let mut subscribers = HashSet::new();
                                    subscribers.insert(client_id);
                                    topic2subscribers.insert(topic_filter.clone(), subscribers);
                                }
                            };
                        }
                        TopicCommand::Unsubscribe { client_id, topic_filter } => {
                            if let Some(mut subscribers) = topic2subscribers.remove(&topic_filter) {
                                trace!("Unsubscribing client {:?} from topic {:?}", client_id, topic_filter);
                                subscribers.retain(|t| t.ne(&client_id));
                                topic2subscribers.insert(topic_filter, subscribers);
                            }
                        }
                        TopicCommand::UnsubscribeAll { client_id } => {
                            for (topic, subscribers) in topic2subscribers.iter_mut() {
                                trace!("Unsubscribing client {:?} from topic {:?}", client_id, topic);
                                subscribers.retain(|s| s.ne(&client_id));
                            }
                        }
                        TopicCommand::FindSubscribers { topic_filter, callback } => {
                            trace!("Finding subscribers for topic {:?} ", topic_filter);
                            if let Some(subscribers) = topic2subscribers.get(&topic_filter) {
                                trace!("Found {:?} subscribers for topic {:?}", subscribers, topic_filter);
                                callback.send(subscribers.into_iter()
                                    .map(|s| { s.clone() })
                                    .collect::<Vec<String>>())
                                    .expect("panic callback send");
                            } else {
                                callback.send(Vec::with_capacity(0)).expect("panic callback send");
                            }
                        }
                    }
                }
            }
        }
    }
}