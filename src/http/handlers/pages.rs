use actix_files::NamedFile;
use actix_web::{get, web, Result};

#[get("/e/{slug}")]
pub async fn expert_public_page(_path: web::Path<String>) -> Result<NamedFile> {
    NamedFile::open_async("./public/expert-public.html")
        .await
        .map_err(Into::into)
}

#[get("/e/{slug}/edit")]
pub async fn expert_edit_page(_path: web::Path<String>) -> Result<NamedFile> {
    NamedFile::open_async("./public/expert-edit.html")
        .await
        .map_err(Into::into)
}