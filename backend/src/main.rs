use std::{env, error::Error};

use axum::{
    Router,
    extract::{Json, Path, State},
    http::StatusCode,
    routing,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use uuid::Uuid;

type Pool = bb8::Pool<Client>;

type HandlerResult<T> = Result<(StatusCode, T), (StatusCode, String)>;

#[derive(Clone)]
struct AppState {
    pool: Pool,
    jwt_secret: String,
}

#[derive(Serialize, Deserialize)]
struct User {
    username: String,
    password_hash: String,
}

#[derive(Serialize, Deserialize)]
struct Task {
    id: Uuid,
    category: String,
    text: String,
    completed: bool,
    due: Option<i32>, // UNIX timestamp
}

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String, // username
    exp: i64,    // expiration timestamp
    iat: i64,    // issued at timestamp
}

#[derive(Deserialize)]
struct RegisterRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct RegisterResponse {
    jwt: String,
}

#[derive(Deserialize)]
struct LoginRequest {
    username: Option<String>,
    password: Option<String>,
    jwt: Option<String>,
}

#[derive(Serialize)]
struct LoginResponse {
    jwt: String,
    username: String,
}

#[derive(Deserialize)]
struct LogoutRequest {
    jwt: String,
}

type LogoutResponse = ();

#[derive(Deserialize)]
struct GetAllTasksRequest {
    jwt: String,
}

type GetAllTasksResponse = Vec<Task>;

#[derive(Deserialize)]
struct CreateTaskRequest {
    jwt: String,
    category: String,
    text: String,
    completed: bool,
    due: Option<i32>, // UNIX timestamp
}

type CreateTaskResponse = ();

#[derive(Deserialize)]
struct GetTaskRequest {
    jwt: String,
}

type GetTaskResponse = Task;

#[derive(Deserialize)]
struct UpdateTaskRequest {
    jwt: String,
    category: Option<String>,
    text: Option<String>,
    completed: Option<bool>,
    due: Option<i32>, // UNIX timestamp
}

type UpdateTaskResponse = ();

#[derive(Deserialize)]
struct DeleteTaskRequest {
    jwt: String,
}

type DeleteTaskResponse = ();

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::open(env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1".to_owned()))?;
    let pool = Pool::builder().build(client).await?;
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "dont-forget-to-remove-me".to_owned()); // TODO: Remove fallback to hardcoded secret!!!
    let router = Router::new()
        .route("/auth/register", routing::post(register_handler))
        .route("/auth/login", routing::post(login_handler))
        .route("/auth/logout", routing::post(logout_handler))
        .route("/task", routing::get(get_all_tasks_handler).post(create_task_handler))
        .route("/task/{id}", routing::get(get_task_handler).post(update_task_handler).delete(delete_task_handler))
        .with_state(AppState {
            pool,
            jwt_secret,
        });
    let listener = TcpListener::bind(env::var("ROUTER_URL").unwrap_or_else(|_| "127.0.0.1:6767".to_owned())).await?;
    axum::serve(listener, router).await?;
    Ok(())
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

fn generate_jwt(secret: &str, username: &str) -> jsonwebtoken::errors::Result<String> {
    let now = Utc::now();
    let expiration = now + Duration::hours(1);
    jsonwebtoken::encode(
        &Header::default(),
        &Claims {
            sub: username.to_owned(),
            exp: expiration.timestamp(),
            iat: now.timestamp(),
        },
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

// TODO: More payload checks!
async fn register_handler(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> HandlerResult<Json<RegisterResponse>> {
    let mut conn = state.pool.get().await.map_err(internal_error)?;
    let user_key = format!("user:{}", payload.username);
    if conn.exists(&user_key).await.map_err(internal_error)? {
        return Err((StatusCode::CONFLICT, "Username already exists".to_owned()));
    }
    let user_json = serde_json::to_string(&User {
        username: payload.username.clone(),
        password_hash: bcrypt::hash(&payload.password, bcrypt::DEFAULT_COST).map_err(internal_error)?,
    })
    .map_err(internal_error)?;
    conn.set::<_, _, ()>(&user_key, user_json).await.map_err(internal_error)?;
    let jwt = generate_jwt(&state.jwt_secret, &payload.username).map_err(internal_error)?;
    Ok((
        StatusCode::OK,
        Json(RegisterResponse {
            jwt,
        }),
    ))
}

async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> HandlerResult<Json<LoginResponse>> {
    let mut conn = state.pool.get().await.map_err(internal_error)?;
    if let Some(jwt) = payload.jwt {
        let blacklist_key = format!("blacklist:{jwt}");
        if conn.exists(&blacklist_key).await.map_err(internal_error)? {
            return Err((StatusCode::UNAUTHORIZED, "JWT has been revoked".to_owned()));
        }
        let jwt_data = jsonwebtoken::decode::<Claims>(
            jwt,
            &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|err| (StatusCode::UNAUTHORIZED, format!("Invalid JWT: {err}")))?;
        let username = jwt_data.claims.sub;
        let user_key = format!("user:{username}");
        if !conn.exists(&user_key).await.map_err(internal_error)? {
            return Err((StatusCode::UNAUTHORIZED, "User not found".to_owned()));
        }
        let jwt = generate_jwt(&state.jwt_secret, &username).map_err(internal_error)?;
        let ttl = jwt_data.claims.exp - Utc::now().timestamp();
        if ttl > 0 {
            conn.set_ex::<_, _, ()>(&blacklist_key, "1", ttl as u64).await.map_err(internal_error)?;
        }
        Ok((
            StatusCode::OK,
            Json(LoginResponse {
                jwt,
                username,
            }),
        ))
    } else if let (Some(username), Some(password)) = (payload.username, payload.password) {
        let user_key = format!("user:{username}");
        let user_json: Option<String> = conn.get(&user_key).await.map_err(internal_error)?;
        let user_json =
            user_json.ok_or_else(|| (StatusCode::UNAUTHORIZED, "Invalid username or password".to_owned()))?;
        let user: User = serde_json::from_str(&user_json).map_err(internal_error)?;
        if !bcrypt::verify(password, &user.password_hash).map_err(internal_error)? {
            return Err((StatusCode::UNAUTHORIZED, "Invalid username or password".to_owned()));
        }
        let jwt = generate_jwt(&state.jwt_secret, &username).map_err(internal_error)?;
        Ok((
            StatusCode::OK,
            Json(LoginResponse {
                jwt,
                username,
            }),
        ))
    } else {
        Err((StatusCode::BAD_REQUEST, "Must provide either JWT or username/password".to_owned()))
    }
}

async fn logout_handler(
    State(state): State<AppState>,
    Json(payload): Json<LogoutRequest>,
) -> HandlerResult<LogoutResponse> {
    let mut conn = state.pool.get().await.map_err(internal_error)?;
    let jwt_data = jsonwebtoken::decode::<Claims>(
        &payload.jwt,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|err| (StatusCode::UNAUTHORIZED, format!("Invalid JWT: {err}")))?;
    let blacklist_key = format!("blacklist:{}", payload.jwt);
    let ttl = jwt_data.claims.exp - Utc::now().timestamp();
    if ttl > 0 {
        conn.set_ex::<_, _, ()>(&blacklist_key, "1", ttl.cast_unsigned()).await.map_err(internal_error)?;
    }
    Ok((StatusCode::OK, ()))
}

async fn get_all_tasks_handler(
    State(state): State<AppState>,
    Json(payload): Json<GetAllTasksRequest>,
) -> HandlerResult<Json<GetAllTasksResponse>> {
    let mut conn = state.pool.get().await.map_err(internal_error)?;
    let blacklist_key = format!("blacklist:{}", payload.jwt);
    if conn.exists(&blacklist_key).await.map_err(internal_error)? {
        return Err((StatusCode::UNAUTHORIZED, "JWT has been revoked".to_owned()));
    }
    let jwt_data = jsonwebtoken::decode::<Claims>(
        payload.jwt,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|err| (StatusCode::UNAUTHORIZED, format!("Invalid JWT: {err}")))?;
    let username = jwt_data.claims.sub;
    let task_ids_key = format!("task_ids:{username}");
    let task_ids: Vec<String> = conn.smembers(&task_ids_key).await.map_err(internal_error)?;
    let mut tasks = Vec::with_capacity(task_ids.len());
    for task_id in task_ids {
        let task_key = format!("task:{username}:{task_id}");
        let task_json: Option<String> = conn.get(&task_key).await.map_err(internal_error)?;
        if let Some(json) = task_json {
            let task: Task = serde_json::from_str(&json).map_err(internal_error)?;
            tasks.push(task);
        }
    }
    Ok((StatusCode::OK, Json(tasks)))
}

async fn create_task_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskRequest>,
) -> HandlerResult<CreateTaskResponse> {
    let mut conn = state.pool.get().await.map_err(internal_error)?;
    let blacklist_key = format!("blacklist:{}", payload.jwt);
    if conn.exists(&blacklist_key).await.map_err(internal_error)? {
        return Err((StatusCode::UNAUTHORIZED, "JWT has been revoked".to_owned()));
    }
    let jwt_data = jsonwebtoken::decode::<Claims>(
        payload.jwt,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|err| (StatusCode::UNAUTHORIZED, format!("Invalid JWT: {err}")))?;
    let username = jwt_data.claims.sub;
    let task_id = Uuid::new_v4();
    let task_key = format!("task:{username}:{task_id}");
    let task_json = serde_json::to_string(&Task {
        id: task_id,
        category: payload.category,
        text: payload.text,
        completed: payload.completed,
        due: payload.due,
    })
    .map_err(internal_error)?;
    // TODO: Expire after 30 days.
    conn.set::<_, _, ()>(&task_key, task_json).await.map_err(internal_error)?;
    let task_ids_key = format!("task_ids:{username}");
    conn.sadd::<_, _, ()>(&task_ids_key, task_id.to_string()).await.map_err(internal_error)?;
    Ok((StatusCode::CREATED, ()))
}

async fn get_task_handler(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
    Json(payload): Json<GetTaskRequest>,
) -> HandlerResult<Json<GetTaskResponse>> {
    let mut conn = state.pool.get().await.map_err(internal_error)?;
    let blacklist_key = format!("blacklist:{}", payload.jwt);
    if conn.exists(&blacklist_key).await.map_err(internal_error)? {
        return Err((StatusCode::UNAUTHORIZED, "JWT has been revoked".to_owned()));
    }
    let jwt_data = jsonwebtoken::decode::<Claims>(
        payload.jwt,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|err| (StatusCode::UNAUTHORIZED, format!("Invalid JWT: {err}")))?;
    let username = jwt_data.claims.sub;
    let task_key = format!("task:{username}:{task_id}");
    let task_json: Option<String> = conn.get(&task_key).await.map_err(internal_error)?;
    let task_json = task_json.ok_or_else(|| (StatusCode::NOT_FOUND, "Task not found".to_owned()))?;
    let task: Task = serde_json::from_str(&task_json).map_err(internal_error)?;
    Ok((StatusCode::OK, Json(task)))
}

async fn update_task_handler(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
    Json(payload): Json<UpdateTaskRequest>,
) -> HandlerResult<UpdateTaskResponse> {
    let mut conn = state.pool.get().await.map_err(internal_error)?;
    let blacklist_key = format!("blacklist:{}", payload.jwt);
    if conn.exists(&blacklist_key).await.map_err(internal_error)? {
        return Err((StatusCode::UNAUTHORIZED, "JWT has been revoked".to_owned()));
    }
    let jwt_data = jsonwebtoken::decode::<Claims>(
        payload.jwt,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|err| (StatusCode::UNAUTHORIZED, format!("Invalid JWT: {err}")))?;
    let username = jwt_data.claims.sub;
    let task_key = format!("task:{username}:{task_id}");
    let task_json: Option<String> = conn.get(&task_key).await.map_err(internal_error)?;
    let task_json = task_json.ok_or_else(|| (StatusCode::NOT_FOUND, "Task not found".to_owned()))?;
    let mut task: Task = serde_json::from_str(&task_json).map_err(internal_error)?;
    if let Some(category) = payload.category {
        task.category = category;
    }
    if let Some(text) = payload.text {
        task.text = text;
    }
    if let Some(completed) = payload.completed {
        task.completed = completed;
    }
    if let Some(due) = payload.due {
        task.due = Some(due);
    }
    let task_json = serde_json::to_string(&task).map_err(internal_error)?;
    conn.set::<_, _, ()>(&task_key, task_json).await.map_err(internal_error)?;
    Ok((StatusCode::OK, ()))
}

async fn delete_task_handler(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
    Json(payload): Json<DeleteTaskRequest>,
) -> HandlerResult<DeleteTaskResponse> {
    let mut conn = state.pool.get().await.map_err(internal_error)?;
    let blacklist_key = format!("blacklist:{}", payload.jwt);
    if conn.exists(&blacklist_key).await.map_err(internal_error)? {
        return Err((StatusCode::UNAUTHORIZED, "JWT has been revoked".to_owned()));
    }
    let jwt_data = jsonwebtoken::decode::<Claims>(
        payload.jwt,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|err| (StatusCode::UNAUTHORIZED, format!("Invalid JWT: {err}")))?;
    let username = jwt_data.claims.sub;
    let task_key = format!("task:{username}:{task_id}");
    if !conn.exists(&task_key).await.map_err(internal_error)? {
        return Err((StatusCode::NOT_FOUND, "Task not found".to_owned()));
    }
    conn.del::<_, ()>(&task_key).await.map_err(internal_error)?;
    let task_ids_key = format!("task_ids:{username}");
    conn.srem::<_, _, ()>(&task_ids_key, task_id.to_string()).await.map_err(internal_error)?;
    Ok((StatusCode::OK, ()))
}
