#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate serde_derive;

pub mod git;

use std::collections::HashMap;
use std::thread;

use actix_web::{error, middleware, web, App, Error, HttpResponse, HttpServer};
use futures::channel::oneshot;
use git::count_repo_files;
use git::Commit;
use tera::Tera;

async fn get_repo(
    url: &'static str,
    range: &'static str,
    patterns: Vec<&'static str>,
) -> Result<Vec<Commit>, Error> {
    let (sender, receiver) = oneshot::channel::<Vec<Commit>>();

    thread::spawn(move || {
        sender
            .send(count_repo_files(url, range, patterns).unwrap())
            .unwrap();
    });

    Ok(receiver.await?)
}

async fn index(templates: web::Data<tera::Tera>) -> Result<HttpResponse, Error> {
    // Fetch all repos in parallel
    let (norne, sbanken, arena, sben) = futures::join!(
        get_repo(
            "http://gitlab.osl.manamind.com/customers/norne.git",
            "50fba560..origin/develop",
            vec!["\\.jsx?$", "\\.tsx?$"],
        ),
        get_repo(
            "http://gitlab.osl.manamind.com/customers/skbn.git",
            "952bc527..origin/develop",
            vec!["\\.jsx?$", "\\.tsx?$"],
        ),
        get_repo(
            "http://gitlab.osl.manamind.com/customers/arena.git",
            "d71a2c86..origin/develop",
            vec!["\\.jsx?$", "\\.tsx?$"],
        ),
        get_repo(
            "http://gitlab.osl.manamind.com/customers/sben.git",
            "dc8c47cc..origin/develop",
            vec!["\\.jsx?$", "\\.tsx?$"],
        ),
    );

    let mut ctx = tera::Context::new();
    ctx.insert(
        "projects",
        &vec![
            ("Norne", norne?),
            ("Sbanken", sbanken?),
            ("Arena", arena?),
            ("SBEN", sben?),
        ]
        .into_iter()
        .collect::<HashMap<&'static str, Vec<Commit>>>(),
    );

    let body = templates
        .render("index.tera", &ctx)
        .map_err(|error| error::ErrorInternalServerError(error))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var(
        "RUST_LOG",
        "error,actix_web=info,actix_http=info,actix_server=info",
    );
    env_logger::init();

    HttpServer::new(|| {
        let tera = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/*")).unwrap();

        App::new()
            .data(tera)
            .wrap(middleware::Logger::default())
            .service(web::resource("/").route(web::get().to(index)))
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}
