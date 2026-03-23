// ============================================================
//  CLIENTE — Héroe Autónomo (tablero 10×10)
//  
//  Cambios vs versión original:
//  • Se conecta a un servidor de zona específico
//  • Puerto configurable vía argumento (default 7878)
//  • Maneja ServerPacket::Redirect para cambiar de servidor
//  • Tablero 10×10: coordenadas van de 0..9
// ============================================================
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use rand::Rng as _;
use serde::{Deserialize, Serialize};

// ─── Paquetes (deben coincidir con server.rs) ─────────────
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ClientPacket {
    QueryNeighbors { client_id: u32 },
    Move            { client_id: u32, row: usize, col: usize },
    Attack          { client_id: u32, target_row: usize, target_col: usize, damage: i32 },
    UsePotion       { client_id: u32 },
    Dead            { client_id: u32 },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ServerPacket {
    NeighborState { north: CellState, south: CellState, east: CellState, west: CellState },
    MoveOk        { row: usize, col: usize },
    MoveDenied    { reason: String },
    TakeDamage    { damage: i32, from_id: u32 },
    YouDied,
    Welcome       { client_id: u32, row: usize, col: usize },
    Error         { msg: String },
    Redirect      { tcp_addr: String, row: usize, col: usize },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CellState {
    pub direction:  String,
    pub in_bounds:  bool,
    pub occupied:   bool,
    pub client_id:  Option<u32>,
}

// ─── Dados ────────────────────────────────────────────────
fn roll_d20() -> i32 { rand::rng().random_range(1..=20) }
fn roll_d8()  -> i32 { rand::rng().random_range(1..=8)  }

fn d20_multiplier(d20: i32) -> f32 {
    match d20 {
        1        => 0.0,
        2..=6    => 0.2,
        7..=9    => 0.75,
        10..=13  => 1.0,
        14..=16  => 1.5,
        17..=19  => 2.0,
        20       => 3.0,
        _        => 0.0,
    }
}

fn calc_damage() -> i32 {
    let d20  = roll_d20();
    let roll = roll_d8() + roll_d8();
    ((roll as f32) * d20_multiplier(d20)) as i32
}

// ─── Enviar / recibir ─────────────────────────────────────
fn send(stream: &mut TcpStream, pkt: &ClientPacket) {
    if let Ok(data) = serde_json::to_vec(pkt) {
        let _ = stream.write_all(&data);
    }
}

fn recv(stream: &mut TcpStream) -> Option<ServerPacket> {
    let mut buf = [0u8; 1024];
    match stream.read(&mut buf) {
        Ok(0) | Err(_) => None,
        Ok(n) => serde_json::from_slice(&buf[..n]).ok(),
    }
}

// ─── Conectar con reintentos ──────────────────────────────
fn connect_with_retry(addr: &str) -> TcpStream {
    loop {
        match TcpStream::connect(addr) {
            Ok(s) => {
                println!("[CLIENTE] Conectado a {}", addr);
                return s;
            }
            Err(e) => {
                eprintln!("[CLIENTE] No se pudo conectar a {}: {}. Reintentando en 2s...", addr, e);
                std::thread::sleep(Duration::from_secs(2));
            }
        }
    }
}

// ─── Sesión de juego con un servidor específico ───────────
/// Retorna (nueva_dirección, row, col) si el servidor redirige,
/// o None si el héroe muere o la conexión se pierde.
fn run_session(
    server_addr: &str,
    entry_row: Option<usize>,
    entry_col: Option<usize>,
    hero_life: &mut i32,
    hero_max_life: i32,
    potions: &mut u32,
) -> Option<(String, usize, usize)> {
    let mut stream = connect_with_retry(server_addr);
    stream.set_read_timeout(Some(Duration::from_millis(200))).unwrap();

    // ── Esperar Welcome ────────────────────────────────────
    let (client_id, mut row, mut col) = loop {
        match recv(&mut stream) {
            Some(ServerPacket::Welcome { client_id, row, col }) => {
                let actual_row = entry_row.unwrap_or(row);
                let actual_col = entry_col.unwrap_or(col);
                println!("[CLIENTE {}] Bienvenido en {}. Posición: ({}, {})",
                    client_id, server_addr, actual_row, actual_col);
                break (client_id, actual_row, actual_col);
            }
            Some(ServerPacket::Error { msg }) => {
                eprintln!("[CLIENTE] Error: {}", msg);
                return None;
            }
            _ => {}
        }
    };

    println!("[CLIENTE {}] HP: {}/{} | Pociones: {}", client_id, hero_life, hero_max_life, potions);

    // ── Loop principal ────────────────────────────────────
    loop {
        // 1. Daño pendiente (non-blocking)
        loop {
            match recv(&mut stream) {
                Some(ServerPacket::TakeDamage { damage, from_id }) => {
                    *hero_life -= damage;
                    println!("[CLIENTE {}] ¡{} daño de {}! HP: {}/{}",
                        client_id, damage, from_id, hero_life, hero_max_life);
                    if *hero_life <= 0 {
                        println!("[CLIENTE {}] ¡He muerto!", client_id);
                        send(&mut stream, &ClientPacket::Dead { client_id });
                        return None;
                    }
                }
                Some(ServerPacket::YouDied) => {
                    println!("[CLIENTE {}] El servidor me eliminó.", client_id);
                    return None;
                }
                Some(ServerPacket::Redirect { tcp_addr, row: r, col: c }) => {
                    println!("[CLIENTE {}] 🔀 Redirigido a {} en ({},{})", client_id, tcp_addr, r, c);
                    return Some((tcp_addr, r, c));
                }
                _ => break,
            }
        }

        // 2. Consultar vecinos
        send(&mut stream, &ClientPacket::QueryNeighbors { client_id });

        let neighbors = loop {
            match recv(&mut stream) {
                Some(ServerPacket::NeighborState { north, south, east, west }) => {
                    break vec![north, south, east, west];
                }
                Some(ServerPacket::TakeDamage { damage, from_id }) => {
                    *hero_life -= damage;
                    println!("[CLIENTE {}] (query) Daño {} de {}. HP: {}/{}",
                        client_id, damage, from_id, hero_life, hero_max_life);
                    if *hero_life <= 0 {
                        send(&mut stream, &ClientPacket::Dead { client_id });
                        return None;
                    }
                }
                _ => break vec![],
            }
        };

        if neighbors.is_empty() {
            std::thread::sleep(Duration::from_millis(500));
            continue;
        }

        let life_ratio = *hero_life as f32 / hero_max_life as f32;
        let occupied: Vec<&CellState> = neighbors.iter().filter(|n| n.occupied && n.in_bounds).collect();
        let free:     Vec<&CellState> = neighbors.iter().filter(|n| !n.occupied && n.in_bounds).collect();

        if life_ratio < 0.5 && *potions > 0 {
            // ── Poción ─────────────────────────────────────
            *potions -= 1;
            let heal = 50.min(hero_max_life - *hero_life);
            *hero_life += heal;
            println!("[CLIENTE {}] 🧪 Poción (+{}). HP: {}/{} | Pociones: {}",
                client_id, heal, hero_life, hero_max_life, potions);
            send(&mut stream, &ClientPacket::UsePotion { client_id });

        } else if life_ratio >= 0.5 && !occupied.is_empty() {
            // ── Atacar ─────────────────────────────────────
            let target = occupied[rand::rng().random_range(0..occupied.len())];
            let damage = calc_damage();
            let (tr, tc) = dir_to_coords(row, col, &target.direction);
            println!("[CLIENTE {}] ⚔️  Ataca {} (cliente {:?}) por {} dmg",
                client_id, target.direction, target.client_id, damage);
            send(&mut stream, &ClientPacket::Attack {
                client_id, target_row: tr, target_col: tc, damage,
            });

        } else if !free.is_empty() {
            // ── Mover ──────────────────────────────────────
            let chosen = free[rand::rng().random_range(0..free.len())];
            let (nr, nc) = dir_to_coords(row, col, &chosen.direction);
            println!("[CLIENTE {}] 🚶 → {} ({},{})", client_id, chosen.direction, nr, nc);
            send(&mut stream, &ClientPacket::Move { client_id, row: nr, col: nc });

            loop {
                match recv(&mut stream) {
                    Some(ServerPacket::MoveOk { row: r, col: c }) => {
                        row = r; col = c;
                        break;
                    }
                    Some(ServerPacket::MoveDenied { reason }) => {
                        println!("[CLIENTE {}] Movimiento denegado: {}", client_id, reason);
                        break;
                    }
                    Some(ServerPacket::TakeDamage { damage, from_id }) => {
                        *hero_life -= damage;
                        if *hero_life <= 0 {
                            send(&mut stream, &ClientPacket::Dead { client_id });
                            return None;
                        }
                        println!("[CLIENTE {}] (move) Daño {} de {}", client_id, damage, from_id);
                    }
                    Some(ServerPacket::Redirect { tcp_addr, row: r, col: c }) => {
                        return Some((tcp_addr, r, c));
                    }
                    _ => break,
                }
            }
        } else {
            println!("[CLIENTE {}] 😴 Sin acciones, esperando...", client_id);
        }

        std::thread::sleep(Duration::from_millis(800));
    }
}

// ─── Main ─────────────────────────────────────────────────
fn main() {
    let args: Vec<String> = std::env::args().collect();
    // Puerto del servidor al que conectarse (default 7878)
    let port: u16 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(7878);
    let mut server_addr = format!("127.0.0.1:{}", port);

    let max_life: i32 = rand::rng().random_range(100..=200);
    let mut life      = max_life;
    let mut potions   = 3u32;
    let mut entry: Option<(usize, usize)> = None;

    // Bucle de reconexión / redirección
    loop {
        let result = run_session(
            &server_addr,
            entry.map(|(r,_)| r),
            entry.map(|(_,c)| c),
            &mut life,
            max_life,
            &mut potions,
        );

        match result {
            None => {
                // Héroe muerto o desconexión permanente
                println!("[CLIENTE] Sesión terminada.");
                return;
            }
            Some((new_addr, r, c)) => {
                // Redirigir a otro servidor
                server_addr = new_addr;
                entry = Some((r, c));
            }
        }
    }
}

/// Dirección → coordenadas absolutas (tablero 10×10)
fn dir_to_coords(row: usize, col: usize, dir: &str) -> (usize, usize) {
    match dir {
        "N" => (row.saturating_sub(1), col),
        "S" => ((row + 1).min(9), col),
        "E" => (row, (col + 1).min(9)),
        "W" => (row, col.saturating_sub(1)),
        _   => (row, col),
    }
}