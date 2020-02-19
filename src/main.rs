#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

pub mod git;

use git::count_repo_files;
use rocket_contrib::templates::Template;

#[derive(Serialize)]
struct TemplateContext {
    commits: Vec<(i64, u32)>,
}

#[get("/")]
fn index() -> Template {
    let result = count_repo_files(
        "http://gitlab.osl.manamind.com/customers/norne.git",
        "50fba560..origin/develop",
        "\\.tsx?$"
    ).unwrap();

    format!("{:?}", result);
    Template::render("index", &TemplateContext {
        commits: result.iter().map(|item| (item.time, item.count)).collect()
    })
}

fn main() {
    rocket::ignite()
        .attach(Template::fairing())
        .mount("/", routes![index]).launch();
}
