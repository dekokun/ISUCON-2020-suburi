use actix_files::Files;
use actix_web::middleware::Logger;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use env_logger;
use log::info;
use serde::{Deserialize, Serialize};
use sqlx::{MySql, MySqlPool, Pool, Row};
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct ElectionResult {
    id: i32,
    name: String,
    political_party: String,
    sex: String,
    count: Option<i64>,
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
            let _ = dbg!(e);
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

fn get_env(key: &'static str, fallback: &'static str) -> String {
    match env::var_os(key) {
        Some(val) => val.into_string().unwrap(),
        _ => fallback.to_string(),
    }
}

async fn get_election_result(pool: Pool<MySql>) -> Vec<ElectionResult> {
    let ret: Vec<ElectionResult> = sqlx::query_as!(
        ElectionResult,
        r#"
		SELECT c.id as id, c.name as name, c.political_party as political_party, c.sex as sex, v.count as count
		FROM candidates AS c
		LEFT OUTER JOIN
	  	(SELECT candidate_id, COUNT(*) AS count
	  	FROM votes
	  	GROUP BY candidate_id) AS v
		ON c.id = v.candidate_id
        ORDER BY v.count DESC"#
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    ret
}

async fn get_voice_supporter(pool: Pool<MySql>, candidates_ids: Vec<i32>) -> Vec<String> {
    if candidates_ids.is_empty() {
        return vec![];
    }
    // 動作確認SQL: insert into votes (user_id, candidate_id, keyword) values (1, 16, "応援してます");
    // 動作確認URL: https://127.0.0.1/political_parties/%E5%A4%A2%E5%AE%9F%E7%8F%BE%E5%85%9A
    let bind_params = vec!["?"; candidates_ids.len()].join(",");
    let query = r#"
    SELECT keyword
    FROM votes
    WHERE candidate_id IN ("#
        .to_owned()
        + &bind_params
        + r#")
    GROUP BY keyword
    ORDER BY COUNT(*) DESC
    LIMIT 10
    "#;
    let mut query = sqlx::query(&query);
    for candidates_id in candidates_ids {
        query = query.bind(candidates_id);
    }
    let rows = query.fetch_all(&pool).await.unwrap();
    rows.into_iter().map(|v| v.get("keyword")).collect()
}

#[get("/political_parties/{name}")]
async fn get_political_parties(data: Data, name: web::Path<String>) -> impl Responder {
    let election_results = get_election_result(data.pool.clone()).await;
    let mut votes = 0;
    let name = &*name;
    for v in &election_results {
        if v.political_party == *name && v.count.is_some() {
            votes += v.count.unwrap();
        }
    }
    let candidates: Vec<ElectionResult> = election_results
        .into_iter()
        .filter(|c| c.political_party == *name)
        .collect();
    let keywords: Vec<String> =
        get_voice_supporter(data.pool.clone(), candidates.iter().map(|c| c.id).collect()).await;
    let mut context = Context::new();
    context.insert("name", name);
    context.insert("votes", &votes);
    context.insert("candidates", &candidates);
    context.insert("keywords", &keywords);
    match TEMPLATES.render("political_parties.tera.html", &context) {
        Ok(s) => HttpResponse::Ok().body(s),
        e => {
            let _ = dbg!(e);
            unimplemented!()
        }
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
            .service(get_political_parties)
            .service(Files::new("/", "./public/").index_file("index.html"))
            .wrap(Logger::default())
        // .service(vote)
    })
    .bind(addr)?
    .run()
    .await
}
