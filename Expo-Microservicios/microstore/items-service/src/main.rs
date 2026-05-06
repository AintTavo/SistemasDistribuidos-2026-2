use actix_cors::Cors;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
struct Item {
    id: String,
    name: String,
    description: String,
    price: f64,
    category: String,
    stock: u32,
    image_url: String,
}

#[derive(Deserialize)]
struct LengthQuery {
    length: Option<usize>,
}

#[derive(Serialize)]
struct ApiResponse<T: Serialize> {
    success: bool,
    message: String,
    data: Option<T>,
    total: usize,
}

static ITEMS_DB: Lazy<Vec<Item>> = Lazy::new(|| {
    vec![
        Item {
            id: "item-001".to_string(),
            name: "Laptop Pro X".to_string(),
            description: "Laptop de alto rendimiento con 16GB RAM y 512GB SSD".to_string(),
            price: 24999.00,
            category: "Electrónica".to_string(),
            stock: 15,
            image_url: "https://placehold.co/400x300/1a1a2e/e94560?text=Laptop+Pro+X".to_string(),
        },
        Item {
            id: "item-002".to_string(),
            name: "Auriculares Quantum".to_string(),
            description: "Auriculares inalámbricos con cancelación de ruido activa".to_string(),
            price: 3499.00,
            category: "Audio".to_string(),
            stock: 42,
            image_url: "https://placehold.co/400x300/1a1a2e/e94560?text=Auriculares".to_string(),
        },
        Item {
            id: "item-003".to_string(),
            name: "Smartwatch Nexus".to_string(),
            description: "Reloj inteligente con monitor cardíaco y GPS integrado".to_string(),
            price: 5999.00,
            category: "Wearables".to_string(),
            stock: 28,
            image_url: "https://placehold.co/400x300/1a1a2e/e94560?text=Smartwatch".to_string(),
        },
        Item {
            id: "item-004".to_string(),
            name: "Teclado Mecánico RGB".to_string(),
            description: "Teclado mecánico con switches Cherry MX y retroiluminación RGB".to_string(),
            price: 2199.00,
            category: "Periféricos".to_string(),
            stock: 60,
            image_url: "https://placehold.co/400x300/1a1a2e/e94560?text=Teclado".to_string(),
        },
        Item {
            id: "item-005".to_string(),
            name: "Monitor UltraWide 34\"".to_string(),
            description: "Monitor curvo ultrawide 4K con 144Hz y HDR10".to_string(),
            price: 18500.00,
            category: "Monitores".to_string(),
            stock: 8,
            image_url: "https://placehold.co/400x300/1a1a2e/e94560?text=Monitor+UW".to_string(),
        },
        Item {
            id: "item-006".to_string(),
            name: "SSD NVMe 2TB".to_string(),
            description: "Disco sólido NVMe con velocidades de hasta 7000 MB/s".to_string(),
            price: 1899.00,
            category: "Almacenamiento".to_string(),
            stock: 100,
            image_url: "https://placehold.co/400x300/1a1a2e/e94560?text=SSD+NVMe".to_string(),
        },
        Item {
            id: "item-007".to_string(),
            name: "Mouse Inalámbrico Pro".to_string(),
            description: "Mouse ergonómico con sensor de 25,600 DPI y batería de 70 horas".to_string(),
            price: 1299.00,
            category: "Periféricos".to_string(),
            stock: 75,
            image_url: "https://placehold.co/400x300/1a1a2e/e94560?text=Mouse+Pro".to_string(),
        },
        Item {
            id: "item-008".to_string(),
            name: "Webcam 4K Stream".to_string(),
            description: "Cámara web 4K con micrófono integrado y autofoco".to_string(),
            price: 2799.00,
            category: "Video".to_string(),
            stock: 33,
            image_url: "https://placehold.co/400x300/1a1a2e/e94560?text=Webcam+4K".to_string(),
        },
        Item {
            id: "item-009".to_string(),
            name: "Hub USB-C 12 en 1".to_string(),
            description: "Hub multipuerto con HDMI 4K, USB 3.2, SD card y carga rápida 100W".to_string(),
            price: 899.00,
            category: "Accesorios".to_string(),
            stock: 90,
            image_url: "https://placehold.co/400x300/1a1a2e/e94560?text=Hub+USB-C".to_string(),
        },
        Item {
            id: "item-010".to_string(),
            name: "Tablet Creator 12".to_string(),
            description: "Tableta gráfica profesional con pantalla OLED y lápiz de 8192 niveles".to_string(),
            price: 12999.00,
            category: "Creación".to_string(),
            stock: 20,
            image_url: "https://placehold.co/400x300/1a1a2e/e94560?text=Tablet+Pro".to_string(),
        },
    ]
});

// GET /items?length=N - Retorna N items de la base de datos
#[get("")]
async fn get_items(query: web::Query<LengthQuery>) -> impl Responder {
    let all_items = &*ITEMS_DB;
    let length = query.length.unwrap_or(all_items.len()).min(all_items.len());
    let items: Vec<&Item> = all_items.iter().take(length).collect();
    let total = items.len();

    HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: format!("Se retornaron {} items", total),
        data: Some(items),
        total,
    })
}

// GET /items/:id - Para uso interno del microservicio de compras
#[get("/{id}")]
async fn get_item_by_id(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    let all_items = &*ITEMS_DB;

    match all_items.iter().find(|i| i.id == id) {
        Some(item) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Item encontrado",
            "data": item
        })),
        None => HttpResponse::NotFound().json(serde_json::json!({
            "success": false,
            "message": "Item no encontrado",
            "data": null
        })),
    }
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "status": "ok", "service": "items" }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 Items Service corriendo en http://0.0.0.0:8002");

    HttpServer::new(|| {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .service(
                web::scope("/items")
                    .service(health)
                    .service(get_items)
                    .service(get_item_by_id),
            )
    })
    .bind("0.0.0.0:8002")?
    .run()
    .await
}
