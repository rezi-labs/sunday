use actix_files::Files;

pub fn files() -> Files {
    Files::new("/assets", "assets").show_files_listing()
}
