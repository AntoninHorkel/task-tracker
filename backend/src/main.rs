use std::{env, error::Error};

use axum::{
    Router,
    extract::{
        Json,
        Path,
        Query,
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    routing,
};
use chrono::{Duration, Utc};
use futures::{SinkExt, StreamExt, stream::SplitSink};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, errors::Result as JWTResult};
use redis::{AsyncCommands, Client as RedisClient};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use uuid::Uuid;

type Pool = bb8::Pool<RedisClient>;

type HandlerResult<T> = Result<(StatusCode, T), (StatusCode, String)>;

#[derive(Clone)]
struct AppState {
    redis_client: RedisClient,
    pool: Pool,
    jwt_secret: String,
}

#[derive(Serialize, Deserialize)]
struct User {
    username: String,
    password_hash: String,
}

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[allow(clippy::enum_variant_names)]
enum Notification {
    #[serde(rename = "task_created")]
    TaskCreated { task: Task },
    #[serde(rename = "task_updated")]
    TaskUpdated { task: Task },
    #[serde(rename = "task_deleted")]
    TaskDeleted { task_id: Uuid },
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum ClientWebSocketMessage {
    #[serde(rename = "refresh_jwt")]
    RefreshJwt { jwt: String },
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum ServerWebSocketMessage {
    #[serde(rename = "error")]
    Error { message: String },
}

#[derive(Deserialize)]
struct WebsocketQuery {
    jwt: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let redis_client =
        RedisClient::open(env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_owned()))?;
    let pool = Pool::builder().build(redis_client.clone()).await?;
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "dont-forget-to-remove-me".to_owned()); // TODO: Remove fallback to hardcoded secret!!!
    let router = Router::new()
        .route("/auth/register", routing::post(register_handler))
        .route("/auth/login", routing::post(login_handler))
        .route("/auth/logout", routing::post(logout_handler))
        .route("/task", routing::get(get_all_tasks_handler).post(create_task_handler))
        .route("/task/{id}", routing::get(get_task_handler).post(update_task_handler).delete(delete_task_handler))
        .route("/websocket", routing::get(websocket_handler))
        .with_state(AppState {
            redis_client,
            pool,
            jwt_secret,
        });
    let listener = TcpListener::bind(env::var("ROUTER_URL").unwrap_or_else(|_| "127.0.0.1:6767".to_owned())).await?;
    axum::serve(listener, router).await?;
    Ok(())
}

fn internal_error<E: Error>(err: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

fn generate_jwt(secret: &str, username: &str) -> JWTResult<String> {
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

fn validate_jwt(secret: &str, jwt: &str) -> Result<Claims, (StatusCode, String)> {
    jsonwebtoken::decode::<Claims>(jwt, &DecodingKey::from_secret(secret.as_bytes()), &Validation::default())
        .map(|data| data.claims)
        .map_err(|err| (StatusCode::UNAUTHORIZED, format!("Invalid JWT: {err}")))
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
        let jwt_data = validate_jwt(&state.jwt_secret, &jwt)?;
        let username = jwt_data.sub;
        let user_key = format!("user:{username}");
        if !conn.exists(&user_key).await.map_err(internal_error)? {
            return Err((StatusCode::UNAUTHORIZED, "User not found".to_owned()));
        }
        let jwt = generate_jwt(&state.jwt_secret, &username).map_err(internal_error)?;
        let ttl = jwt_data.exp - Utc::now().timestamp();
        if ttl > 0 {
            conn.set_ex::<_, _, ()>(&blacklist_key, "1", ttl.cast_unsigned()).await.map_err(internal_error)?;
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
    let jwt_data = validate_jwt(&state.jwt_secret, &payload.jwt)?;
    let blacklist_key = format!("blacklist:{}", payload.jwt);
    let ttl = jwt_data.exp - Utc::now().timestamp();
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
    let jwt_data = validate_jwt(&state.jwt_secret, &payload.jwt)?;
    let username = jwt_data.sub;
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
    let jwt_data = validate_jwt(&state.jwt_secret, &payload.jwt)?;
    let username = jwt_data.sub;
    let task_id = Uuid::new_v4();
    let task = Task {
        id: task_id,
        category: payload.category,
        text: payload.text,
        completed: payload.completed,
        due: payload.due,
    };
    let task_key = format!("task:{username}:{task_id}");
    let task_json = serde_json::to_string(&task).map_err(internal_error)?;
    conn.set::<_, _, ()>(&task_key, task_json).await.map_err(internal_error)?;
    let task_ids_key = format!("task_ids:{username}");
    conn.sadd::<_, _, ()>(&task_ids_key, task_id.to_string()).await.map_err(internal_error)?;
    let channel = format!("notifications:{username}");
    let notification_json = serde_json::to_string(&Notification::TaskCreated {
        task,
    })
    .map_err(internal_error)?;
    conn.publish::<_, _, ()>(&channel, notification_json).await.map_err(internal_error)?;
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
    let jwt_data = validate_jwt(&state.jwt_secret, &payload.jwt)?;
    let username = jwt_data.sub;
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
    let jwt_data = validate_jwt(&state.jwt_secret, &payload.jwt)?;
    let username = jwt_data.sub;
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
    let channel = format!("notifications:{username}");
    let notification_json = serde_json::to_string(&Notification::TaskUpdated {
        task,
    })
    .map_err(internal_error)?;
    conn.publish::<_, _, ()>(&channel, notification_json).await.map_err(internal_error)?;
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
    let jwt_data = validate_jwt(&state.jwt_secret, &payload.jwt)?;
    let username = jwt_data.sub;
    let task_key = format!("task:{username}:{task_id}");
    if !conn.exists(&task_key).await.map_err(internal_error)? {
        return Err((StatusCode::NOT_FOUND, "Task not found".to_owned()));
    }
    conn.del::<_, ()>(&task_key).await.map_err(internal_error)?;
    let task_ids_key = format!("task_ids:{username}");
    conn.srem::<_, _, ()>(&task_ids_key, task_id.to_string()).await.map_err(internal_error)?;
    let channel = format!("notifications:{username}");
    let notification_json = serde_json::to_string(&Notification::TaskDeleted {
        task_id,
    })
    .map_err(internal_error)?;
    conn.publish::<_, _, ()>(&channel, notification_json).await.map_err(internal_error)?;
    Ok((StatusCode::OK, ()))
}

async fn websocket_handler(
    websocket: WebSocketUpgrade,
    Query(query): Query<WebsocketQuery>,
    State(state): State<AppState>,
) -> HandlerResult<()> {
    let mut conn = state.pool.get().await.map_err(internal_error)?;
    let blacklist_key = format!("blacklist:{}", query.jwt);
    if conn.exists(&blacklist_key).await.map_err(internal_error)? {
        return Err((StatusCode::UNAUTHORIZED, "JWT has been revoked".to_owned()));
    }
    let jwt_data = validate_jwt(&state.jwt_secret, &query.jwt)?;
    let username = jwt_data.sub;
    let channel = format!("notifications:{username}");
    let _dropme = websocket.on_upgrade(|socket| async move {
        let (mut sender, mut receiver) = socket.split();
        let mut pubsub = match state.redis_client.get_async_pubsub().await {
            Ok(pubsub) => pubsub,
            Err(err) => {
                send_error(&mut sender, format!("Failed to parse notification: {err}")).await;
                if let Err(err) = sender.send(Message::Close(None)).await {
                    eprintln!("WebSocket connection close send error: {err}");
                }
                return;
            }
        };
        if let Err(err) = pubsub.subscribe(channel).await {
            send_error(&mut sender, format!("Failed to subscribe to notifications: {err}")).await;
            if let Err(err) = sender.send(Message::Close(None)).await {
                eprintln!("WebSocket connection close send error: {err}");
            }
            return;
        }
        let mut pubsub_stream = pubsub.on_message();
        loop {
            tokio::select! {
                msg = receiver.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            match serde_json::from_str::<ClientWebSocketMessage>(&text) {
                                Ok(ClientWebSocketMessage::RefreshJwt { jwt }) => {
                                    // TODO
                                    println!("JWT refresh request {jwt:?}");
                                }
                                Err(err) => {
                                    send_error(&mut sender, format!("Internal error: {err}")).await;
                                    break;
                                }
                            }
                        }
                        Some(Ok(Message::Ping(data))) => {
                            if let Err(err) = sender.send(Message::Pong(data)).await {
                                eprintln!("WebSocket PONG send error: {err}");
                                break;
                            }
                        }
                        Some(Ok(Message::Close(_))) | None => break,
                        Some(Err(err)) => {
                            eprintln!("WebSocket error: {err}");
                            break;
                        },
                        _ => {}
                    }
                }
                msg = pubsub_stream.next() => {
                    if let Some(msg) = msg {
                        let payload: Result<String, _> = msg.get_payload();
                        match payload {
                            Ok(notification_json) => {
                                if let Err(err) = sender.send(Message::Text(notification_json.into())).await {
                                    eprintln!("WebSocket notification JSON send error: {err}");
                                    break;
                                }
                            }
                            Err(err) => {
                                send_error(&mut sender, format!("Failed to parse notification: {err}")).await;
                                break;
                            }
                        }
                    } else {
                        send_error(&mut sender, "Notification stream closed".to_owned()).await;
                        break;
                    }
                }
            }
        }
        if let Err(err) = sender.send(Message::Close(None)).await {
            eprintln!("WebSocket connection close send error: {err}");
        }
    });
    Ok((StatusCode::OK, ()))
}

async fn send_error(sender: &mut SplitSink<WebSocket, Message>, message: String) {
    let json = serde_json::to_string(&ServerWebSocketMessage::Error {
        message,
    })
    .expect("Failed to serialize ServerWebSocketMessage::Error");
    if let Err(err) = sender.send(Message::Text(json.into())).await {
        eprintln!("WebSocket error JSON send error: {err}");
    }
}
