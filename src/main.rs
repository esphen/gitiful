#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;

pub mod git;

use git::{count_repo_files, Commit};
use rocket_contrib::templates::Template;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
struct TemplateContext {
    projects: HashMap<&'static str, Vec<Commit>>,
}

#[get("/")]
fn index() -> Template {
    Template::render(
        "index",
        &TemplateContext {
            projects: vec![
                (
                    "Norne",
                    count_repo_files(
                        "http://gitlab.osl.manamind.com/customers/norne.git",
                        "50fba560..origin/develop",
                        vec!["\\.jsx?$", "\\.tsx?$"],
                    )
                    .unwrap(),
                ),
                (
                    "Sbanken",
                    count_repo_files(
                        "http://gitlab.osl.manamind.com/customers/skbn.git",
                        "952bc527..origin/develop",
                        vec!["\\.jsx?$", "\\.tsx?$"],
                    )
                    .unwrap(),
                ),
            ]
            .into_iter()
            .collect(),
        },
    )
}

fn main() {
    rocket::ignite()
        .attach(Template::fairing())
        .mount("/", routes![index])
        .launch();
}
