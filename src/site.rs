use axum::{
    body::{boxed, Body, BoxBody},
    http::{Request, Response, StatusCode, Uri},
};
use tower::util::ServiceExt;
use tower_http::services::ServeDir;

pub async fn index(_: Uri) -> Result<Response<BoxBody>, (StatusCode, String)> {
    // println!("root uri: {:?}", uri);
    get_static_file("/index.html".parse().unwrap()).await
}

pub async fn static_file_server(uri: Uri) -> Result<Response<BoxBody>, (StatusCode, String)> {
    println!("file_handler uri: {:?}", uri);
    let res = get_static_file(uri.clone()).await?;
    println!("{:?}", res);

    // allows retry with `.html` extension if desired (it isn't)
    if res.status() == StatusCode::NOT_FOUND {
        Err((StatusCode::NOT_FOUND, "Not Found".to_string()))
        // try with `.html`
        // TODO: handle if the Uri has query parameters
        // match format!("{}.html", uri).parse() {
        //     Ok(uri_html) => get_static_file(uri_html).await,
        //     Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Invalid URI".to_string())),
        // }
    } else {
        Ok(res)
    }
}

async fn get_static_file(uri: Uri) -> Result<Response<BoxBody>, (StatusCode, String)> {
    // println!("get_static_file uri: {:?}", uri);
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();

    // `ServeDir` implements `tower::Service` so we can call it with `tower::ServiceExt::oneshot`
    // When run normally, the root is the workspace root
    match ServeDir::new("./static").oneshot(req).await {
        Ok(res) => Ok(res.map(boxed)),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", err),
        )),
    }
}
