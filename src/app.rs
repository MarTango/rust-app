use serde_json::Value;
use tide::http::headers::CONTENT_TYPE;
use tide::http::{Error, StatusCode};
use tide::{Request, Response};

use crate::state::State;

pub fn app() -> tide::Server<State> {
    let mut app = tide::with_state(State::new());
    app.at("/:type").post(post);
    app.at("/:type/:id").get(get);
    app
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
