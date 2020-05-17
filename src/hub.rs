use super::session::Session;
use std::collections::HashMap;
use std::sync::Arc;

pub struct Hub {
    sessions: HashMap<String, Arc<Session>>,
    identifiers: HashMap<String, HashMap<String, bool>>,
}

impl Hub {
    pub fn new() -> Hub {
        Hub {
            sessions: HashMap::new(),
            identifiers: HashMap::new(),
        }
    }

    fn add_session(&mut self, session: Arc<Session>) {
        self.sessions.insert(session.uid.clone(), session.clone());
        self.identifiers
            .entry(session.identifiers.clone())
            .or_insert(HashMap::new());
    }
}