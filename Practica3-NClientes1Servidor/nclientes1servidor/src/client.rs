// ============================================================
//  CLIENTE — Héroe Autónomo
//  Lógica: QueryNeighbors → decidir acción → enviar al servidor
// ============================================================
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use rand::Rng;
use serde::{Deserialize, Serialize};

// ─── Paquetes (deben coincidir con server.rs) ────────────────
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CellState {
    pub direction:  String,
    pub in_bounds:  bool,
    pub occupied:   bool,
    pub client_id:  Option<u32>,
}

// ─── Dados ───────────────────────────────────────────────────
fn roll_d20() -> i32 { rand::thread_rng().gen_range(1..=20) }
fn roll_d8()  -> i32 { rand::thread_rng().gen_range(1..=8)  }

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

// ─── Enviar / recibir ────────────────────────────────────────
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

// ─── Main ─────────────────────────────────────────────────
fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:7878")
        .expect("No se pudo conectar al servidor (¿está corriendo server?)");

    stream.set_read_timeout(Some(Duration::from_millis(200))).unwrap();

    // ── Esperar bienvenida del servidor ──────────────────
    let (client_id, mut row, mut col) = loop {
        match recv(&mut stream) {
            Some(ServerPacket::Welcome { client_id, row, col }) => {
                println!("[CLIENTE {}] Bienvenido! Posición inicial: ({}, {})", client_id, row, col);
                break (client_id, row, col);
            }
            Some(ServerPacket::Error { msg }) => {
                eprintln!("[CLIENTE] Error del servidor: {}", msg);
                return;
            }
            _ => {}
        }
    };

    // ── Stats del héroe ──────────────────────────────────
    let max_life: i32 = rand::thread_rng().gen_range(100..=200);
    let mut life      = max_life;
    let mut potions   = 3u32;

    println!("[CLIENTE {}] HP: {}/{} | Pociones: {}", client_id, life, max_life, potions);

    // ── Loop principal ───────────────────────────────────
    loop {
        // 1. Recibir daño pendiente del servidor (non-blocking)
        loop {
            match recv(&mut stream) {
                Some(ServerPacket::TakeDamage { damage, from_id }) => {
                    life -= damage;
                    println!(
                        "[CLIENTE {}] ¡Recibí {} de daño del cliente {}! HP: {}/{}",
                        client_id, damage, from_id, life, max_life
                    );
                    if life <= 0 {
                        println!("[CLIENTE {}] ¡He muerto!", client_id);
                        send(&mut stream, &ClientPacket::Dead { client_id });
                        return;
                    }
                }
                Some(ServerPacket::YouDied) => {
                    println!("[CLIENTE {}] El servidor me ha eliminado.", client_id);
                    return;
                }
                _ => break,
            }
        }

        // 2. Consultar vecinos al servidor
        send(&mut stream, &ClientPacket::QueryNeighbors { client_id });

        let neighbors = loop {
            match recv(&mut stream) {
                Some(ServerPacket::NeighborState { north, south, east, west }) => {
                    break vec![north, south, east, west];
                }
                Some(ServerPacket::TakeDamage { damage, from_id }) => {
                    // Puede llegar daño mientras esperamos respuesta
                    life -= damage;
                    println!(
                        "[CLIENTE {}] (durante query) Daño {} de {}. HP: {}/{}",
                        client_id, damage, from_id, life, max_life
                    );
                    if life <= 0 {
                        send(&mut stream, &ClientPacket::Dead { client_id });
                        return;
                    }
                }
                _ => break vec![],
            }
        };

        if neighbors.is_empty() {
            std::thread::sleep(Duration::from_millis(500));
            continue;
        }

        let life_ratio = life as f32 / max_life as f32;

        // ── DECISIÓN DEL CLIENTE ─────────────────────────
        //
        //  Prioridad 1: HP < 50% y tiene pociones → usar poción
        //  Prioridad 2: HP >= 50% y hay vecino ocupado → atacar
        //  Prioridad 3: moverse a casilla libre

        let occupied: Vec<&CellState> = neighbors.iter().filter(|n| n.occupied && n.in_bounds).collect();
        let free:     Vec<&CellState> = neighbors.iter().filter(|n| !n.occupied && n.in_bounds).collect();

        if life_ratio < 0.5 && potions > 0 {
            // ── Poción ──────────────────────────────────
            potions  -= 1;
            let heal  = 50.min(max_life - life);
            life     += heal;
            println!(
                "[CLIENTE {}] 🧪 Usé poción (+{}). HP: {}/{} | Pociones restantes: {}",
                client_id, heal, life, max_life, potions
            );
            send(&mut stream, &ClientPacket::UsePotion { client_id });

        } else if life_ratio >= 0.5 && !occupied.is_empty() {
            // ── Atacar ──────────────────────────────────
            let target = occupied[rand::thread_rng().gen_range(0..occupied.len())];
            let damage = calc_damage();

            // Calcular coordenadas del objetivo desde la dirección
            let (tr, tc) = dir_to_coords(row, col, &target.direction);

            println!(
                "[CLIENTE {}] ⚔️  Ataca dirección {} (cliente {:?}) con {} de daño",
                client_id, target.direction, target.client_id, damage
            );
            send(&mut stream, &ClientPacket::Attack {
                client_id,
                target_row: tr,
                target_col: tc,
                damage,
            });

        } else if !free.is_empty() {
            // ── Mover ────────────────────────────────────
            let chosen = free[rand::thread_rng().gen_range(0..free.len())];
            let (nr, nc) = dir_to_coords(row, col, &chosen.direction);

            println!(
                "[CLIENTE {}] 🚶 Mueve hacia {} → ({}, {})",
                client_id, chosen.direction, nr, nc
            );
            send(&mut stream, &ClientPacket::Move { client_id, row: nr, col: nc });

            // Actualizar posición local según respuesta
            loop {
                match recv(&mut stream) {
                    Some(ServerPacket::MoveOk { row: r, col: c }) => {
                        row = r;
                        col = c;
                        break;
                    }
                    Some(ServerPacket::MoveDenied { reason }) => {
                        println!("[CLIENTE {}] Movimiento denegado: {}", client_id, reason);
                        break;
                    }
                    Some(ServerPacket::TakeDamage { damage, from_id }) => {
                        life -= damage;
                        if life <= 0 {
                            send(&mut stream, &ClientPacket::Dead { client_id });
                            return;
                        }
                        println!("[CLIENTE {}] (durante move) Daño {} de {}", client_id, damage, from_id);
                    }
                    _ => break,
                }
            }
        } else {
            println!("[CLIENTE {}] 😴 Sin acciones posibles, esperando...", client_id);
        }

        std::thread::sleep(Duration::from_millis(800));
    }
}

/// Convierte una dirección ("N","S","E","W") a coordenadas absolutas
fn dir_to_coords(row: usize, col: usize, dir: &str) -> (usize, usize) {
    match dir {
        "N" => (row.saturating_sub(1), col),
        "S" => (row + 1, col),
        "E" => (row, col + 1),
        "W" => (row, col.saturating_sub(1)),
        _   => (row, col),
    }
}