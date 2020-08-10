use actix_files::Files;
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

lazy_static! {
    pub static ref DATABASE_URL: String = {
        let user = get_env("ISHOCON2_DB_USER", "ishocon");
        let pass = get_env("ISHOCON2_DB_PASSWORD", "ishocon");
        let db_name = get_env("ISHOCON2_DB_NAME", "ishocon2");
        format!("mysql://{}:{}@localhost/{}", user, pass, db_name)
    };
}

#[derive(Debug, Clone, PartialEq)]
struct Vote {
    id: usize,
    user_id: usize,
    candidate_id: usize,
    keyword: String,
}

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

async fn get_candidate_by_id(pool: Pool<MySql>, candidate_id: u32) -> Candidate {
    let candidate = sqlx::query!(
            "select * from candidates where id = ?", 
            candidate_id)
        .fetch_one(&pool).await.unwrap();
    
    Candidate {
        id: candidate.id as usize,
        name: candidate.name,
        political_party: candidate.political_party,
        sex: candidate.sex,
    }
}

async fn get_all_candidates(pool: Pool<MySql>) -> Vec<Candidate> {
    let candidates: Vec<_> = sqlx::query!("select name from candidates")
        .fetch_all(&pool) // -> Vec<{ country: String, count: i64 }>
        .await
        .unwrap();
    candidates
        .into_iter()
        .map(|can| Candidate {
            name: can.name,
            ..Default::default()
        })
        .collect()
}

#[get("/vote")]
async fn get_vote(data: Data) -> impl Responder {
    let candidates = get_all_candidates(data.pool.clone()).await;
    let mut context = Context::new();
    context.insert("greeting", &"hello");
    context.insert("candidates", &candidates);
    match TEMPLATES.render("vote.tera.html", &context) {
        Ok(s) => HttpResponse::Ok().body(s),
        e => {
            dbg!(e);
            unimplemented!()
        }
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

#[get("/candidates/{candidateId}")]
async fn get_candidate(data: Data, candidate_id: web::Path<u32>) -> impl Responder {
    let candidate = get_candidate_by_id(data.pool.clone(), *candidate_id).await;
    let mut context = Context::new();
    context.insert("candidate", &candidate);
    match TEMPLATES.render("candidate.tera.html", &context) {
        Ok(s) => HttpResponse::Ok().body(s),
        e => {
            dbg!(e);
            unimplemented!()
        }
    }
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
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let env_args: Vec<_> = std::env::args().collect();
    if env_args.len() != 2 {
        panic!("port must be specified!");
    }
    let port = &env_args[1];
    let addr = format!("0.0.0.0:{}", port);

    let pool = MySqlPool::connect(&DATABASE_URL).await;

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
            .service(get_candidate)
            .service(Files::new("/", "./public/").index_file("index.html"))
        // .service(vote)
    })
    .bind(addr)?
    .run()
    .await
}
