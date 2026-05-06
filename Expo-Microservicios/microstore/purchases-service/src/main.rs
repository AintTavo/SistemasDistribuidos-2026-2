use actix_cors::Cors;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use chrono::Utc;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use uuid::Uuid;

// URLs de los otros microservicios (configurables via variables de entorno)
fn users_service_url() -> String {
    std::env::var("USERS_SERVICE_URL").unwrap_or_else(|_| "http://localhost:8001".to_string())
}
fn items_service_url() -> String {
    std::env::var("ITEMS_SERVICE_URL").unwrap_or_else(|_| "http://localhost:8002".to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Purchase {
    id: String,
    user_email: String,
    user_name: String,
    item_ids: Vec<String>,
    item_details: Vec<ItemSummary>,
    total: f64,
    created_at: String,
    status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ItemSummary {
    id: String,
    name: String,
    price: f64,
}

#[derive(Deserialize)]
struct CreatePurchaseRequest {
    user_email: String,
    item_ids: Vec<String>,
}

#[derive(Serialize)]
struct ApiResponse<T: Serialize> {
    success: bool,
    message: String,
    data: Option<T>,
}

// Structs para deserializar respuestas de otros servicios
#[derive(Deserialize)]
struct UserServiceResponse {
    success: bool,
    data: Option<UserData>,
}

#[derive(Deserialize)]
struct UserData {
    id: String,
    name: String,
    email: String,
}

#[derive(Deserialize)]
struct ItemServiceResponse {
    success: bool,
    data: Option<ItemData>,
}

#[derive(Deserialize)]
struct ItemData {
    id: String,
    name: String,
    price: f64,
}

static PURCHASES_DB: Lazy<Mutex<Vec<Purchase>>> = Lazy::new(|| Mutex::new(vec![]));

// POST /purchases - Crear nueva compra
// Llama síncronamente a usuarios (GET email) e items (GET ids)
#[post("")]
async fn create_purchase(body: web::Json<CreatePurchaseRequest>) -> impl Responder {
    if body.item_ids.is_empty() {
        return HttpResponse::BadRequest().json(ApiResponse::<()> {
            success: false,
            message: "Debe incluir al menos un item".to_string(),
            data: None,
        });
    }

    let client = reqwest::Client::new();

    // === LLAMADA SÍNCRONA 1: Verificar usuario en users-service ===
    let user_url = format!("{}/users/email/{}", users_service_url(), body.user_email);
    let user_response = match client.get(&user_url).send().await {
        Ok(r) => r,
        Err(e) => {
            return HttpResponse::ServiceUnavailable().json(ApiResponse::<()> {
                success: false,
                message: format!("No se pudo contactar al servicio de usuarios: {}", e),
                data: None,
            })
        }
    };

    if !user_response.status().is_success() {
        return HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            message: "Usuario no encontrado en el sistema".to_string(),
            data: None,
        });
    }

    let user_data: UserServiceResponse = match user_response.json().await {
        Ok(d) => d,
        Err(_) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                message: "Error al procesar respuesta del servicio de usuarios".to_string(),
                data: None,
            })
        }
    };

    let user = match user_data.data {
        Some(u) => u,
        None => {
            return HttpResponse::NotFound().json(ApiResponse::<()> {
                success: false,
                message: "Usuario no encontrado".to_string(),
                data: None,
            })
        }
    };

    // === LLAMADA SÍNCRONA 2: Verificar items en items-service ===
    let mut item_details: Vec<ItemSummary> = Vec::new();
    let mut total: f64 = 0.0;

    for item_id in &body.item_ids {
        let item_url = format!("{}/items/{}", items_service_url(), item_id);
        let item_response = match client.get(&item_url).send().await {
            Ok(r) => r,
            Err(e) => {
                return HttpResponse::ServiceUnavailable().json(ApiResponse::<()> {
                    success: false,
                    message: format!("No se pudo contactar al servicio de items: {}", e),
                    data: None,
                })
            }
        };

        if !item_response.status().is_success() {
            return HttpResponse::NotFound().json(ApiResponse::<()> {
                success: false,
                message: format!("Item '{}' no encontrado", item_id),
                data: None,
            });
        }

        let item_data: ItemServiceResponse = match item_response.json().await {
            Ok(d) => d,
            Err(_) => {
                return HttpResponse::InternalServerError().json(ApiResponse::<()> {
                    success: false,
                    message: "Error al procesar respuesta del servicio de items".to_string(),
                    data: None,
                })
            }
        };

        if let Some(item) = item_data.data {
            total += item.price;
            item_details.push(ItemSummary {
                id: item.id,
                name: item.name,
                price: item.price,
            });
        }
    }

    // === Crear la compra con ID único ===
    let purchase = Purchase {
        id: format!("PUR-{}", Uuid::new_v4().to_string().split('-').next().unwrap_or("XXXX").to_uppercase()),
        user_email: user.email,
        user_name: user.name,
        item_ids: body.item_ids.clone(),
        item_details,
        total,
        created_at: Utc::now().to_rfc3339(),
        status: "completed".to_string(),
    };

    let mut db = PURCHASES_DB.lock().unwrap();
    db.push(purchase.clone());

    HttpResponse::Created().json(ApiResponse {
        success: true,
        message: "Compra registrada exitosamente".to_string(),
        data: Some(purchase),
    })
}

// GET /purchases - Listar todas las compras
#[get("")]
async fn get_purchases() -> impl Responder {
    let db = PURCHASES_DB.lock().unwrap();
    let purchases: Vec<Purchase> = db.clone();

    HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: format!("{} compras encontradas", purchases.len()),
        data: Some(purchases),
    })
}

// GET /purchases/:id - Obtener compra por ID
#[get("/{id}")]
async fn get_purchase_by_id(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    let db = PURCHASES_DB.lock().unwrap();

    match db.iter().find(|p| p.id == id) {
        Some(p) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            message: "Compra encontrada".to_string(),
            data: Some(p.clone()),
        }),
        None => HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            message: "Compra no encontrada".to_string(),
            data: None,
        }),
    }
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "status": "ok", "service": "purchases" }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 Purchases Service corriendo en http://0.0.0.0:8003");
    println!("   → Users Service: {}", users_service_url());
    println!("   → Items Service:  {}", items_service_url());

    HttpServer::new(|| {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .service(
                web::scope("/purchases")
                    .service(health)
                    .service(create_purchase)
                    .service(get_purchases)
                    .service(get_purchase_by_id),
            )
    })
    .bind("0.0.0.0:8003")?
    .run()
    .await
}
