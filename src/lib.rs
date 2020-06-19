use std::collections::HashMap;

use async_std::sync::{Arc, Mutex};

use serde_json::Value;

use tide::http::headers::CONTENT_TYPE;
use tide::http::{Error, StatusCode};
use tide::{Request, Response};

pub fn app() -> tide::Server<State> {
    let mut app = tide::with_state(State::new());
    app.at("/:type").post(post);
    app.at("/:type/:id").get(get);
    app
}

#[derive(Debug)]
/// In-memory key-Vec store
pub struct State {
    /// Map from collection_name to collection of json
    ///
    /// Someone please let me know if there's a way to do it without
    /// using Arc and Mutex.
    collections: Arc<Mutex<HashMap<String, Vec<Value>>>>,
}

impl State {
    fn new() -> Self {
        State {
            collections: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    /// Insert `payload` into the `typ` collection
    ///
    async fn insert(&self, typ: &String, payload: Value) -> Result<(), Error> {
        let payload = match payload {
            Value::Object(x) => Value::Object(x),
            _ => return Err(Error::from_str(StatusCode::BadRequest, "Bad payload")),
        };

        let mut collections = self.collections.lock().await;
        if let Some(x) = collections.get_mut(typ) {
            x.push(payload);
        } else {
            collections.insert(typ.clone(), vec![payload]);
        }
        Ok(())
    }

    /// Get the `id`-th record from the `typ` collection
    async fn get(&self, typ: &String, id: usize) -> Option<Value> {
        let collections = self.collections.lock().await;
        if let Some(c) = collections.get(typ) {
            if let Some(v) = c.get(id) {
                return Some(v.clone());
            }
        }
        None
    }
}

/// Endpoint handler to fetch an entity
async fn get(req: Request<State>) -> Result<Response, Error> {
    let typ = req.param("type")?;
    let id = match req.param::<usize>("id") {
        Ok(i) => i,
        Err(_) => return Ok(Response::new(StatusCode::BadRequest)),
    };

    let stored = match req.state().get(&typ, id).await {
        Some(thing) => thing,
        None => {
            return Err(Error::from_str(
                StatusCode::NotFound,
                format!("{} with id {} not found", typ, id),
            ))
        }
    };

    Ok(Response::new(StatusCode::Ok)
        .append_header(CONTENT_TYPE, "application/json")
        .body_json(&stored)?)
}

/// Handler to store a new entity in the state
async fn post(mut req: Request<State>) -> Result<Response, Error> {
    let typ = req.param("type")?;
    let insertion: Value = match req.body_json().await {
        Ok(Value::Object(i)) => Value::Object(i),
        _ => {
            return Ok(Response::new(StatusCode::BadRequest));
        }
    };

    let optimistic_response = Response::new(StatusCode::Ok)
        .append_header(CONTENT_TYPE, "application/json")
        .body_json(&insertion)?;

    req.state().insert(&typ, insertion).await?;

    Ok(optimistic_response)
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
