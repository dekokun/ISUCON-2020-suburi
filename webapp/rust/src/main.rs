use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use sqlx::{query, MySql, MySqlPool, Pool};
use std::env;
use tera::Tera;

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

// #[get("/vote")]
// async fn get_vote() -> impl Responder {
//     let mut ctx = tera::Context::new();
//     let candidates = get_all_candidates();
//     let template = unimplemented!();
//     HttpResponse::Ok().body(template)
// }

// #[post("/vote")]
// async fn post_vote() -> impl Responder {
//     let mut ctx = tera::Context::new();
//     let candidates = get_all_candidates();
//     let template = unimplemented!();
//     HttpResponse::Ok().body(template)
// }

fn get_all_candidates() -> usize {
    1
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
    let user = get_env("ISHOCON2_DB_USER", "ishocon");
    let pass = get_env("ISHOCON2_DB_PASSWORD", "ishocon");
    let db_name = get_env("ISHOCON2_DB_NAME", "ishocon2");
    dbg!(&user, &pass, &db_name);

    let pool =
        MySqlPool::connect(&format!("mysql://{}:{}@localhost/{}", user, pass, db_name)).await;
    dbg!(&pool);

    let pool = pool.unwrap();

    HttpServer::new(move || {
        App::new()
            .data(AppData {
                name: "krouton!",
                pool: pool.clone(),
            })
            .service(index)
            .service(initialize)
        // .service(vote)
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}
