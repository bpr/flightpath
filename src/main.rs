use actix_web::{web, App, HttpResponse, HttpServer};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
struct ItineraryParams {
    itinerary: String,
}

fn main() {
    let server = HttpServer::new(|| {
        App::new()
        .route("/calculate", web::get().to(get_itinerary))
        .route("/endpoints", web::post().to(post_endpoints))
    });
    println!("Serving on http://localhost:8080");
    server
        .bind("127.0.0.1:8080").expect("error binding service to address")
        .run().expect("error running server");
}

fn get_itinerary() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(
            r#"
                <title>Itinerary Calculator</title>
                <form action="/endpoints" method="post">
                <input type="text" name="itinerary"/>
                <button type="submit">Compute start and end</button>
                </form>
            "#,
        )
}

fn post_endpoints(form: web::Form<ItineraryParams>) -> HttpResponse {
    if form.itinerary.len() == 0 {
        return HttpResponse::BadRequest()
            .content_type("text/html")
            .body("Empty itinerary.");
    }

    let response =
        format!("The start and end locations of {} \
                 is <b>{}</b>\n",
                form.itinerary,
                calculate_itinerary(form.itinerary.clone()));

    HttpResponse::Ok()
        .content_type("text/html")
        .body(response)
}


fn calculate_itinerary(s: String) -> String {
    if s.len() == 0 || s.len() == 2 {
        return "Empty itinerary".to_string();
    }
    let js_str = s.replace("'", "\"");
    let js_val: Value = serde_json::from_str(&js_str).unwrap();
    let endpoints = flightpath::js_itinerary_endpoints(js_val).unwrap();
    let endpoints_str = serde_json::to_string(&endpoints).unwrap();
    endpoints_str.replace("\"", "'")
}