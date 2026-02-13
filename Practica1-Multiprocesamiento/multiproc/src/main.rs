// Standard Libraries
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

// External Dependencies
use rand::Rng;


fn main() {
    // ----------- Comunicación entre hilos ------------
    // let (tx, rx) = mpsc::channel();
    // ------------ Monster -----------------------
    let monster_handle = thread::spawn( move || {
        // Handlers propio del hilo


        let mut dice = rand::thread_rng();
        let max_life = dice.gen_range(400..=500);
        let mut life = max_life

        let _defense = 10;

        loop{
            // Checa si la vida llego a 0
            if life <= 0 {
                break;
            };

            let action = dice.gen_range(1..=3);
            let action_factor = d20_action_validation_monster(dice.gen_range(1..=20 as i32));

            // La accion que realizara este turno el jefe
            match action{
                1 => { // Healing
                    let healing = (50.0 * action_factor) as i32;
                    life = life + healing;
                    if (life + healing) >= max_life {
                        life = max_life;
                        println("El monstruo ya curó todas sus heridas");
                    }
                    else{
                        life = life + healing;
                        println!("El monstruo se curo {}, pasa a tener {} de vida", healing, life);
                    }
                    
                },
                2 => { // Attack
                    let attack : i32 = (((dice.gen_range(1..=6) as f32) + (dice.gen_range(1..=6) as f32)) * action_factor) as i32;
                    let objective = dice.gen_range(1..=2);
                    println!("El monstruo ataco a Thread{} con {}", objective, attack);
                },
                3 => println!("El monstruo decidio descansar este turno"),
                _ => println!("Error generating an action!!"),
            }

            thread::sleep(Duration::from_secs(5));
        };
    });

    // ----------------- Hero -----------------
    let hero_handle = thread::spawn(move || {
        let mut dice = rand::thread_rng();
        let max_life = dice.gen_range(100..=200);
        let mut life = max_life;

        let mut potions = 3

        loop{
            let action = dice.gen_range(1..=3);
            match action {
                1 => {

                },
                2 => {

                },
                3 => println!("El heroe esta tomando un descanso"),
                _ => println!("Error generando accion de acción"),
            };
            thread::sleep(Duration::from_secs(3));
        };
    });


    monster_handle.join().expect("El hilo de monstruo fallo");

}

fn d20_action_validation_monster(d20 : i32) -> f32{
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