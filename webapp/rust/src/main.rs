use actix_files::Files;
use actix_web::{
    get, post,
    web::{self, Form},
    App, HttpResponse, HttpServer, Responder,
};
use env_logger;
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

#[derive(Debug, Serialize, Deserialize)]
pub struct VoteParams {
    name: String,
    address: String,
    mynumber: String,
}

async fn get_candidate_by_name(pool: &Pool<MySql>, name: &str) -> Candidate {
    let candidate = sqlx::query!("select * from candidates where name = ?", name)
        .fetch_one(pool) // -> Vec<{ country: String, count: i64 }>
        .await
        .unwrap();

    Candidate {
        id: candidate.id as usize,
        name: candidate.name,
        political_party: candidate.political_party,
        sex: candidate.sex,
    }
}

async fn get_user_voted_count(pool: &Pool<MySql>, mynumber: &str) -> usize {
    // sqlx::query!(
    //     "SELECT COUNT(*) AS count FROM votes WHERE user_id =  ?",
    //     userID
    // )
    1
}

async fn get_user(pool: &Pool<MySql>, name: &str, address: &str, mynumber: &str) -> User {
    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE name = ? AND address = ? AND mynumber = ?",
        name,
        address,
        mynumber
    )
    .fetch_one(pool) // -> Vec<{ country: String, count: i64 }>
    .await
    .unwrap();
    user
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct User {
    id: i32,
    name: String,
    address: String,
    mynumber: String,
    votes: i32,
}

#[post("/vote")]
async fn post_vote(data: Data, params: Form<VoteParams>) -> impl Responder {
    let mut ctx = tera::Context::new();
    let pool = &data.pool;
    let name = params.name;
    let address = params.address;
    let mynumber = params.mynumber;
    let user = get_user(pool, &name, &address, &mynumber);

    let voted_count = get_user_voted_count(&data.pool, &params.mynumber).await;

    HttpResponse::Ok().body("body")
}

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
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let env_args: Vec<_> = std::env::args().collect();
    if env_args.len() != 2 {
        panic!("port must be specified!");
    }
    let port = &env_args[1];
    let addr = format!("127.0.0.1:{}", port);

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
            .service(post_vote)
            .service(Files::new("/", "./public/").index_file("index.html"))
        // .service(vote)
    })
    .bind(addr)?
    .run()
    .await
}
