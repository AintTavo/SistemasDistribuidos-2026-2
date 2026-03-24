// ============================================================
//  COORDINATOR — Puerto 8000 (TCP servidores) | 9000 (WS GUI)
//  Puerto 8600: recibe eventos del ORB Broker
// ============================================================
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tungstenite::accept;

pub const BOARD_SIZE: usize = 10;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum CoordToServer {
    AssignZone { zone: Zone },
    AbsorbZone { zone: Zone, clients: Vec<ClientTransfer> },
    Ping { tick: u64 },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ServerToCoord {
    Register      { server_id: u32, tcp_addr: String, ws_addr: String },
    StateUpdate   { server_id: u32, snapshot: ZoneSnapshot },
    Pong          { server_id: u32, tick: u64 },
    ReadyToAbsorb { server_id: u32 },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Zone {
    pub row_start: usize, pub row_end: usize,
    pub col_start: usize, pub col_end: usize,
}
impl Zone {
    pub fn cells(&self) -> usize {
        (self.row_end - self.row_start + 1) * (self.col_end - self.col_start + 1)
    }
    pub fn contains(&self, r: usize, c: usize) -> bool {
        r >= self.row_start && r <= self.row_end && c >= self.col_start && c <= self.col_end
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientTransfer {
    pub client_id: u32, pub row: usize, pub col: usize,
    pub life: i32, pub max_life: i32, pub potions: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ZoneSnapshot {
    pub zone: Zone, pub clients: Vec<ClientInfo>, pub events: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientInfo {
    pub id: u32, pub row: usize, pub col: usize,
    pub life: i32, pub max_life: i32, pub potions: u32, pub action: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrbEvent {
    pub tick: u64, pub attacker: u32, pub target: u32, pub damage: i32,
    pub target_row: usize, pub target_col: usize, pub died: bool, pub routing: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrbStats {
    pub total_invocations: u64, pub cross_zone_hits: u64,
    pub local_hits: u64, pub misses: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerMeta {
    pub id: u32, pub zone: Zone, pub alive: bool,
    pub client_count: usize, pub ws_addr: String, pub tcp_addr: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GlobalSnapshot {
    pub board: Vec<Vec<Option<ClientInfo>>>,
    pub servers: Vec<ServerMeta>,
    pub events: Vec<String>,
    pub orb_events: Vec<OrbEvent>,
    pub orb_stats: OrbStats,
    pub tick: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum OrbToCoord {
    AttackEvent {
        attacker: u32, target: u32, damage: i32,
        target_row: usize, target_col: usize, died: bool, routing: String,
    },
}

struct ServerEntry {
    id: u32, zone: Zone, tcp_addr: String, ws_addr: String,
    stream_tx: std::sync::mpsc::Sender<CoordToServer>,
    last_pong: Instant, client_count: usize,
}

struct CoordState {
    servers: HashMap<u32, ServerEntry>,
    board: Vec<Vec<Option<ClientInfo>>>,
    events: Vec<String>,
    orb_events: Vec<OrbEvent>,
    orb_stats: OrbStats,
    tick: u64, next_srv: u32,
}

impl CoordState {
    fn new() -> Self {
        CoordState {
            servers: HashMap::new(),
            board: vec![vec![None; BOARD_SIZE]; BOARD_SIZE],
            events: Vec::new(),
            orb_events: Vec::new(),
            orb_stats: OrbStats { total_invocations: 0, cross_zone_hits: 0, local_hits: 0, misses: 0 },
            tick: 0, next_srv: 0,
        }
    }
    fn log(&mut self, msg: String) {
        println!("[COORD] {}", msg);
        self.events.push(msg);
        if self.events.len() > 100 { self.events.remove(0); }
        self.tick += 1;
    }
    fn compute_zones(n: usize) -> Vec<Zone> {
        if n == 0 { return vec![]; }
        let rows_per = BOARD_SIZE / n;
        let extra = BOARD_SIZE % n;
        let mut zones = Vec::new();
        let mut start = 0;
        for i in 0..n {
            let rows = rows_per + if i < extra { 1 } else { 0 };
            zones.push(Zone { row_start: start, row_end: start + rows - 1, col_start: 0, col_end: BOARD_SIZE - 1 });
            start += rows;
        }
        zones
    }
    fn rebalance(&mut self) {
        let ids: Vec<u32> = self.servers.keys().cloned().collect();
        if ids.is_empty() { return; }
        let zones = Self::compute_zones(ids.len());
        for (i, &srv_id) in ids.iter().enumerate() {
            let zone = zones[i].clone();
            if let Some(entry) = self.servers.get_mut(&srv_id) {
                entry.zone = zone.clone();
                let _ = entry.stream_tx.send(CoordToServer::AssignZone { zone });
            }
        }
        self.log(format!("🔄 Rebalanceo: {} servidores", ids.len()));
    }
    fn snapshot(&self) -> GlobalSnapshot {
        let servers: Vec<ServerMeta> = self.servers.values().map(|s| ServerMeta {
            id: s.id, zone: s.zone.clone(), alive: true, client_count: s.client_count,
            ws_addr: s.ws_addr.clone(), tcp_addr: s.tcp_addr.clone(),
        }).collect();
        GlobalSnapshot {
            board: self.board.clone(), servers, events: self.events.clone(),
            orb_events: self.orb_events.clone(), orb_stats: self.orb_stats.clone(),
            tick: self.tick,
        }
    }
}

fn handle_server(mut stream: TcpStream, state: Arc<Mutex<CoordState>>, rx: std::sync::mpsc::Receiver<CoordToServer>) {
    stream.set_read_timeout(Some(Duration::from_millis(50))).unwrap();
    let mut buf = [0u8; 8192];
    let server_id = loop {
        match stream.read(&mut buf) {
            Ok(n) if n > 0 => {
                if let Ok(ServerToCoord::Register { server_id, tcp_addr, ws_addr }) = serde_json::from_slice(&buf[..n]) {
                    let mut s = state.lock().unwrap();
                    s.log(format!("🟢 Servidor {} → TCP:{} WS:{}", server_id+1, tcp_addr, ws_addr));
                    if let Some(entry) = s.servers.get_mut(&server_id) {
                        entry.tcp_addr = tcp_addr; entry.ws_addr = ws_addr;
                    }
                    s.rebalance();
                    break server_id;
                }
            }
            _ => {}
        }
        while let Ok(msg) = rx.try_recv() {
            if let Ok(data) = serde_json::to_vec(&msg) { let _ = stream.write_all(&data); }
        }
    };

    let mut ping_tick = 0u64;
    let mut last_ping = Instant::now();
    loop {
        while let Ok(msg) = rx.try_recv() {
            if let Ok(data) = serde_json::to_vec(&msg) {
                if stream.write_all(&data).is_err() { break; }
            }
        }
        if last_ping.elapsed() > Duration::from_secs(2) {
            ping_tick += 1;
            if let Ok(data) = serde_json::to_vec(&CoordToServer::Ping { tick: ping_tick }) {
                if stream.write_all(&data).is_err() { handle_disconnect(server_id, Arc::clone(&state)); return; }
            }
            last_ping = Instant::now();
            let timed_out = {
                let s = state.lock().unwrap();
                s.servers.get(&server_id).map(|e| e.last_pong.elapsed() > Duration::from_secs(6)).unwrap_or(true)
            };
            if timed_out { println!("[COORD] ⏰ Servidor {} timeout", server_id+1); handle_disconnect(server_id, Arc::clone(&state)); return; }
        }
        match stream.read(&mut buf) {
            Ok(0) => { handle_disconnect(server_id, Arc::clone(&state)); return; }
            Ok(n) => {
                let pkt: ServerToCoord = match serde_json::from_slice(&buf[..n]) { Ok(p) => p, Err(_) => continue };
                match pkt {
                    ServerToCoord::Pong { .. } => {
                        if let Some(e) = state.lock().unwrap().servers.get_mut(&server_id) { e.last_pong = Instant::now(); }
                    }
                    ServerToCoord::StateUpdate { snapshot, .. } => {
                        let mut s = state.lock().unwrap();
                        for ci in &snapshot.clients {
                            if ci.row < BOARD_SIZE && ci.col < BOARD_SIZE { s.board[ci.row][ci.col] = Some(ci.clone()); }
                        }
                        let zone = snapshot.zone.clone();
                        for r in zone.row_start..=zone.row_end {
                            for c in zone.col_start..=zone.col_end {
                                if !snapshot.clients.iter().any(|ci| ci.row == r && ci.col == c) { s.board[r][c] = None; }
                            }
                        }
                        if let Some(e) = s.servers.get_mut(&server_id) { e.client_count = snapshot.clients.len(); }
                        for ev in snapshot.events { s.events.push(ev); if s.events.len() > 100 { s.events.remove(0); } }
                        s.tick += 1;
                    }
                    _ => {}
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(_) => { handle_disconnect(server_id, Arc::clone(&state)); return; }
        }
    }
}

fn handle_disconnect(server_id: u32, state: Arc<Mutex<CoordState>>) {
    let mut s = state.lock().unwrap();
    s.log(format!("🔴 Servidor {} desconectado", server_id+1));
    let dead_zone = s.servers.remove(&server_id).map(|e| e.zone);
    if s.servers.is_empty() { s.log("⚠️  Sin servidores".into()); return; }
    if let Some(zone) = dead_zone {
        let target_id = s.servers.iter().min_by_key(|(_, e)| e.zone.cells()).map(|(&id, _)| id);
        if let Some(tid) = target_id {
            let mut clients = Vec::new();
            for r in zone.row_start..=zone.row_end {
                for c in zone.col_start..=zone.col_end {
                    if let Some(ci) = s.board[r][c].as_ref() {
                        clients.push(ClientTransfer { client_id: ci.id, row: r, col: c, life: ci.life, max_life: ci.max_life, potions: ci.potions });
                    }
                }
            }
            let msg = CoordToServer::AbsorbZone { zone: zone.clone(), clients };
            if let Some(entry) = s.servers.get_mut(&tid) {
                let ez = &entry.zone;
                entry.zone = Zone { row_start: ez.row_start.min(zone.row_start), row_end: ez.row_end.max(zone.row_end), col_start: 0, col_end: BOARD_SIZE - 1 };
                let _ = entry.stream_tx.send(msg);
            }
            s.log(format!("↔️  Zona {} → Servidor {}", server_id+1, tid+1));
        }
    }
}

fn run_ws_bridge(state: Arc<Mutex<CoordState>>) {
    let listener = TcpListener::bind("127.0.0.1:9000").expect("No se pudo bindear :9000");
    println!("[COORD-WS] ws://127.0.0.1:9000");
    for stream in listener.incoming().flatten() {
        let state = Arc::clone(&state);
        thread::spawn(move || {
            let mut ws = match accept(stream) { Ok(w) => w, Err(_) => return };
            loop {
                let snap = { let s = state.lock().unwrap(); serde_json::to_string(&s.snapshot()).unwrap() };
                if ws.send(tungstenite::Message::Text(snap.into())).is_err() { break; }
                thread::sleep(Duration::from_millis(300));
            }
        });
    }
}

fn run_orb_listener(state: Arc<Mutex<CoordState>>) {
    let listener = match TcpListener::bind("127.0.0.1:8600") {
        Ok(l) => l,
        Err(e) => { eprintln!("[COORD-ORB] Error :8600 — {}", e); return; }
    };
    println!("[COORD-ORB] Eventos ORB en 127.0.0.1:8600");
    for stream in listener.incoming().flatten() {
        let state = Arc::clone(&state);
        thread::spawn(move || {
            let mut s2 = stream;
            s2.set_read_timeout(Some(Duration::from_secs(60))).ok();
            let mut buf = [0u8; 2048];
            loop {
                match s2.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        if let Ok(OrbToCoord::AttackEvent { attacker, target, damage, target_row, target_col, died, routing }) = serde_json::from_slice(&buf[..n]) {
                            let mut s = state.lock().unwrap();
                            let tick = s.tick;
                            let cross = routing == "cross_zone";
                            s.orb_stats.total_invocations += 1;
                            if cross { s.orb_stats.cross_zone_hits += 1; } else { s.orb_stats.local_hits += 1; }
                            s.orb_events.push(OrbEvent { tick, attacker, target, damage, target_row, target_col, died, routing: routing.clone() });
                            if s.orb_events.len() > 30 { s.orb_events.remove(0); }
                            s.log(format!("🗡️  [ORB] #{} → #{} dmg={}{}{}", attacker+1, target+1, damage, if died { " 💀" } else { "" }, if cross { " (cross)" } else { " (local)" }));
                        }
                    }
                }
            }
        });
    }
}

fn main() {
    let state: Arc<Mutex<CoordState>> = Arc::new(Mutex::new(CoordState::new()));
    thread::spawn({ let s = Arc::clone(&state); move || run_ws_bridge(s) });
    thread::spawn({ let s = Arc::clone(&state); move || run_orb_listener(s) });
    let listener = TcpListener::bind("127.0.0.1:8000").expect("No se pudo bindear :8000");
    println!("[COORD] 127.0.0.1:8000 | GUI ws://127.0.0.1:9000 | ORB-events 127.0.0.1:8600");
    for incoming in listener.incoming() {
        if let Ok(stream) = incoming {
            let (tx, rx) = std::sync::mpsc::channel::<CoordToServer>();
            {
                let mut s = state.lock().unwrap();
                let id = s.next_srv; s.next_srv += 1;
                s.servers.insert(id, ServerEntry { id, zone: Zone { row_start:0, row_end:0, col_start:0, col_end:0 }, tcp_addr: String::new(), ws_addr: String::new(), stream_tx: tx, last_pong: Instant::now(), client_count: 0 });
            }
            let sc = Arc::clone(&state);
            thread::spawn(move || handle_server(stream, sc, rx));
        }
    }
}
