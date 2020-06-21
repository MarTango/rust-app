use async_std::sync::RwLock;
use std::collections::HashMap;

use serde_json::Value;
use tide::http::{Error, StatusCode};

#[derive(Debug)]
/// In-memory key-Vec store
pub struct State {
    /// Map from collection_name to collection of json
    ///
    /// Someone please let me know if there's a way to do it without
    /// using Arc and Mutex.
    collections: RwLock<HashMap<String, Vec<Value>>>,
}

impl State {
    pub fn new() -> Self {
        State {
            collections: RwLock::new(HashMap::new()),
        }
    }
    /// Insert `payload` into the `typ` collection
    ///
    pub async fn insert(&self, typ: &String, payload: Value) -> Result<(), Error> {
        let payload = match payload {
            Value::Object(x) => Value::Object(x),
            _ => return Err(Error::from_str(StatusCode::BadRequest, "Bad payload")),
        };

        let mut collections = self.collections.write().await;
        let collection = collections.entry(typ.to_string()).or_insert(vec![]);
        collection.push(payload);
        Ok(())
    }

    /// Get the `id`-th record from the `typ` collection
    pub async fn get(&self, typ: &String, id: usize) -> Option<Value> {
        Some(self.collections.read().await.get(typ)?.get(id)?.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[async_std::test]
    async fn test_state_insert_then_get() {
        let state = State::new();
        let typ = String::from("people");
        state.insert(&typ, json!({"sup": "boyy"})).await.unwrap();
        let person: Value = state.get(&typ, 0).await.unwrap();
        assert_eq!(person, json!({"sup": "boyy"}));
    }
}
