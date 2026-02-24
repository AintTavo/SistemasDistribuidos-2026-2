use std::net::TcpStream;
use std::io::{Read, Write};
use std::time::Duration;
use rand::Rng;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct GamePacket {
    life: i32,
    damage: i32,
    message: String,
    is_dead: bool,
}

fn main() {
    // 1. Conectar al servidor del Monstruo
    let mut stream = TcpStream::connect("127.0.0.1:7878").expect("¡No se pudo encontrar al monstruo!");
    println!(" ¡Has entrado en la mazmorra! El combate comienza.");

    let mut dice = rand::thread_rng();
    let max_life = dice.gen_range(100..=200);
    let mut life = max_life;
    let mut potions = 3;

    loop {
        // --- TURNO DEL HÉROE ---
        let mut damage_to_send = 0;
        let mut action_message = String::new();

        // Lógica de decisión simple
        if life < (max_life / 3) && potions > 0 {
            potions -= 1;
            life = (life + 50).min(max_life);
            action_message = format!("El héroe usó una poción. HP actual: {}", life);
            println!("Pociones : {}", action_message);
        } else {
            let d20 = dice.gen_range(1..=20);
            let d8 = dice.gen_range(1..=8) + dice.gen_range(1..=8);
            damage_to_send = (d8 as f32 * d20_action_validation(d20)) as i32;
            action_message = format!("¡El héroe ataca con fuerza!");
            println!(" {} Daño: {}", action_message, damage_to_send);
        }

        // Enviar paquete al Monstruo
        let packet = GamePacket {
            life,
            damage: damage_to_send,
            message: action_message,
            is_dead: false,
        };

        let serialized = serde_json::to_vec(&packet).unwrap();
        stream.write_all(&serialized).unwrap();

        // --- ESPERAR RESPUESTA DEL MONSTRUO ---
        let mut buffer = [0; 512];
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("La conexión se cerró.");
                break;
            }
            Ok(bytes_read) => {
                let incoming: GamePacket = serde_json::from_slice(&buffer[..bytes_read]).unwrap();

                if incoming.is_dead {
                    println!(" ¡Increíble! El monstruo ha caído. Victoria para el héroe.");
                    break;
                }

                life -= incoming.damage;
                println!(" Monstruo ataca: {} (Recibiste {} de daño)", incoming.message, incoming.damage);
                println!(" Vida restante del Héroe: {}/{}", life, max_life);

                if life <= 0 {
                    println!(" El héroe ha sucumbido ante la bestia...");
                    let death_packet = GamePacket { life: 0, damage: 0, message: "Muerto".into(), is_dead: true };
                    let _ = stream.write_all(&serde_json::to_vec(&death_packet).unwrap());
                    break;
                }
            }
            Err(e) => {
                println!("Error leyendo del servidor: {}", e);
                break;
            }
        }

        std::thread::sleep(Duration::from_secs(1));
    }
}

fn d20_action_validation(d20: i32) -> f32 {
    match d20 {
        1 => 0.0,
        2..=6 => 0.2,
        7..=9 => 0.75,
        10..=13 => 1.0,
        14..=16 => 1.5,
        17..=19 => 2.0,
        20 => 3.0,
        _ => -1.0,
    }
}