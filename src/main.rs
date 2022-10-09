use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use serde_json::Value;
use std::error;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Deserialize)]
struct ItineraryParams {
    itinerary: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/calculate", web::get().to(get_itinerary))
            .route("/termini", web::post().to(post_termini))
    });
    println!("Serving on http://localhost:8080/calculate");
    server
        .bind("127.0.0.1:8080")
        .expect("error binding service to address")
        .run()
        .await
}

async fn get_itinerary() -> impl Responder {
    HttpResponse::Ok().content_type("text/html").body(
        r#"
                <title>Itinerary Calculator</title>
                <form action="/termini" method="post">
                <input type="text" name="itinerary"/>
                <button type="submit">Calculate termini</button>
                </form>
            "#,
    )
}

async fn post_termini(form: web::Form<ItineraryParams>) -> impl Responder {
    if form.itinerary.is_empty() {
        return HttpResponse::BadRequest()
            .content_type("text/html")
            .body("Empty itinerary.");
    }

    let res = calculate_itinerary(form.itinerary.clone());
    let response = format!(
        "The terminal locations of {} \
                 are <b>{}</b>\n",
        form.itinerary,
        if res.is_ok() {
            res.unwrap()
        } else {
            "Invalid itinerary".to_string()
        }
    );

    HttpResponse::Ok().content_type("text/html").body(response)
}

fn calculate_itinerary(s: String) -> Result<String> {
    let js_str = s.replace('\'', "\"");
    let js_val: Value = serde_json::from_str(&js_str)?;
    let termini = flightpath::js_itinerary_termini(js_val)?;
    let termini_str = serde_json::to_string(&termini)?;
    Ok(termini_str.replace('\"', "\'"))
}
