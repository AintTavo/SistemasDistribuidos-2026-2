// ============================================================
//  SERVIDOR — Mazmorra Multijugador
//  • 1 hilo por cliente TCP (puerto 7878)
//  • Tablero 3x3 con Arc<Mutex<ServerState>>
//  • WS bridge integrado en el mismo proceso (puerto 9001)
//    → Lee el mismo Arc, CERO mensajes extra al servidor
// ============================================================
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tungstenite::accept;

// ─── Paquetes cliente → servidor ─────────────────────────
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ClientPacket {
    QueryNeighbors { client_id: u32 },
    Move           { client_id: u32, row: usize, col: usize },
    Attack         { client_id: u32, target_row: usize, target_col: usize, damage: i32 },
    UsePotion      { client_id: u32 },
    Dead           { client_id: u32 },
}

// ─── Paquetes servidor → cliente ─────────────────────────
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
    pub direction: String,
    pub in_bounds: bool,
    pub occupied:  bool,
    pub client_id: Option<u32>,
}

// ─── Info de cliente visible al frontend ─────────────────
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientInfo {
    pub id:       u32,
    pub life:     i32,
    pub max_life: i32,
    pub potions:  u32,
    pub action:   String,
}

// ─── Snapshot que se envía al HTML via WS ────────────────
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BoardSnapshot {
    pub board:  Vec<Vec<Option<ClientInfo>>>,
    pub events: Vec<String>,
    pub tick:   u64,
}

// ─── Estado global ────────────────────────────────────────
struct Cell { client_id: Option<u32> }

pub struct ServerState {
    board:          Vec<Vec<Cell>>,
    positions:      HashMap<u32, (usize, usize)>,
    client_info:    HashMap<u32, ClientInfo>,
    damage_senders: HashMap<u32, std::sync::mpsc::Sender<ServerPacket>>,
    events:         Vec<String>,
    tick:           u64,
    next_id:        u32,
}

impl ServerState {
    fn new(size: usize) -> Self {
        ServerState {
            board:          (0..size).map(|_| (0..size).map(|_| Cell { client_id: None }).collect()).collect(),
            positions:      HashMap::new(),
            client_info:    HashMap::new(),
            damage_senders: HashMap::new(),
            events:         Vec::new(),
            tick:           0,
            next_id:        0,
        }
    }

    fn find_free_cell(&self) -> Option<(usize, usize)> {
        let candidates: Vec<_> = (0..self.board.len())
            .flat_map(|r| (0..self.board.len())
                .filter(move |&c| self.board[r][c].client_id.is_none())
                .map(move |c| (r, c)))
            .collect();
        if candidates.is_empty() { return None; }
        let idx = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap()
            .subsec_nanos() as usize % candidates.len();
        Some(candidates[idx])
    }

    fn place_client(&mut self, id: u32, row: usize, col: usize) {
        self.board[row][col].client_id = Some(id);
        self.positions.insert(id, (row, col));
    }

    fn remove_client(&mut self, id: u32) {
        if let Some((r, c)) = self.positions.remove(&id) {
            self.board[r][c].client_id = None;
        }
        self.client_info.remove(&id);
        self.damage_senders.remove(&id);
        self.log(format!("💀 Cliente {} eliminado", id + 1));
    }

    fn neighbors(&self, row: usize, col: usize) -> [CellState; 4] {
        let size = self.board.len() as isize;
        [("N",-1isize,0isize),("S",1,0),("E",0,1),("W",0,-1)].map(|(dir, dr, dc)| {
            let nr = row as isize + dr;
            let nc = col as isize + dc;
            if nr < 0 || nr >= size || nc < 0 || nc >= size {
                CellState { direction: dir.into(), in_bounds: false, occupied: true, client_id: None }
            } else {
                let cell = &self.board[nr as usize][nc as usize];
                CellState { direction: dir.into(), in_bounds: true,
                    occupied: cell.client_id.is_some(), client_id: cell.client_id }
            }
        })
    }

    fn log(&mut self, msg: String) {
        println!("[SERVER] {}", msg);
        self.events.push(msg);
        if self.events.len() > 60 { self.events.remove(0); }
    }

    fn snapshot(&self) -> BoardSnapshot {
        let size = self.board.len();
        let board = (0..size).map(|r|
            (0..size).map(|c|
                self.board[r][c].client_id.and_then(|id| self.client_info.get(&id).cloned())
            ).collect()
        ).collect();
        BoardSnapshot { board, events: self.events.clone(), tick: self.tick }
    }
}

// ─── Hilo por cliente ─────────────────────────────────────
fn handle_client(
    mut stream: TcpStream,
    state: Arc<Mutex<ServerState>>,
    client_id: u32,
    damage_rx: std::sync::mpsc::Receiver<ServerPacket>,
) {
    println!("[SERVER] Cliente {} conectado", client_id);
    stream.set_read_timeout(Some(Duration::from_millis(50))).unwrap();

    // Asignar posición y vida inicial
    let welcome = {
        let mut s = state.lock().unwrap();
        match s.find_free_cell() {
            Some((r, c)) => {
                let max_life = 100 + (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH).unwrap()
                    .subsec_millis() % 101) as i32;  // 100-200
                s.place_client(client_id, r, c);
                s.client_info.insert(client_id, ClientInfo {
                    id: client_id, life: max_life, max_life,
                    potions: 3, action: "idle".into(),
                });
                s.log(format!("⚡ Cliente {} entró en ({},{})", client_id + 1, r, c));
                s.tick += 1;
                ServerPacket::Welcome { client_id, row: r, col: c }
            }
            None => ServerPacket::Error { msg: "Tablero lleno".into() },
        }
    };
    send_packet(&mut stream, &welcome);

    let mut buf = [0u8; 1024];
    loop {
        // ── Daño pendiente (non-blocking) ─────────────────
        while let Ok(dmg) = damage_rx.try_recv() {
            if let ServerPacket::TakeDamage { damage, from_id } = &dmg {
                let mut s = state.lock().unwrap();
                let dead = if let Some(info) = s.client_info.get_mut(&client_id) {
                    info.life -= damage;
                    let dead = info.life <= 0;
                    let (life, max) = (info.life, info.max_life);
                    s.log(format!("💢 Cliente {} recibió {} daño de {} → {}/{} HP",
                        client_id+1, damage, from_id+1, life, max));
                    s.tick += 1;
                    dead
                } else { false };

                if dead {
                    s.log(format!("💀 Cliente {} ha muerto!", client_id+1));
                    s.remove_client(client_id);
                    drop(s);
                    send_packet(&mut stream, &ServerPacket::YouDied);
                    return;
                }
            }
            send_packet(&mut stream, &dmg);
        }

        // ── Leer paquete del cliente ──────────────────────
        match stream.read(&mut buf) {
            Ok(0) => {
                println!("[SERVER] Cliente {} desconectado", client_id);
                state.lock().unwrap().remove_client(client_id);
                return;
            }
            Ok(n) => {
                let pkt: ClientPacket = match serde_json::from_slice(&buf[..n]) {
                    Ok(p) => p,
                    Err(e) => { eprintln!("[SERVER] Parse error cliente {}: {}", client_id, e); continue; }
                };

                match pkt {
                    ClientPacket::QueryNeighbors { .. } => {
                        let s = state.lock().unwrap();
                        let (row, col) = *s.positions.get(&client_id).unwrap_or(&(0, 0));
                        let [n, so, e, w] = s.neighbors(row, col);
                        drop(s);
                        send_packet(&mut stream, &ServerPacket::NeighborState {
                            north: n, south: so, east: e, west: w,
                        });
                    }

                    ClientPacket::Move { row: nr, col: nc, .. } => {
                        let mut s = state.lock().unwrap();
                        let size = s.board.len();
                        if nr >= size || nc >= size || s.board[nr][nc].client_id.is_some() {
                            drop(s);
                            send_packet(&mut stream, &ServerPacket::MoveDenied { reason: "Casilla no disponible".into() });
                            continue;
                        }
                        if let Some((or, oc)) = s.positions.get(&client_id).copied() {
                            s.board[or][oc].client_id = None;
                        }
                        s.place_client(client_id, nr, nc);
                        if let Some(info) = s.client_info.get_mut(&client_id) { info.action = "move".into(); }
                        s.log(format!("🚶 Cliente {} → ({},{})", client_id+1, nr, nc));
                        s.tick += 1;
                        drop(s);
                        send_packet(&mut stream, &ServerPacket::MoveOk { row: nr, col: nc });
                    }

                    ClientPacket::Attack { target_row, target_col, damage, .. } => {
                        let mut s = state.lock().unwrap();
                        let target_id = s.board[target_row][target_col].client_id;
                        if let Some(info) = s.client_info.get_mut(&client_id) { info.action = "attack".into(); }
                        if let Some(tid) = target_id {
                            if let Some(sender) = s.damage_senders.get(&tid) {
                                let _ = sender.send(ServerPacket::TakeDamage { damage, from_id: client_id });
                            }
                            s.log(format!("⚔️  Cliente {} atacó a {} por {} daño", client_id+1, tid+1, damage));
                        }
                        s.tick += 1;
                    }

                    ClientPacket::UsePotion { .. } => {
                        let mut s = state.lock().unwrap();
                        if let Some(info) = s.client_info.get_mut(&client_id) {
                            info.potions  = info.potions.saturating_sub(1);
                            info.life     = (info.life + 50).min(info.max_life);
                            info.action   = "potion".into();
                        }
                        s.log(format!("🧪 Cliente {} usó poción", client_id+1));
                        s.tick += 1;
                    }

                    ClientPacket::Dead { .. } => {
                        state.lock().unwrap().remove_client(client_id);
                        return;
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock
                       || e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(e) => {
                eprintln!("[SERVER] I/O error cliente {}: {}", client_id, e);
                state.lock().unwrap().remove_client(client_id);
                return;
            }
        }
    }
}

fn send_packet(stream: &mut TcpStream, pkt: &ServerPacket) {
    if let Ok(data) = serde_json::to_vec(pkt) { let _ = stream.write_all(&data); }
}

// ─── WS Bridge integrado ──────────────────────────────────
// Lee el Arc<Mutex<ServerState>> directamente — sin mensajes TCP extra
fn run_ws_bridge(state: Arc<Mutex<ServerState>>) {
    let listener = TcpListener::bind("127.0.0.1:9001")
        .expect("[BRIDGE] No se pudo bindear :9001");
    println!("[BRIDGE] WebSocket listo en ws://127.0.0.1:9001");

    for stream in listener.incoming().flatten() {
        let state = Arc::clone(&state);
        thread::spawn(move || {
            let mut ws = match accept(stream) { Ok(w) => w, Err(_) => return };
            println!("[BRIDGE] Frontend conectado");
            loop {
                let snap = { serde_json::to_string(&state.lock().unwrap().snapshot()).unwrap() };
                if ws.send(tungstenite::Message::Text(snap)).is_err() { break; }
                thread::sleep(Duration::from_millis(400));
            }
        });
    }
}

// ─── Main ─────────────────────────────────────────────────
fn main() {
    let state: Arc<Mutex<ServerState>> = Arc::new(Mutex::new(ServerState::new(3)));

    // WS bridge en hilo separado, mismo Arc
    thread::spawn({ let s = Arc::clone(&state); move || run_ws_bridge(s) });

    let listener = TcpListener::bind("127.0.0.1:7878")
        .expect("[SERVER] No se pudo bindear :7878");
    println!("[SERVER] Mazmorra en 127.0.0.1:7878  |  WS bridge en :9001");
    println!("[SERVER] Abre index.html en el navegador");

    for incoming in listener.incoming() {
        if let Ok(stream) = incoming {
            let (tx, rx) = std::sync::mpsc::channel::<ServerPacket>();
            let client_id = {
                let mut s = state.lock().unwrap();
                let id = s.next_id;
                s.next_id += 1;
                s.damage_senders.insert(id, tx);
                id
            };
            let sc = Arc::clone(&state);
            thread::spawn(move || handle_client(stream, sc, client_id, rx));
        }
    }
}