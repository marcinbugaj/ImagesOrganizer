mod build_tree;
mod convert_tree;
mod extract_filepath_location;
mod haversine_metric;
mod launch_pipeline_for_directory;
mod to_serde_tree;
mod types;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Result};
use launch_pipeline_for_directory::launch_pipeline_for_directory;
mod static_assets;
use static_assets::*;
mod reorganize;
use actix_files::NamedFile;
use reorganize::*;
use std::path::PathBuf;

use std::{future, path::Path};

use crate::types::Commit;

async fn index(req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = req.match_info().query("absfilepath").parse().unwrap();

    Ok(NamedFile::open(path)?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let compute_clusters_handle = |path: String| {
        let path = Path::new(path.as_str());
        if !path.is_dir() {
            return future::ready(
                HttpResponse::InternalServerError()
                    .body("Not an absolute path to a directory provided."),
            );
        }

        let maybe_tree = launch_pipeline_for_directory(path);
        match maybe_tree {
            Some(tree) => {
                let serialized_tree =
                    serde_json::to_string(&tree).expect("Cannot serialize response to frontend");

                future::ready(HttpResponse::Ok().body(serialized_tree))
            }
            None => {
                future::ready(HttpResponse::InternalServerError().body("Not enough files found."))
            }
        }
    };

    let reorganize_handle = |json: String| {
        let commit: Commit =
            serde_json::from_str(&json).expect("Malformed 'Commit' command from frontend");

        match reorganize(commit) {
            Ok(_) => future::ready(HttpResponse::Ok().body("")),
            Err(e) => future::ready(HttpResponse::InternalServerError().body(e.to_string())),
        }
    };

    let route = move || {
        let r = get_assets().into_iter().fold(
            App::new(),
            |accum, (WebPath(path), FileContent(content))| {
                let handler = move || future::ready(HttpResponse::Ok().body(content.clone()));
                accum.route(path.as_str(), web::get().to(handler))
            },
        );
        r.route("/file/{absfilepath:.*}", web::get().to(index))
            .route("compute_clusters", web::post().to(compute_clusters_handle))
            .route("reorganize", web::post().to(reorganize_handle))
    };

    open::that("http://127.0.0.1:3000/index.html")?;

    HttpServer::new(route)
        .bind(("127.0.0.1", 3000))?
        .run()
        .await
}
