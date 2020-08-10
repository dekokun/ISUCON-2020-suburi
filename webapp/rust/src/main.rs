use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use env_logger;
use log::info;
use serde::{Deserialize, Serialize};
use sqlx::{MySql, MySqlPool, Pool};
use std::env;
use tera::{Context, Tera};

#[macro_use]
extern crate lazy_static;

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec!["html", ".sql"]);
        tera
    };
}

// Vote Model
// type Vote struct {
// 	ID          int
// 	UserID      int
// 	CandidateID int
// 	Keyword     string
// }

#[derive(Debug, Clone, PartialEq)]
struct Vote {
    id: usize,
    user_id: usize,
    candidate_id: usize,
    keyword: String,
}

// type Candidate struct {
// 	ID             int
// 	Name           string
// 	PoliticalParty string
// 	Sex            string
// }

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Candidate {
    id: usize,
    name: String,
    political_party: String,
    sex: String,
}

impl Default for Candidate {
    fn default() -> Self {
        Self {
            id: 1,
            name: "デフォルト太郎".to_string(),
            political_party: "デフォルト政党".to_string(),
            sex: "デフォルト性別".to_string(),
        }
    }
}

async fn get_all_candidates() -> Vec<Candidate> {
    vec![Candidate::default()]
}

#[get("/vote")]
async fn get_vote() -> impl Responder {
    let candidates = get_all_candidates().await;
    let mut context = Context::new();
    context.insert("greeting", &"hello");
    let candidate = Candidate::default();
    context.insert("candidate", &candidate);
    match TEMPLATES.render("vote.tera.html", &context) {
        Ok(s) => HttpResponse::Ok().body(s),
        _ => unimplemented!(),
    }
}

// #[post("/vote")]
// async fn post_vote() -> impl Responder {
//     let mut ctx = tera::Context::new();
//     let candidates = get_all_candidates();
//     let template = unimplemented!();
//     HttpResponse::Ok().body(template)
// }

#[get("/")]
async fn index(data: web::Data<AppData>) -> impl Responder {
    HttpResponse::Ok().body(data.name)
}

type Data = web::Data<AppData>;

#[get("/initialize")]
async fn initialize(data: Data) -> impl Responder {
    sqlx::query("DELETE FROM votes")
        .execute(&data.pool)
        .await
        .unwrap();
    HttpResponse::Ok().body("Finished")
}

fn get_env(key: &'static str, fallback: &'static str) -> String {
    match env::var_os(key) {
        Some(val) => val.into_string().unwrap(),
        _ => fallback.to_string(),
    }
}

#[derive(Debug)]
struct AppData {
    name: &'static str,
    pool: Pool<MySql>,
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let user = get_env("ISHOCON2_DB_USER", "ishocon");
    let pass = get_env("ISHOCON2_DB_PASSWORD", "ishocon");
    let db_name = get_env("ISHOCON2_DB_NAME", "ishocon2");
    let env_args: Vec<_> = std::env::args().collect();
    if env_args.len() != 2 {
        panic!("port must be specified!");
    }

    let port = &env_args[1];

    let addr = format!("127.0.0.1:{}", port);

    env::set_var("RUST_LOG", "info");
    env_logger::init();
    info!("The server is running on {:?}", addr);

    let pool =
        MySqlPool::connect(&format!("mysql://{}:{}@localhost/{}", user, pass, db_name)).await;

    let pool = pool.unwrap();

    HttpServer::new(move || {
        App::new()
            .data(AppData {
                name: "krouton!",
                pool: pool.clone(),
            })
            .service(index)
            .service(initialize)
            .service(get_vote)
        // .service(vote)
    })
    .bind(addr)?
    .run()
    .await
}
