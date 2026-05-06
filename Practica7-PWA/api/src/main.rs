// ---------------------------------------- Librerias y Crates ----------------------------------------
use std::net::SocketAddr;
use std::env;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::{
    routing::{get, post},
    Json,
    Router,
    extract::State,
    http::{StatusCode, Method, HeaderValue},
};
use serde::{Deserialize, Serialize};
use colored::Colorize;
use rand::Rng;
use tower_http::cors::{CorsLayer, Any};

// ---------------------------------------- Estado Global ----------------------------------------

#[derive(Clone)]
struct AppState {
    // Mapa de api_key -> nivel actual del juego
    sessions: Arc<Mutex<HashMap<String, u32>>>,
}

// ---------------------------------------- Structs de Items ----------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "item_type")]
enum Item {
    Weapon(Weapon),
    Potion(Potion),
    Magic(Magic),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Weapon {
    name: String,
    icon: String,
    damage_min: u32,
    damage_max: u32,
    rarity: String,
    level_required: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Potion {
    name: String,
    icon: String,
    heal_min: u32,
    heal_max: u32,
    tier: u8,
    rarity: String,
    level_required: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Magic {
    name: String,
    icon: String,
    damage_min: u32,
    damage_max: u32,
    rarity: String,
    level_required: u32,
    element: String,
}

// ---------------------------------------- Requests y Responses ----------------------------------------

#[derive(Deserialize)]
struct PetitionPost {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct ResponsePost {
    api_key: String,
    success: bool,
    message: String,
}

#[derive(Deserialize)]
struct PetitionGetItem {
    api_key: String,
    level: u32,
}

#[derive(Serialize)]
struct ResponseGetItem {
    item: Option<serde_json::Value>,
    success: bool,
    message: String,
}

#[derive(Deserialize)]
struct PetitionUpdateLevel {
    api_key: String,
    level: u32,
}

#[derive(Serialize)]
struct ResponseUpdateLevel {
    success: bool,
    message: String,
    current_level: u32,
}

// ---------------------------------------- Pool de Items ----------------------------------------

fn get_items_pool(level: u32) -> Vec<serde_json::Value> {
    let mut pool: Vec<serde_json::Value> = Vec::new();

    // --- ARMAS (comunes) ---
    let weapons = vec![
        // Nivel 1+
        ("Espada Oxidada",       "⚔️",  3, 8,  "common", 1),
        ("Daga Astillada",       "🗡️",  2, 6,  "common", 1),
        ("Hacha de Madera",      "🪓",  4, 9,  "common", 1),
        // Nivel 2+
        ("Espada Corta",         "⚔️",  6, 14, "uncommon", 2),
        ("Lanza de Hueso",       "🏹",  5, 12, "uncommon", 2),
        ("Martillo de Piedra",   "🔨",  7, 15, "uncommon", 2),
        // Nivel 3+
        ("Espada Larga",         "⚔️",  10, 22, "rare", 3),
        ("Arco Élfico",          "🏹",  8,  20, "rare", 3),
        ("Hacha de Guerra",      "🪓",  12, 25, "rare", 3),
        // Nivel 4+
        ("Mandoble Oscuro",      "⚔️",  18, 35, "epic", 4),
        ("Ballesta de Sombra",   "🏹",  15, 32, "epic", 4),
        // Nivel 5+
        ("Filo del Abismo",      "⚔️",  28, 55, "legendary", 5),
        ("Martillo del Caos",    "🔨",  25, 50, "legendary", 5),
    ];

    for (name, icon, dmin, dmax, rarity, lvl_req) in &weapons {
        if level >= *lvl_req {
            // Las armas tienen peso x3 para ser más comunes
            for _ in 0..3 {
                pool.push(serde_json::json!({
                    "item_type": "weapon",
                    "name": name,
                    "icon": icon,
                    "damage_min": dmin,
                    "damage_max": dmax,
                    "rarity": rarity,
                    "level_required": lvl_req,
                }));
            }
        }
    }

    // --- POCIONES (comunes) ---
    let potions = vec![
        // Tier 1 - Nivel 1+
        ("Poción Pequeña",   "🧪", 10, 20, 1u8, "common",   1u32),
        ("Hierba Curativa",  "🌿", 8,  15, 1,   "common",   1),
        // Tier 2 - Nivel 2+
        ("Poción Mediana",   "🍶", 25, 45, 2,   "uncommon", 2),
        ("Elixir Rojo",      "🔴", 20, 40, 2,   "uncommon", 2),
        ("Tónico Vital",     "💊", 30, 50, 2,   "uncommon", 2),
        // Tier 3 - Nivel 4+
        ("Poción Mayor",     "⚗️", 60, 100, 3,  "rare",     4),
        ("Elixir Supremo",   "💉", 80, 130, 3,  "rare",     4),
    ];

    for (name, icon, hmin, hmax, tier, rarity, lvl_req) in &potions {
        if level >= *lvl_req {
            // Las pociones tienen peso x2
            for _ in 0..2 {
                pool.push(serde_json::json!({
                    "item_type": "potion",
                    "name": name,
                    "icon": icon,
                    "heal_min": hmin,
                    "heal_max": hmax,
                    "tier": tier,
                    "rarity": rarity,
                    "level_required": lvl_req,
                }));
            }
        }
    }

    // --- MAGIAS (raras: peso x1, sólo si nivel >= req) ---
    let magics = vec![
        ("Destello Arcano",    "✨", 8,  18, "uncommon", 1, "arcane"),
        ("Bola de Fuego",      "🔥", 12, 25, "rare",     2, "fire"),
        ("Rayo Gélido",        "❄️", 10, 22, "rare",     2, "ice"),
        ("Cadena de Rayos",    "⚡", 15, 30, "rare",     3, "lightning"),
        ("Nova de Sombra",     "🌑", 18, 35, "epic",     3, "shadow"),
        ("Tormenta Arcana",    "🌀", 22, 42, "epic",     4, "arcane"),
        ("Apocalipsis Ígneo",  "💥", 35, 65, "legendary",5, "fire"),
        ("Colapso Dimensional","🕳️", 40, 70, "legendary",5, "void"),
    ];

    for (name, icon, dmin, dmax, rarity, lvl_req, element) in &magics {
        if level >= *lvl_req {
            // Peso x1: mucho más raro que armas y pociones
            pool.push(serde_json::json!({
                "item_type": "magic",
                "name": name,
                "icon": icon,
                "damage_min": dmin,
                "damage_max": dmax,
                "rarity": rarity,
                "level_required": lvl_req,
                "element": element,
            }));
        }
    }

    pool
}

fn roll_item(level: u32) -> Option<serde_json::Value> {
    let pool = get_items_pool(level);
    if pool.is_empty() {
        return None;
    }
    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..pool.len());
    Some(pool[idx].clone())
}

// ---------------------------------------- Handlers ----------------------------------------

async fn petition_post(
    State(state): State<AppState>,
    Json(payload): Json<PetitionPost>,
) -> (StatusCode, Json<ResponsePost>) {
    let users_raw = env::var("API_USERS").unwrap_or_default();
    let allowed_users: Vec<&str> = users_raw.split(',').map(|s| s.trim()).collect();

    if !allowed_users.contains(&payload.username.as_str()) {
        println!("{}{}: Usuario '{}' no registrado",
            "[POST]".blue().bold(), "[Error]".red(), payload.username);
        return (StatusCode::UNAUTHORIZED, Json(ResponsePost {
            api_key: "".into(),
            success: false,
            message: "Usuario no registrado".into(),
        }));
    }

    let password = env::var("PASSWORD").expect("PASSWORD no configurada");
    if password != payload.password {
        println!("{}{}: Contraseña inválida para '{}'",
            "[POST]".blue().bold(), "[Error]".red(), payload.username);
        return (StatusCode::UNAUTHORIZED, Json(ResponsePost {
            api_key: "".into(),
            success: false,
            message: "Contraseña inválida".into(),
        }));
    }

    let api_key = env::var("API_PASSWORD").expect("API_PASSWORD no configurada");

    // Registrar sesión en nivel 1
    {
        let mut sessions = state.sessions.lock().unwrap();
        sessions.entry(api_key.clone()).or_insert(1);
    }

    println!("{}{}: Usuario '{}' autenticado",
        "[POST]".blue().bold(), "[Success]".green(), payload.username);
    (StatusCode::OK, Json(ResponsePost {
        api_key,
        success: true,
        message: "Autenticado correctamente".into(),
    }))
}

async fn petition_get_item(
    State(state): State<AppState>,
    Json(payload): Json<PetitionGetItem>,
) -> (StatusCode, Json<ResponseGetItem>) {
    let api_key = env::var("API_PASSWORD").expect("API_PASSWORD no configurada");

    if payload.api_key != api_key {
        println!("{}{}: API key inválida", "[GET]".blue().bold(), "[Error]".red());
        return (StatusCode::UNAUTHORIZED, Json(ResponseGetItem {
            item: None,
            success: false,
            message: "API key inválida".into(),
        }));
    }

    // Actualizar nivel en sesión
    {
        let mut sessions = state.sessions.lock().unwrap();
        sessions.insert(payload.api_key.clone(), payload.level);
    }

    let item = roll_item(payload.level);
    println!("{}{}: Generando item para nivel {}",
        "[GET]".blue().bold(), "[Success]".green(), payload.level);

    (StatusCode::OK, Json(ResponseGetItem {
        item,
        success: true,
        message: "Item generado".into(),
    }))
}

async fn petition_update_level(
    State(state): State<AppState>,
    Json(payload): Json<PetitionUpdateLevel>,
) -> (StatusCode, Json<ResponseUpdateLevel>) {
    let api_key = env::var("API_PASSWORD").expect("API_PASSWORD no configurada");

    if payload.api_key != api_key {
        return (StatusCode::UNAUTHORIZED, Json(ResponseUpdateLevel {
            success: false,
            message: "API key inválida".into(),
            current_level: 0,
        }));
    }

    let current_level = {
        let mut sessions = state.sessions.lock().unwrap();
        let entry = sessions.entry(payload.api_key.clone()).or_insert(1);
        *entry = payload.level;
        *entry
    };

    println!("{}{}: Nivel actualizado a {}",
        "[PUT]".blue().bold(), "[Success]".green(), current_level);

    (StatusCode::OK, Json(ResponseUpdateLevel {
        success: true,
        message: "Nivel actualizado".into(),
        current_level,
    }))
}

async fn shutdown() {
    tokio::signal::ctrl_c()
        .await
        .expect("No se pudo instalar el manejador de Ctrl+C");
}

// ---------------------------------------- Main ----------------------------------------

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let port_str = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let port: u16 = port_str.trim().parse().expect("Puerto inválido");

    let state = AppState {
        sessions: Arc::new(Mutex::new(HashMap::new())),
    };

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT])
        .allow_headers(Any)
        .allow_origin(Any);

    let app = Router::new()
        .route("/login",       post(petition_post))
        .route("/item",        get(petition_get_item))
        .route("/level",       post(petition_update_level))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let tmp = addr.to_string().yellow();
    println!("Servidor corriendo en {}{}", "http://".yellow().bold(), tmp.bold());

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown())
        .await
        .unwrap();
}