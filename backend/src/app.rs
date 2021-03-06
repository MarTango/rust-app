use serde_json::Value;
use tide::http::{mime, Error, StatusCode};
use tide::{Body, Request, Response};

use crate::state::State;

pub fn app() -> tide::Server<()> {
    let mut api = tide::with_state(State::new());
    api.at("/:type").post(post);
    api.at("/:type/:id").get(get);

    let mut root = tide::new();
    root.at("/").get(|_req| async {
        let mut resp = Response::new(StatusCode::Ok);
        resp.set_content_type(mime::HTML);
        resp.set_body(Body::from_file("frontend/index.html").await.unwrap());

        Ok(resp)
    });
    root.at("/dist/:file").get(|req: Request<()>| async move {
        let fp: String = req.param("file")?;
        let mut resp = Response::new(StatusCode::Ok);
        resp.set_body(
            Body::from_file(format!("frontend/dist/{}", fp))
                .await
                .unwrap(),
        );

        Ok(resp)
    });
    root.at("/api").nest(api);
    root
}

/// Endpoint handler to fetch an entity
async fn get(req: Request<State>) -> Result<Response, Error> {
    let typ = req.param("type")?;
    let id = req
        .param::<usize>("id")
        .map_err(|e| Error::new(StatusCode::BadRequest, e))?;

    let stored = req.state().get(&typ, id).await.ok_or(Error::from_str(
        StatusCode::NotFound,
        format!("{} with id {} not found", typ, id),
    ))?;

    let mut res = Response::new(StatusCode::Ok);
    // res.append_header(CONTENT_TYPE, "application/json");
    res.set_content_type(mime::JSON);
    res.set_body(Body::from_json(&stored)?);

    Ok(res)
}

/// Handler to store a new entity in the state
async fn post(mut req: Request<State>) -> Result<Response, Error> {
    let typ = req.param("type")?;
    let insertion: Value = req.body_json().await.map_err(|e| Error::from(e))?;

    let mut optimistic_response = Response::new(StatusCode::Ok);
    optimistic_response.set_content_type(mime::JSON);
    optimistic_response.set_body(Body::from_json(&insertion)?);

    req.state().insert(&typ, insertion).await?;

    Ok(optimistic_response)
}
