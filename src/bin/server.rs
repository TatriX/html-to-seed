#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use html_to_seed;
use rocket::response::content;

#[get("/")]
fn index() -> content::Html<&'static str> {
    content::Html(include_str!("index.html"))
}


#[post("/convert", format = "text/plain", data = "<html>")]
fn convert(html: String) -> String {
    html_to_seed::format(html_to_seed::convert(html))
}

fn main() {
    rocket::ignite().mount("/", routes![index, convert]).launch();
}
