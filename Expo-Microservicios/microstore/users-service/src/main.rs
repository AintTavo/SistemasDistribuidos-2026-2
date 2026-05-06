use actix_cors::Cors;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use bcrypt::{hash, verify, DEFAULT_COST};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: String,
    name: String,
    email: String,
    #[serde(skip_serializing)]
    password_hash: String,
}

#[derive(Deserialize)]
struct SignInRequest {
    name: String,
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct UserPublic {
    id: String,
    name: String,
    email: String,
}

#[derive(Serialize)]
struct ApiResponse<T: Serialize> {
    success: bool,
    message: String,
    data: Option<T>,
}

static USERS_DB: Lazy<Mutex<Vec<User>>> = Lazy::new(|| {
    Mutex::new(vec![
        User {
            id: Uuid::new_v4().to_string(),
            name: "Alice García".to_string(),
            email: "alice@microstore.com".to_string(),
            password_hash: hash("password123", DEFAULT_COST).unwrap(),
        },
        User {
            id: Uuid::new_v4().to_string(),
            name: "Bob Martínez".to_string(),
            email: "bob@microstore.com".to_string(),
            password_hash: hash("password123", DEFAULT_COST).unwrap(),
        },
    ])
});

// POST /users/signin - Registrar nuevo usuario
#[post("/signin")]
async fn sign_in(body: web::Json<SignInRequest>) -> impl Responder {
    let mut db = USERS_DB.lock().unwrap();

    // Verificar si el email ya existe
    if db.iter().any(|u| u.email == body.email) {
        return HttpResponse::Conflict().json(ApiResponse::<()> {
            success: false,
            message: "El correo ya está registrado".to_string(),
            data: None,
        });
    }

    let password_hash = match hash(&body.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                message: "Error al procesar la contraseña".to_string(),
                data: None,
            })
        }
    };

    let new_user = User {
        id: Uuid::new_v4().to_string(),
        name: body.name.clone(),
        email: body.email.clone(),
        password_hash,
    };

    let public_user = UserPublic {
        id: new_user.id.clone(),
        name: new_user.name.clone(),
        email: new_user.email.clone(),
    };

    db.push(new_user);

    HttpResponse::Created().json(ApiResponse {
        success: true,
        message: "Usuario registrado exitosamente".to_string(),
        data: Some(public_user),
    })
}

// GET /users/login - Autenticar usuario
#[get("/login")]
async fn login(query: web::Query<LoginRequest>) -> impl Responder {
    let db = USERS_DB.lock().unwrap();

    match db.iter().find(|u| u.email == query.email) {
        Some(user) => {
            let valid = verify(&query.password, &user.password_hash).unwrap_or(false);
            if valid {
                HttpResponse::Ok().json(ApiResponse {
                    success: true,
                    message: "Login exitoso".to_string(),
                    data: Some(UserPublic {
                        id: user.id.clone(),
                        name: user.name.clone(),
                        email: user.email.clone(),
                    }),
                })
            } else {
                HttpResponse::Unauthorized().json(ApiResponse::<()> {
                    success: false,
                    message: "Contraseña incorrecta".to_string(),
                    data: None,
                })
            }
        }
        None => HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            message: "Usuario no encontrado".to_string(),
            data: None,
        }),
    }
}

// GET /users/email/:email - Para uso interno de otros microservicios
#[get("/email/{email}")]
async fn get_by_email(path: web::Path<String>) -> impl Responder {
    let email = path.into_inner();
    let db = USERS_DB.lock().unwrap();

    match db.iter().find(|u| u.email == email) {
        Some(user) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            message: "Usuario encontrado".to_string(),
            data: Some(UserPublic {
                id: user.id.clone(),
                name: user.name.clone(),
                email: user.email.clone(),
            }),
        }),
        None => HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            message: "Usuario no encontrado".to_string(),
            data: None,
        }),
    }
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "status": "ok", "service": "users" }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 Users Service corriendo en http://0.0.0.0:8001");

    HttpServer::new(|| {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .service(
                web::scope("/users")
                    .service(sign_in)
                    .service(login)
                    .service(get_by_email)
                    .service(health),
            )
    })
    .bind("0.0.0.0:8001")?
    .run()
    .await
}
