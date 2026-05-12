use actix_web::{get, HttpResponse, Responder};

#[get("/dvp/widget.js")]
pub async fn serve_dvp_widget() -> impl Responder {
    let js = include_str!("../../static/dvp-widget.js");

    HttpResponse::Ok()
        .content_type("application/javascript; charset=utf-8")
        .insert_header(("Cache-Control", "public, max-age=3600"))
        .insert_header(("Access-Control-Allow-Origin", "*"))
        .body(js)
}
