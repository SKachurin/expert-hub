use actix_files::NamedFile;
use actix_web::{get, Result};

#[get("/e/{slug}")]
pub async fn expert_public_page() -> Result<NamedFile> {
    Ok(NamedFile::open("./public/expert-public.html")?)
}

#[get("/e/{slug}/edit")]
pub async fn expert_edit_page() -> Result<NamedFile> {
    Ok(NamedFile::open("./public/expert-edit.html")?)
}