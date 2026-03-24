// ============================================================
//  ORB BROKER — Attack Object Request Broker
//  Puerto 8500: recibe peticiones de Stubs (clientes/servidores)
//  Puerto 8600 → Coordinador: notifica eventos de ataque
// ============================================================
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use serde::{Deserialize, Serialize};

// ─── IOR ─────────────────────────────────────────────────────
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InteropRef {
    pub zone_server_id: u32,
    pub skeleton_addr:  String,
    pub row:            usize,
    pub col:            usize,
    pub occupant_id:    Option<u32>,
}

// ─── Protocolo Stub ↔ ORB ────────────────────────────────────
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum OrbRequest {
    RegisterZone { server_id: u32, skeleton_addr: String, zone: OrbZone },
    UpdateCells  { server_id: u32, cells: Vec<CellReg> },
    InvokeAttack { request_id: u64, attacker_id: u32, target_row: usize, target_col: usize, damage: i32 },
    LookupIOR    { row: usize, col: usize },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum OrbReply {
    AttackResult   { request_id: u64, target_id: Option<u32>, damage_applied: i32, target_died: bool, routing: String },
    TargetNotFound { request_id: u64, reason: String },
    IORFound       { ior: InteropRef },
    IORNotFound    { row: usize, col: usize },
    ZoneRegistered { server_id: u32 },
    CellsUpdated   { server_id: u32, count: usize },
    Error          { request_id: u64, msg: String },
}

// ─── Protocolo ORB → Skeleton ─────────────────────────────────
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum SkelRequest {
    PerformAttack { request_id: u64, attacker_id: u32, target_row: usize, target_col: usize, damage: i32 },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum SkelReply {
    AttackExecuted { request_id: u64, target_id: u32, damage_applied: i32, target_died: bool },
    TargetNotFound { request_id: u64 },
    Error          { request_id: u64, msg: String },
}

// ─── Evento hacia Coordinador ─────────────────────────────────
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum OrbToCoord {
    AttackEvent { attacker: u32, target: u32, damage: i32, target_row: usize, target_col: usize, died: bool, routing: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrbZone {
    pub row_start: usize, pub row_end: usize,
    pub col_start: usize, pub col_end: usize,
}
impl OrbZone {
    pub fn contains(&self, r: usize, c: usize) -> bool {
        r >= self.row_start && r <= self.row_end && c >= self.col_start && c <= self.col_end
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CellReg {
    pub row: usize, pub col: usize, pub occupant_id: Option<u32>,
}

// ─── Object Registry ──────────────────────────────────────────
struct Registry {
    cells:   HashMap<usize, InteropRef>,
    servers: HashMap<u32, (String, OrbZone)>,
}
impl Registry {
    fn new() -> Self { Registry { cells: HashMap::new(), servers: HashMap::new() } }

    fn register_zone(&mut self, server_id: u32, addr: String, zone: OrbZone) {
        println!("[ORB Registry] Servidor {} @ {} zona ({},{})-({},{})",
            server_id, addr, zone.row_start, zone.col_start, zone.row_end, zone.col_end);
        self.servers.insert(server_id, (addr, zone));
    }

    fn update_cells(&mut self, server_id: u32, cells: Vec<CellReg>) {
        let addr = match self.servers.get(&server_id) { Some((a,_)) => a.clone(), None => return };
        let zone = match self.servers.get(&server_id) { Some((_,z)) => z.clone(), None => return };
        for cell in &cells {
            let key = cell.row * 10 + cell.col;
            self.cells.insert(key, InteropRef { zone_server_id: server_id, skeleton_addr: addr.clone(), row: cell.row, col: cell.col, occupant_id: cell.occupant_id });
        }
        self.cells.retain(|_, ior| {
            if ior.zone_server_id != server_id { return true; }
            if !zone.contains(ior.row, ior.col) { return true; }
            ior.occupant_id.is_some()
        });
    }

    fn lookup(&self, row: usize, col: usize) -> Option<&InteropRef> {
        self.cells.get(&(row * 10 + col))
    }
}

// ─── Notificar Coordinador ─────────────────────────────────────
fn notify_coordinator(evt: &OrbToCoord) {
    if let Ok(mut s) = TcpStream::connect("127.0.0.1:8600") {
        s.set_write_timeout(Some(Duration::from_secs(1))).ok();
        if let Ok(data) = serde_json::to_vec(evt) { let _ = s.write_all(&data); }
    }
}

// ─── Despachar ataque al Skeleton ─────────────────────────────
fn dispatch(registry: &Registry, req_id: u64, attacker_id: u32, target_row: usize, target_col: usize, damage: i32) -> OrbReply {
    let ior = match registry.lookup(target_row, target_col) {
        Some(i) if i.occupant_id.is_some() => i.clone(),
        _ => return OrbReply::TargetNotFound { request_id: req_id, reason: format!("Celda ({},{}) vacía", target_row, target_col) },
    };

    println!("[ORB IIOP] req#{} → skeleton {} (srv {})", req_id, ior.skeleton_addr, ior.zone_server_id);

    let mut stream = match TcpStream::connect(&ior.skeleton_addr) {
        Ok(s) => s,
        Err(e) => return OrbReply::Error { request_id: req_id, msg: format!("No se pudo conectar skeleton: {}", e) },
    };
    stream.set_read_timeout(Some(Duration::from_secs(3))).ok();

    let skel_req = SkelRequest::PerformAttack { request_id: req_id, attacker_id, target_row, target_col, damage };
    let data = match serde_json::to_vec(&skel_req) { Ok(d) => d, Err(e) => return OrbReply::Error { request_id: req_id, msg: e.to_string() } };
    if stream.write_all(&data).is_err() { return OrbReply::Error { request_id: req_id, msg: "Write al skeleton falló".into() }; }

    let mut buf = [0u8; 2048];
    let n = match stream.read(&mut buf) {
        Ok(n) if n > 0 => n,
        _ => return OrbReply::Error { request_id: req_id, msg: "Sin respuesta del skeleton".into() },
    };

    match serde_json::from_slice::<SkelReply>(&buf[..n]) {
        Ok(SkelReply::AttackExecuted { request_id, target_id, damage_applied, target_died }) => {
            // Notificar al coordinador
            notify_coordinator(&OrbToCoord::AttackEvent {
                attacker: attacker_id, target: target_id, damage: damage_applied,
                target_row, target_col, died: target_died, routing: "cross_zone".into(),
            });
            OrbReply::AttackResult { request_id, target_id: Some(target_id), damage_applied, target_died, routing: "cross_zone".into() }
        }
        Ok(SkelReply::TargetNotFound { request_id }) => {
            OrbReply::TargetNotFound { request_id, reason: "Skeleton: celda vacía".into() }
        }
        Ok(SkelReply::Error { request_id, msg }) => OrbReply::Error { request_id, msg },
        Err(e) => OrbReply::Error { request_id: req_id, msg: e.to_string() },
    }
}

// ─── Manejador de conexión entrante ───────────────────────────
fn handle_conn(mut stream: TcpStream, registry: Arc<Mutex<Registry>>) {
    stream.set_read_timeout(Some(Duration::from_secs(10))).ok();
    let mut buf = [0u8; 4096];
    loop {
        let n = match stream.read(&mut buf) { Ok(0) | Err(_) => break, Ok(n) => n };
        let req: OrbRequest = match serde_json::from_slice(&buf[..n]) { Ok(r) => r, Err(_) => break };

        let reply = {
            let mut reg = registry.lock().unwrap();
            match req {
                OrbRequest::RegisterZone { server_id, skeleton_addr, zone } => {
                    reg.register_zone(server_id, skeleton_addr, zone);
                    OrbReply::ZoneRegistered { server_id }
                }
                OrbRequest::UpdateCells { server_id, cells } => {
                    let count = cells.len();
                    reg.update_cells(server_id, cells);
                    OrbReply::CellsUpdated { server_id, count }
                }
                OrbRequest::InvokeAttack { request_id, attacker_id, target_row, target_col, damage } => {
                    println!("[ORB] InvokeAttack req#{}: c{} → ({},{}) dmg={}", request_id, attacker_id+1, target_row, target_col, damage);
                    let ior_opt = reg.lookup(target_row, target_col).cloned();
                    drop(reg); // liberar mutex antes del I/O bloqueante
                    match ior_opt {
                        Some(ref ior) if ior.occupant_id.is_some() => {
                            let reg2 = registry.lock().unwrap();
                            dispatch(&reg2, request_id, attacker_id, target_row, target_col, damage)
                        }
                        _ => OrbReply::TargetNotFound { request_id, reason: format!("({},{}) sin ocupante", target_row, target_col) },
                    }
                }
                OrbRequest::LookupIOR { row, col } => {
                    match reg.lookup(row, col).cloned() {
                        Some(ior) => OrbReply::IORFound { ior },
                        None      => OrbReply::IORNotFound { row, col },
                    }
                }
            }
        };

        if let Ok(data) = serde_json::to_vec(&reply) {
            if stream.write_all(&data).is_err() { break; }
        }
    }
}

fn main() {
    let registry: Arc<Mutex<Registry>> = Arc::new(Mutex::new(Registry::new()));
    let listener = TcpListener::bind("127.0.0.1:8500").expect("No se pudo bindear :8500");
    println!("╔══════════════════════════════════════════╗");
    println!("║  ORB Broker  127.0.0.1:8500             ║");
    println!("║  Eventos  →  Coordinador :8600          ║");
    println!("╚══════════════════════════════════════════╝");
    for incoming in listener.incoming() {
        if let Ok(stream) = incoming {
            let reg = Arc::clone(&registry);
            thread::spawn(move || handle_conn(stream, reg));
        }
    }
}
