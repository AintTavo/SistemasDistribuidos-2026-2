// ============================================================
//  SERVIDOR DE ZONA — Mazmorra Distribuida 10×10
//  
//  Cada instancia:
//  • Se conecta al Coordinador (puerto 8000)
//  • Recibe su zona dinámica del tablero 10×10
//  • Acepta clientes TCP (puerto configurable, por defecto 7878)
//  • Expone WS local para debug (puerto configurable)
//  • Maneja absorción de zonas vecinas muertas
//  • Mecanismos: Arc<Mutex>, mpsc channels, timeouts
//
//  Uso:  cargo run --bin server -- <tcp_port> <ws_port>
//        cargo run --bin server -- 7878 9001
//        cargo run --bin server -- 7879 9002
// ============================================================
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tungstenite::accept;

const BOARD_SIZE: usize = 10;

// ─── Paquetes compartidos con el coordinador ─────────────
// (replicados para evitar dependencia entre binarios)
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum CoordToServer {
    AssignZone  { zone: Zone },
    AbsorbZone  { zone: Zone, clients: Vec<ClientTransfer> },
    Ping        { tick: u64 },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ServerToCoord {
    Register       { server_id: u32, tcp_addr: String, ws_addr: String },
    StateUpdate    { server_id: u32, snapshot: ZoneSnapshot },
    Pong           { server_id: u32, tick: u64 },
    ReadyToAbsorb  { server_id: u32 },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Zone {
    pub row_start: usize,
    pub row_end:   usize,
    pub col_start: usize,
    pub col_end:   usize,
}
impl Zone {
    pub fn contains(&self, r: usize, c: usize) -> bool {
        r >= self.row_start && r <= self.row_end &&
        c >= self.col_start && c <= self.col_end
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientTransfer {
    pub client_id: u32,
    pub row: usize, pub col: usize,
    pub life: i32,  pub max_life: i32,
    pub potions: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ZoneSnapshot {
    pub zone:    Zone,
    pub clients: Vec<ClientInfo>,
    pub events:  Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientInfo {
    pub id:       u32,
    pub row:      usize,
    pub col:      usize,
    pub life:     i32,
    pub max_life: i32,
    pub potions:  u32,
    pub action:   String,
}

// ─── Paquetes cliente ↔ servidor (sin cambios de API) ─────
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
    /// Redirige al cliente a otro servidor cuando sale de la zona
    Redirect      { tcp_addr: String, row: usize, col: usize },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CellState {
    pub direction: String,
    pub in_bounds: bool,
    pub occupied:  bool,
    pub client_id: Option<u32>,
}

// ─── Estado de celda ──────────────────────────────────────
struct Cell { client_id: Option<u32> }

// ─── Estado global del servidor ──────────────────────────
pub struct ServerState {
    server_id:      u32,
    zone:           Zone,
    board:          Vec<Vec<Cell>>,
    positions:      HashMap<u32, (usize, usize)>,
    client_info:    HashMap<u32, ClientInfo>,
    damage_senders: HashMap<u32, std::sync::mpsc::Sender<ServerPacket>>,
    events:         Vec<String>,
    tick:           u64,
    next_id:        u32,
}

impl ServerState {
    fn new(server_id: u32) -> Self {
        ServerState {
            server_id,
            zone: Zone { row_start: 0, row_end: 0, col_start: 0, col_end: 9 },
            board: (0..BOARD_SIZE).map(|_| (0..BOARD_SIZE)
                .map(|_| Cell { client_id: None }).collect()).collect(),
            positions:      HashMap::new(),
            client_info:    HashMap::new(),
            damage_senders: HashMap::new(),
            events:         Vec::new(),
            tick:           0,
            next_id:        0,
        }
    }

    /// Acepta zona nueva del coordinador (rebalanceo)
    fn assign_zone(&mut self, zone: Zone) {
        self.log(format!("📦 Zona asignada: ({},{})→({},{})",
            zone.row_start, zone.col_start, zone.row_end, zone.col_end));
        self.zone = zone;
    }

    /// Absorbe una zona muerta de otro servidor
    fn absorb_zone(&mut self, zone: Zone, clients: Vec<ClientTransfer>) {
        self.log(format!("⬆️  Absorbiendo zona ({},{})→({},{}) con {} clientes",
            zone.row_start, zone.col_start, zone.row_end, zone.col_end, clients.len()));
        // Expandir zona propia
        self.zone = Zone {
            row_start: self.zone.row_start.min(zone.row_start),
            row_end:   self.zone.row_end.max(zone.row_end),
            col_start: 0,
            col_end:   BOARD_SIZE - 1,
        };
        // Colocar clientes transferidos
        for ct in clients {
            self.board[ct.row][ct.col].client_id = Some(ct.client_id);
            self.positions.insert(ct.client_id, (ct.row, ct.col));
            self.client_info.insert(ct.client_id, ClientInfo {
                id:       ct.client_id,
                row:      ct.row,
                col:      ct.col,
                life:     ct.life,
                max_life: ct.max_life,
                potions:  ct.potions,
                action:   "idle".into(),
            });
        }
    }

    fn find_free_cell_in_zone(&self) -> Option<(usize, usize)> {
        let candidates: Vec<_> = (self.zone.row_start..=self.zone.row_end)
            .flat_map(|r| (self.zone.col_start..=self.zone.col_end)
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
        if let Some(ci) = self.client_info.get_mut(&id) {
            ci.row = row; ci.col = col;
        }
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
        let size = BOARD_SIZE as isize;
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
        println!("[SRV-{}] {}", self.server_id + 1, msg);
        self.events.push(msg);
        if self.events.len() > 60 { self.events.remove(0); }
        self.tick += 1;
    }

    fn snapshot(&self) -> ZoneSnapshot {
        ZoneSnapshot {
            zone:    self.zone.clone(),
            clients: self.client_info.values().cloned().collect(),
            events:  self.events.clone(),
        }
    }
}

// ─── Conexión al coordinador ──────────────────────────────
fn connect_to_coordinator(
    state: Arc<Mutex<ServerState>>,
    tcp_port: u16,
    ws_port: u16,
) {
    thread::spawn(move || {
        loop {
            match TcpStream::connect("127.0.0.1:8000") {
                Ok(mut stream) => {
                    stream.set_read_timeout(Some(Duration::from_millis(100))).unwrap();
                    let server_id = state.lock().unwrap().server_id;

                    // Registrarse
                    let reg = ServerToCoord::Register {
                        server_id,
                        tcp_addr: format!("127.0.0.1:{}", tcp_port),
                        ws_addr:  format!("127.0.0.1:{}", ws_port),
                    };
                    if let Ok(data) = serde_json::to_vec(&reg) {
                        let _ = stream.write_all(&data);
                    }
                    println!("[SRV-{}] Conectado al coordinador", server_id + 1);

                    let mut buf = [0u8; 8192];
                    let mut last_update = Instant::now();

                    loop {
                        // ── Enviar snapshot periódico al coordinador ───
                        if last_update.elapsed() > Duration::from_millis(500) {
                            let snap = {
                                let s = state.lock().unwrap();
                                ServerToCoord::StateUpdate {
                                    server_id: s.server_id,
                                    snapshot:  s.snapshot(),
                                }
                            };
                            if let Ok(data) = serde_json::to_vec(&snap) {
                                if stream.write_all(&data).is_err() { break; }
                            }
                            last_update = Instant::now();
                        }

                        // ── Leer mensajes del coordinador ─────────────
                        match stream.read(&mut buf) {
                            Ok(0) => break,
                            Ok(n) => {
                                let pkt: CoordToServer = match serde_json::from_slice(&buf[..n]) {
                                    Ok(p) => p,
                                    Err(_) => continue,
                                };
                                match pkt {
                                    CoordToServer::AssignZone { zone } => {
                                        state.lock().unwrap().assign_zone(zone);
                                    }
                                    CoordToServer::AbsorbZone { zone, clients } => {
                                        state.lock().unwrap().absorb_zone(zone, clients);
                                        // Confirmar
                                        let sid = state.lock().unwrap().server_id;
                                        let msg = ServerToCoord::ReadyToAbsorb { server_id: sid };
                                        if let Ok(data) = serde_json::to_vec(&msg) {
                                            let _ = stream.write_all(&data);
                                        }
                                    }
                                    CoordToServer::Ping { tick } => {
                                        let sid = state.lock().unwrap().server_id;
                                        let pong = ServerToCoord::Pong { server_id: sid, tick };
                                        if let Ok(data) = serde_json::to_vec(&pong) {
                                            if stream.write_all(&data).is_err() { break; }
                                        }
                                    }
                                }
                            }
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock
                                       || e.kind() == std::io::ErrorKind::TimedOut => {}
                            Err(_) => break,
                        }
                    }
                    println!("[SRV] Desconectado del coordinador, reintentando...");
                }
                Err(_) => {
                    println!("[SRV] Coordinador no disponible, reintentando en 3s...");
                }
            }
            thread::sleep(Duration::from_secs(3));
        }
    });
}

// ─── Hilo por cliente TCP ─────────────────────────────────
fn handle_client(
    mut stream: TcpStream,
    state: Arc<Mutex<ServerState>>,
    client_id: u32,
    damage_rx: std::sync::mpsc::Receiver<ServerPacket>,
) {
    stream.set_read_timeout(Some(Duration::from_millis(50))).unwrap();

    let welcome = {
        let mut s = state.lock().unwrap();
        match s.find_free_cell_in_zone() {
            Some((r, c)) => {
                let max_life = 100 + (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH).unwrap()
                    .subsec_millis() % 101) as i32;
                s.place_client(client_id, r, c);
                s.client_info.insert(client_id, ClientInfo {
                    id: client_id, row: r, col: c,
                    life: max_life, max_life,
                    potions: 3, action: "idle".into(),
                });
                s.log(format!("⚡ Cliente {} entró en ({},{})", client_id + 1, r, c));
                ServerPacket::Welcome { client_id, row: r, col: c }
            }
            None => ServerPacket::Error { msg: "Zona llena".into() },
        }
    };
    send_pkt(&mut stream, &welcome);

    let mut buf = [0u8; 1024];
    loop {
        // ── Daño pendiente ────────────────────────────────
        while let Ok(dmg) = damage_rx.try_recv() {
            if let ServerPacket::TakeDamage { damage, from_id } = &dmg {
                let mut s = state.lock().unwrap();
                let dead = if let Some(info) = s.client_info.get_mut(&client_id) {
                    info.life -= damage;
                    let dead     = info.life <= 0;
                    let life     = info.life;
                    let max_life = info.max_life;
                    let _ = info; // liberar borrow mutable antes de llamar s.log
                    s.log(format!("💢 Cliente {} recibió {} de {} → {}/{} HP",
                        client_id+1, damage, from_id+1, life, max_life));
                    s.tick += 1;
                    dead
                } else { false };

                if dead {
                    s.remove_client(client_id);
                    drop(s);
                    send_pkt(&mut stream, &ServerPacket::YouDied);
                    return;
                }
            }
            send_pkt(&mut stream, &dmg);
        }

        // ── Leer paquete del cliente ──────────────────────
        match stream.read(&mut buf) {
            Ok(0) => {
                state.lock().unwrap().remove_client(client_id);
                return;
            }
            Ok(n) => {
                let pkt: ClientPacket = match serde_json::from_slice(&buf[..n]) {
                    Ok(p) => p,
                    Err(_) => continue,
                };

                match pkt {
                    ClientPacket::QueryNeighbors { .. } => {
                        let s = state.lock().unwrap();
                        let (row, col) = *s.positions.get(&client_id).unwrap_or(&(0, 0));
                        let [n, so, e, w] = s.neighbors(row, col);
                        drop(s);
                        send_pkt(&mut stream, &ServerPacket::NeighborState {
                            north: n, south: so, east: e, west: w,
                        });
                    }

                    ClientPacket::Move { row: nr, col: nc, .. } => {
                        let mut s = state.lock().unwrap();
                        if nr >= BOARD_SIZE || nc >= BOARD_SIZE || s.board[nr][nc].client_id.is_some() {
                            drop(s);
                            send_pkt(&mut stream, &ServerPacket::MoveDenied {
                                reason: "Casilla no disponible".into()
                            });
                            continue;
                        }
                        // Si la celda destino está fuera de la zona actual → denegar
                        // (el cliente eventualmente seguirá intentando; el servidor vecino
                        //  podría estar disponible pero en esta arquitectura los clientes
                        //  se quedan con el servidor que los acoge a menos que se redirijan)
                        if !s.zone.contains(nr, nc) {
                            drop(s);
                            send_pkt(&mut stream, &ServerPacket::MoveDenied {
                                reason: "Fuera de zona — esperando rebalanceo".into()
                            });
                            continue;
                        }
                        if let Some((or, oc)) = s.positions.get(&client_id).copied() {
                            s.board[or][oc].client_id = None;
                        }
                        s.place_client(client_id, nr, nc);
                        if let Some(info) = s.client_info.get_mut(&client_id) {
                            info.action = "move".into();
                        }
                        s.log(format!("🚶 Cliente {} → ({},{})", client_id+1, nr, nc));
                        s.tick += 1;
                        drop(s);
                        send_pkt(&mut stream, &ServerPacket::MoveOk { row: nr, col: nc });
                    }

                    ClientPacket::Attack { target_row, target_col, damage, .. } => {
                        let mut s = state.lock().unwrap();
                        let target_id = if target_row < BOARD_SIZE && target_col < BOARD_SIZE {
                            s.board[target_row][target_col].client_id
                        } else { None };
                        if let Some(info) = s.client_info.get_mut(&client_id) {
                            info.action = "attack".into();
                        }
                        if let Some(tid) = target_id {
                            if let Some(sender) = s.damage_senders.get(&tid) {
                                let _ = sender.send(ServerPacket::TakeDamage { damage, from_id: client_id });
                            }
                            s.log(format!("⚔️  Cliente {} atacó a {} por {} dmg", client_id+1, tid+1, damage));
                        }
                        s.tick += 1;
                    }

                    ClientPacket::UsePotion { .. } => {
                        let mut s = state.lock().unwrap();
                        if let Some(info) = s.client_info.get_mut(&client_id) {
                            info.potions = info.potions.saturating_sub(1);
                            info.life    = (info.life + 50).min(info.max_life);
                            info.action  = "potion".into();
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
                eprintln!("[SRV] I/O error cliente {}: {}", client_id, e);
                state.lock().unwrap().remove_client(client_id);
                return;
            }
        }
    }
}

fn send_pkt(stream: &mut TcpStream, pkt: &ServerPacket) {
    if let Ok(data) = serde_json::to_vec(pkt) { let _ = stream.write_all(&data); }
}

// ─── WS Bridge local (debug por servidor) ────────────────
fn run_local_ws(state: Arc<Mutex<ServerState>>, ws_port: u16) {
    let addr = format!("127.0.0.1:{}", ws_port);
    let listener = TcpListener::bind(&addr)
        .expect(&format!("[SRV-WS] No se pudo bindear :{}", ws_port));
    println!("[SRV-WS] WS local debug en ws://{}", addr);

    for stream in listener.incoming().flatten() {
        let state = Arc::clone(&state);
        thread::spawn(move || {
            let mut ws = match accept(stream) { Ok(w) => w, Err(_) => return };
            loop {
                let snap = {
                    let s = state.lock().unwrap();
                    serde_json::to_string(&s.snapshot()).unwrap()
                };
                if ws.send(tungstenite::Message::Text(snap.into())).is_err() { break; }
                thread::sleep(Duration::from_millis(400));
            }
        });
    }
}

// ─── Main ─────────────────────────────────────────────────
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let tcp_port: u16 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(7878);
    let ws_port:  u16 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(9001);

    // ID provisional (el coordinador asigna el real, pero como nos conectamos
    // antes de recibir AssignZone usamos el puerto como discriminador temporal)
    let provisional_id = tcp_port - 7878;
    let state: Arc<Mutex<ServerState>> = Arc::new(Mutex::new(
        ServerState::new(provisional_id as u32)
    ));

    println!("[SRV-{}] Puerto TCP:{} WS:{}", provisional_id+1, tcp_port, ws_port);

    // Conectar al coordinador en hilo separado
    connect_to_coordinator(Arc::clone(&state), tcp_port, ws_port);

    // WS local en hilo separado
    thread::spawn({
        let s = Arc::clone(&state);
        move || run_local_ws(s, ws_port)
    });

    // Aceptar clientes TCP
    let listener = TcpListener::bind(format!("127.0.0.1:{}", tcp_port))
        .expect(&format!("[SRV] No se pudo bindear :{}", tcp_port));
    println!("[SRV] Aceptando clientes en 127.0.0.1:{}", tcp_port);

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