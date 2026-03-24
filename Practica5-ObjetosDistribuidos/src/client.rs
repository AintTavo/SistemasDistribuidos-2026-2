// ============================================================
//  CLIENTE — Héroe Autónomo con Attack Stub integrado
//  Usa ORB para ataques; fallback local si ORB no disponible
// ============================================================
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use rand::Rng as _;
use serde::{Deserialize, Serialize};

const ORB_ADDR: &str = "127.0.0.1:8500";

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ClientPacket {
    QueryNeighbors { client_id: u32 },
    Move           { client_id: u32, row: usize, col: usize },
    Attack         { client_id: u32, target_row: usize, target_col: usize, damage: i32 },
    UsePotion      { client_id: u32 },
    Dead           { client_id: u32 },
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
    pub direction: String, pub in_bounds: bool, pub occupied: bool, pub client_id: Option<u32>,
}

// ─── ORB tipos ────────────────────────────────────────────────
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
enum OrbRequest {
    RegisterZone { server_id: u32, skeleton_addr: String, zone: OrbZone },
    UpdateCells  { server_id: u32, cells: Vec<CellReg> },
    InvokeAttack { request_id: u64, attacker_id: u32, target_row: usize, target_col: usize, damage: i32 },
    LookupIOR    { row: usize, col: usize },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
enum OrbReply {
    AttackResult   { request_id: u64, target_id: Option<u32>, damage_applied: i32, target_died: bool, routing: String },
    TargetNotFound { request_id: u64, reason: String },
    IORFound       { ior: serde_json::Value },
    IORNotFound    { row: usize, col: usize },
    ZoneRegistered { server_id: u32 },
    CellsUpdated   { server_id: u32, count: usize },
    Error          { request_id: u64, msg: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct OrbZone { row_start: usize, row_end: usize, col_start: usize, col_end: usize }

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CellReg { row: usize, col: usize, occupant_id: Option<u32> }

struct AttackOutcome {
    target_id: Option<u32>, damage_applied: i32, target_died: bool, routing: String,
}

// ─── Attack Stub ─────────────────────────────────────────────
struct AttackStub { orb_addr: String, req_counter: AtomicU64 }

impl AttackStub {
    fn new() -> Self { AttackStub { orb_addr: ORB_ADDR.to_string(), req_counter: AtomicU64::new(1) } }

    fn invoke_attack(&self, attacker_id: u32, target_row: usize, target_col: usize, damage: i32) -> Result<AttackOutcome, String> {
        let request_id = self.req_counter.fetch_add(1, Ordering::SeqCst);
        let mut stream = TcpStream::connect(&self.orb_addr).map_err(|e| format!("ORB inalcanzable: {}", e))?;
        stream.set_read_timeout(Some(Duration::from_secs(4))).ok();
        let req = OrbRequest::InvokeAttack { request_id, attacker_id, target_row, target_col, damage };
        let data = serde_json::to_vec(&req).map_err(|e| e.to_string())?;
        stream.write_all(&data).map_err(|e| e.to_string())?;
        let mut buf = [0u8; 2048];
        let n = stream.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 { return Err("ORB cerró conexión".into()); }
        let reply: OrbReply = serde_json::from_slice(&buf[..n]).map_err(|e| e.to_string())?;
        match reply {
            OrbReply::AttackResult { target_id, damage_applied, target_died, routing, .. } =>
                Ok(AttackOutcome { target_id, damage_applied, target_died, routing }),
            OrbReply::TargetNotFound { reason, .. } => Err(format!("target_not_found:{}", reason)),
            OrbReply::Error { msg, .. } => Err(msg),
            _ => Err("Respuesta inesperada del ORB".into()),
        }
    }
}

// ─── Dados ────────────────────────────────────────────────────
fn roll_d20() -> i32 { rand::rng().random_range(1..=20) }
fn roll_d8()  -> i32 { rand::rng().random_range(1..=8) }
fn d20_mult(d20: i32) -> f32 {
    match d20 { 1 => 0.0, 2..=6 => 0.2, 7..=9 => 0.75, 10..=13 => 1.0, 14..=16 => 1.5, 17..=19 => 2.0, 20 => 3.0, _ => 0.0 }
}
fn calc_damage() -> i32 { let d20 = roll_d20(); let roll = roll_d8() + roll_d8(); ((roll as f32) * d20_mult(d20)) as i32 }

fn send(stream: &mut TcpStream, pkt: &ClientPacket) {
    if let Ok(data) = serde_json::to_vec(pkt) { let _ = stream.write_all(&data); }
}
fn recv(stream: &mut TcpStream) -> Option<ServerPacket> {
    let mut buf = [0u8; 1024];
    match stream.read(&mut buf) { Ok(0) | Err(_) => None, Ok(n) => serde_json::from_slice(&buf[..n]).ok() }
}
fn connect_with_retry(addr: &str) -> TcpStream {
    loop {
        match TcpStream::connect(addr) {
            Ok(s) => { println!("[CLIENTE] Conectado a {}", addr); return s; }
            Err(e) => { eprintln!("[CLIENTE] {} → {}. Reintentando en 2s...", addr, e); std::thread::sleep(Duration::from_secs(2)); }
        }
    }
}

fn run_session(
    server_addr: &str, entry_row: Option<usize>, entry_col: Option<usize>,
    hero_life: &mut i32, hero_max_life: i32, potions: &mut u32,
) -> Option<(String, usize, usize)> {
    let mut stream = connect_with_retry(server_addr);
    stream.set_read_timeout(Some(Duration::from_millis(200))).unwrap();
    let stub = AttackStub::new();

    let (client_id, mut row, mut col) = loop {
        match recv(&mut stream) {
            Some(ServerPacket::Welcome { client_id, row, col }) => {
                let r = entry_row.unwrap_or(row); let c = entry_col.unwrap_or(col);
                println!("[CLIENTE {}] Bienvenido en {}. Pos: ({},{})", client_id, server_addr, r, c);
                break (client_id, r, c);
            }
            Some(ServerPacket::Error { msg }) => { eprintln!("[CLIENTE] Error: {}", msg); return None; }
            _ => {}
        }
    };

    println!("[CLIENTE {}] HP:{}/{} Pociones:{} ORB:{}", client_id, hero_life, hero_max_life, potions, ORB_ADDR);

    loop {
        loop {
            match recv(&mut stream) {
                Some(ServerPacket::TakeDamage { damage, from_id }) => {
                    *hero_life -= damage;
                    println!("[CLIENTE {}] ¡{} daño de {}! HP:{}/{}", client_id, damage, from_id, hero_life, hero_max_life);
                    if *hero_life <= 0 { println!("[CLIENTE {}] ¡He muerto!", client_id); send(&mut stream, &ClientPacket::Dead { client_id }); return None; }
                }
                Some(ServerPacket::YouDied) => { println!("[CLIENTE {}] Eliminado.", client_id); return None; }
                Some(ServerPacket::Redirect { tcp_addr, row: r, col: c }) => { println!("[CLIENTE {}] → {} ({},{})", client_id, tcp_addr, r, c); return Some((tcp_addr, r, c)); }
                _ => break,
            }
        }

        send(&mut stream, &ClientPacket::QueryNeighbors { client_id });
        let neighbors = loop {
            match recv(&mut stream) {
                Some(ServerPacket::NeighborState { north, south, east, west }) => break vec![north, south, east, west],
                Some(ServerPacket::TakeDamage { damage, from_id }) => {
                    *hero_life -= damage;
                    println!("[CLIENTE {}] (q) Daño {} de {}", client_id, damage, from_id);
                    if *hero_life <= 0 { send(&mut stream, &ClientPacket::Dead { client_id }); return None; }
                }
                _ => break vec![],
            }
        };

        if neighbors.is_empty() { std::thread::sleep(Duration::from_millis(500)); continue; }

        let life_ratio = *hero_life as f32 / hero_max_life as f32;
        let occupied: Vec<&CellState> = neighbors.iter().filter(|n| n.occupied && n.in_bounds).collect();
        let free:     Vec<&CellState> = neighbors.iter().filter(|n| !n.occupied && n.in_bounds).collect();

        if life_ratio < 0.5 && *potions > 0 {
            *potions -= 1;
            let heal = 50.min(hero_max_life - *hero_life);
            *hero_life += heal;
            println!("[CLIENTE {}] Poción (+{}). HP:{}/{} Pociones:{}", client_id, heal, hero_life, hero_max_life, potions);
            send(&mut stream, &ClientPacket::UsePotion { client_id });

        } else if life_ratio >= 0.5 && !occupied.is_empty() {
            let target = occupied[rand::rng().random_range(0..occupied.len())];
            let damage = calc_damage();
            let (tr, tc) = dir_to_coords(row, col, &target.direction);
            println!("[CLIENTE {}] Atacando ({},{}) por {} dmg vía ORB...", client_id, tr, tc, damage);
            match stub.invoke_attack(client_id, tr, tc, damage) {
                Ok(o) => println!("[CLIENTE {}] ⚔️  ORB: target={:?} dmg={} muerto={} vía={}", client_id, o.target_id.map(|x|x+1), o.damage_applied, o.target_died, o.routing),
                Err(ref e) if e.starts_with("target_not_found") => println!("[CLIENTE {}] Target vacío según ORB", client_id),
                Err(e) => {
                    eprintln!("[CLIENTE {}] ORB error: {} — fallback local", client_id, e);
                    send(&mut stream, &ClientPacket::Attack { client_id, target_row: tr, target_col: tc, damage });
                }
            }

        } else if !free.is_empty() {
            let chosen = free[rand::rng().random_range(0..free.len())];
            let (nr, nc) = dir_to_coords(row, col, &chosen.direction);
            println!("[CLIENTE {}] → {} ({},{})", client_id, chosen.direction, nr, nc);
            send(&mut stream, &ClientPacket::Move { client_id, row: nr, col: nc });
            loop {
                match recv(&mut stream) {
                    Some(ServerPacket::MoveOk { row: r, col: c }) => { row = r; col = c; break; }
                    Some(ServerPacket::MoveDenied { reason }) => { println!("[CLIENTE {}] Movimiento denegado: {}", client_id, reason); break; }
                    Some(ServerPacket::TakeDamage { damage, from_id }) => {
                        *hero_life -= damage;
                        if *hero_life <= 0 { send(&mut stream, &ClientPacket::Dead { client_id }); return None; }
                        println!("[CLIENTE {}] (mov) Daño {} de {}", client_id, damage, from_id);
                    }
                    Some(ServerPacket::Redirect { tcp_addr, row: r, col: c }) => return Some((tcp_addr, r, c)),
                    _ => break,
                }
            }
        } else {
            println!("[CLIENTE {}] Sin acciones, esperando...", client_id);
        }

        std::thread::sleep(Duration::from_millis(800));
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let port: u16 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(7878);
    let mut server_addr = format!("127.0.0.1:{}", port);
    let max_life: i32 = rand::rng().random_range(100..=200);
    let mut life = max_life;
    let mut potions = 3u32;
    let mut entry: Option<(usize, usize)> = None;
    println!("[CLIENTE] HP máx:{} ORB:{}", max_life, ORB_ADDR);
    loop {
        match run_session(&server_addr, entry.map(|(r,_)|r), entry.map(|(_,c)|c), &mut life, max_life, &mut potions) {
            None => { println!("[CLIENTE] Sesión terminada."); return; }
            Some((a, r, c)) => { server_addr = a; entry = Some((r, c)); }
        }
    }
}

fn dir_to_coords(row: usize, col: usize, dir: &str) -> (usize, usize) {
    match dir {
        "N" => (row.saturating_sub(1), col), "S" => ((row+1).min(9), col),
        "E" => (row, (col+1).min(9)),        "W" => (row, col.saturating_sub(1)),
        _   => (row, col),
    }
}
