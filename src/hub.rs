use super::session::Session;
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

pub enum HubAction {
    Add {
        session: Arc<Session>,
    },
    Remove {
        session: Arc<Session>,
    },
    Subscribe {
        session_id: String,
        stream: String,
        identifier: String,
    },
    Unsubscribe {
        session_id: String,
        identifier: String,
    },
    Broadcast {
        stream: String,
        data: serde_json::Value,
    },
    Shutdown,
}

pub struct Hub {
    // sender: UnboundedSender<HubAction>,
    receiver: UnboundedReceiver<HubAction>,
    sessions: HashMap<String, Arc<Session>>,
    identifiers: HashMap<String, HashMap<String, bool>>,
    streams: HashMap<String, HashMap<String, String>>,
    session_streams: HashMap<String, HashMap<String, Vec<String>>>,
}

impl Hub {
    /// Create a hub and return a sender to send data to Hub
    pub fn new() -> UnboundedSender<HubAction> {
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(Hub::start(rx));

        tx
    }

    async fn start(rx: UnboundedReceiver<HubAction>) {
        let mut hub = Hub {
            receiver: rx,
            sessions: HashMap::new(),
            identifiers: HashMap::new(),
            streams: HashMap::new(),
            session_streams: HashMap::new(),
        };

        while let Some(action) = hub.receiver.next().await {
            match action {
                HubAction::Add { session } => {
                    println!("[HUB] - add session");
                    hub.add_session(session);
                }
                HubAction::Remove { session } => {
                    println!("[HUB] - remove session");
                    hub.remove_session(session);
                }
                HubAction::Subscribe {
                    session_id,
                    stream,
                    identifier,
                } => {
                    println!("[HUB] - subscribe stream {}", stream);
                    hub.subscribe_session(session_id, stream, identifier);
                }
                HubAction::Unsubscribe {
                    session_id,
                    identifier,
                } => {
                    println!(
                        "[HUB] - unsubscribe session {} from {}",
                        session_id, identifier
                    );
                    hub.unsubscribe_from_channel(session_id, identifier);
                }
                HubAction::Broadcast { stream, data } => {
                    println!(
                        "[HUB] - broadcast called to {} with {:?}",
                        stream,
                        data.as_str().unwrap()
                    );
                    hub.broadcast(stream, data.as_str().unwrap()).await;
                }
                HubAction::Shutdown => break,
            }
        }
    }

    /// Adds an active session to the Hub
    fn add_session(&mut self, session: Arc<Session>) {
        self.sessions.insert(session.uid.clone(), session.clone());
        self.identifiers
            .entry(session.identifiers.clone())
            .or_insert(HashMap::new());

        self.identifiers
            .get_mut(&session.identifiers)
            .unwrap()
            .insert(session.uid.clone(), true);
    }

    fn remove_session(&mut self, session: Arc<Session>) {
        if !self.sessions.contains_key(&session.uid) {
            println!("[HUB] - Session hasn't been registered");
            return;
        }

        self.unsubscribe_from_channels(session.uid.clone());

        self.sessions.remove(&session.uid);
        self.identifiers
            .get_mut(&session.identifiers)
            .unwrap()
            .remove(&session.uid);
    }

    fn subscribe_session(&mut self, session_id: String, stream: String, identifier: String) {
        self.streams.entry(stream.clone()).or_insert(HashMap::new());

        self.streams
            .get_mut(&stream)
            .unwrap()
            .insert(session_id.clone(), identifier.clone());

        self.session_streams
            .entry(session_id.clone())
            .or_insert(HashMap::new());

        let session_streams = self.session_streams.get_mut(&session_id).unwrap();
        session_streams
            .entry(identifier.clone())
            .or_insert(Vec::new());
        session_streams.get_mut(&identifier).unwrap().push(stream);
    }

    fn unsubscribe_from_channels(&mut self, session_id: String) {
        self.session_streams
            .get_mut(&session_id)
            .unwrap()
            .clone()
            .iter()
            .for_each(|(key, _)| self.unsubscribe_from_channel(session_id.clone(), key.clone()));
        self.session_streams.remove(&session_id);
    }

    fn unsubscribe_from_channel(&mut self, session_id: String, identifier: String) {
        if !self.session_streams.contains_key(&session_id) {
            return;
        }

        let streams = self
            .session_streams
            .get(&session_id)
            .unwrap()
            .get(&identifier)
            .unwrap()
            .clone();

        streams.iter().for_each(|stream_name| {
            self.streams
                .get_mut(stream_name)
                .unwrap()
                .remove(&session_id);
        });
    }

    pub async fn broadcast(&self, stream: String, data: &str) {
        if !self.streams.contains_key(&stream) {
            println!("[HUB] - No sessions subscribed to stream {}", stream);
            return;
        }

        for (key, val) in self.streams.get(&stream).unwrap().iter() {
            if !self.sessions.contains_key(key) {
                continue;
            }

            let session = self.sessions.get(key).unwrap();
            session
                .clone()
                .send_broadcast(data, val.clone())
                .await
                .expect("Error when broadcasting to a session");
        }
    }
}
