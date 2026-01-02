use actix_files::Files;
use actix_web::web;

pub fn scope() -> actix_web::Scope {
    web::scope("/assets").service(Files::new("", "assets").show_files_listing())
}
