/**
 * sqlx::query("SELECT");   -> Query
 * sqlx::query_as("");      -> QueryAs
 * 
 * .fetch               -> Stream (not recommended it's too complicated)
 * .fetch_one           -> T
 * .fetch_optional      -> Option<T>
 * .fetch_all           -> Vec<T>
 * .execute             -> Database::QueryResult -> MySqlQueryResult
 * 
 * all_fetch return MySqlRow
 */ 

use actix_web::{
    HttpServer, 
    App, 
    Responder, 
    HttpResponse,
    web
};

use sqlx::mysql::{
    MySqlPool,
    MySqlPoolOptions
};

#[derive(Clone)]
pub struct AppState {
    pub pool: MySqlPool
}

#[derive(serde::Serialize)]
struct User {
    id: i32,
    username: String,
    email: String
}

#[derive(serde::Deserialize, Clone)]
pub struct UserPayload {
    pub username: String,
    pub email: String
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();


    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_err| {
        eprintln!("DATABASE_URL must set first.");
        std::process::exit(1);
    });

    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await
        .unwrap();

    let app_state = AppState { pool };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .route("/ping", web::get().to(ping_server))
            .service(
                web::scope("/users")
                    .route("", web::get().to(get_users))
                    .route("", web::post().to(create_user))
                    .service(
                        web::resource("/{user_id}")
                            .get(get_user)
                            .put(update_user)
                            .delete(delete_user)
                            .default_service(web::to(|| async {
                                HttpResponse::MethodNotAllowed().json(
                                    serde_json::json!({
                                        "msg": "Method Not Allowed"
                                    })
                                )
                            }))
                    )
                    .service(
                        web::scope("/another")
                            .route("/{user_id}", web::get().to(get_user_another_way))
                    )
            )
            .default_service(web::to(|| async {
                HttpResponse::NotFound().json(
                    serde_json::json!({
                        "msg": "Page Not Found"
                    })
                )
            }))
            
    })
    .workers(8)
    .bind(("0.0.0.0", 80))?
    .run()
    .await
}

async fn ping_server() -> impl Responder {
    HttpResponse::Ok().body("PONG")
}

async fn get_users(app_state: web::Data<AppState>) -> impl Responder {
    let query = sqlx::query_as!(
        User,
        "SELECT * FROM users"
    )
        .fetch_all(&app_state.pool)
        .await;

    match query {
        Ok(users_data) => HttpResponse::Ok().json(
            serde_json::json!({
                "msg": "success",
                "data": users_data
            })
        ),
        Err(error) => HttpResponse::InternalServerError().json(
            serde_json::json!({
                "msg": "failed",
                "details": error.to_string()
            })
        )
    }
}

async fn get_user(path: web::Path<usize>, app_state: web::Data<AppState>) -> impl Responder {
    let user_id = path.into_inner();

    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE id = ?",
        user_id as u64
    )
        .fetch_optional(&app_state.pool)
        .await.unwrap();

    match user {
        Some(user) => HttpResponse::Ok().json(user),
        None => HttpResponse::NotFound().json(
            serde_json::json!({
                "msg": format!("user with id: {} not found", user_id)
            })
        )
    }
}

pub async fn create_user(payload: web::Json<UserPayload>, app_state: web::Data<AppState>) -> impl Responder {
    let query = sqlx::query!(
        "INSERT INTO users (username, email) VALUES (?, ?);",
        payload.username,
        payload.email
    )
        .execute(&app_state.pool)
        .await;

    match query {
        Ok(res) => {
            if res.rows_affected() > 0 {
                HttpResponse::Ok().json(
                    serde_json::json!({
                        "status": "success",
                        "msg": format!("user {} successfully added!", payload.username)
                    })
                )
            } else {
                HttpResponse::BadRequest().json(
                    serde_json::json!({
                        "msg": format!("failed creating user {}", payload.username)
                    })
                )
            }
        }
        Err(_) => HttpResponse::InternalServerError().finish()
    }
}

async fn update_user(path: web::Path<usize>, payload: web::Json<UserPayload>, app_state: web::Data<AppState>) -> impl Responder {
    let user_id = path.into_inner();

    let query = sqlx::query("UPDATE users SET username = ?, email = ? WHERE id = ?")
        .bind(payload.username.clone())
        .bind(payload.email.clone())
        .bind(user_id as u64)
        .execute(&app_state.pool)
        .await;

    match query {
        Ok(res) => {
            if res.rows_affected() > 0 {
                HttpResponse::Ok().json(
                    serde_json::json!({
                        "status": "success",
                        "msg": format!("user with id: {} successfully updated", user_id)
                    })
                )
            } else {
                HttpResponse::BadRequest()
                    .json(serde_json::json!({
                        "status": "failed",
                        "msg": format!("failed updating user with id: {}", user_id)
                    }))
            }
        }
        Err(error) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "msg": "failed",
                "detail": error.to_string()
            }))
        }
    }
}

async fn delete_user(path: web::Path<usize>, app_state: web::Data<AppState>) -> impl Responder {
    let user_id = path.into_inner();

    let result = sqlx::query!(
        "DELETE FROM users WHERE id = ?",
        user_id as u64
    )
        .execute(&app_state.pool)
        .await;

    match result {
        Ok(res) => {
            if res.rows_affected() > 0 {
                HttpResponse::Ok()
                    .json(
                        serde_json::json!({
                            "msg": format!("user with id: {} successfully deleted", user_id)
                        })
                    )
            } else {
                HttpResponse::UnprocessableEntity().json(
                    serde_json::json!({
                        "msg": format!("failed delete user with id: {}", user_id)
                    })
                )
            }
        }
        Err(_) => HttpResponse::BadRequest().finish()
    }
}

async fn get_user_another_way(path: web::Path<usize>, app_state: web::Data<AppState>) -> impl Responder {
    let user_id = path.into_inner();

    #[derive(serde::Serialize)]
    struct User {
        id: i32,
        username: String,
        email: String
    }

    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE id = ?",
        user_id as u64
    )
        .fetch_one(&app_state.pool)
        .await;

    match user {
        Ok(user_data) => {
            HttpResponse::Ok().json(
                serde_json::json!({
                    "status": "success",
                    "data": user_data
                })
            )
        }
        Err(err) => HttpResponse::BadRequest().json(
            serde_json::json!({
                "status": "failed",
                "msg": format!("user with id: {} not found", user_id),
                "detail": err.to_string()
            })
        ) 
    }
}