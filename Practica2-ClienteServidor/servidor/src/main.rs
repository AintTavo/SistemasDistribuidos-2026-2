// Standard Libraries
use std::net::{TcpListener};
use std::io::{Read, Write};
use std::time::Duration;

// External Dependencies
use rand::Rng;
use serde::{Serialize, Deserialize};
use serde_json;

#[derive(Serialize, Deserialize)]
struct GamePacket {
    life : i32,
    damage : i32,
    message : String,
    is_dead : bool
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("El servidor esta escuchado en el puerto 7878");

    if let Ok((mut stream , _)) = listener.accept() {
        let mut dice = rand::thread_rng();
        let max_life = dice.gen_range(400..=500);
        let mut life = max_life;

        loop {
            // 1. Recibir ataque del Héroe
            let mut buffer = [0; 512];
            let bytes_read = stream.read(&mut buffer).unwrap();
            if bytes_read == 0 { break; } // Conexión cerrada

            let incoming: GamePacket = serde_json::from_slice(&buffer[..bytes_read]).unwrap();
            
            if incoming.is_dead {
                println!("Monstruo: ¡Jajaja! El héroe ha caído.");
                break;
            }

            life -= incoming.damage;
            println!("Monstruo [{}/{}]: Recibí {} de daño", life, max_life, incoming.damage);

            if life <= 0 {
                println!("El monstruo ha sido derrotado...");
                let packet = GamePacket { life: 0, damage: 0, message: "Muerte".into(), is_dead: true };
                let j = serde_json::to_vec(&packet).unwrap();
                stream.write_all(&j).unwrap();
                break;
            }

            // 2. Contraataque del Monstruo
            let attack = (dice.gen_range(10..25) as f32 * d20_action_validation(dice.gen_range(1..20))) as i32;
            let packet = GamePacket {
                life,
                damage: attack,
                message: "¡Rugido mortal!".into(),
                is_dead: false,
            };

            let response = serde_json::to_vec(&packet).unwrap();
            stream.write_all(&response).unwrap();
            std::thread::sleep(Duration::from_secs(2));
        }
    }
}



fn d20_action_validation(d20 : i32) -> f32{
    match d20 {
        1 => return 0.0,
        2..=6 => return 0.2,
        7..=9 => return 0.75,
        10..=13 => return 1.0,
        14..=16 => return 1.5,
        17..=19 => return 2.0,
        20 => return 3.0,
        _ => return -1.0,
    }
}