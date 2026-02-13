// Standard Libraries
use std::thread;
use std::sync::mpsc;
use std::time::Duration;

// External Dependencies
use rand::Rng;


fn main() {
    // ----------- Comunicación entre hilos ------------
    let (tx_m, rx_m) = mpsc::channel::<(i32, i32)>(); // comunicador entre hilo monstruo a heroe
    let (tx_h, rx_h) = mpsc::channel::<(i32, i32)>(); // Comunicador entre hilo heroe a monstruo


    // ------------ Monster -----------------------
    let monster_handle = thread::spawn( move || {

        let mut dice = rand::thread_rng();
        let max_life = dice.gen_range(400..=500);
        let mut life = max_life;

        let _defense = 10;

        loop{
            // Checa si la vida llego a 0
            while let Ok((x,y)) = rx_m.try_recv(){
                if ( x == 0 ) && ( y == 0 ) {
                    println!("Monstruo: Y asi cayo el dichoso heroe");
                    return;
                }

                match y {
                    0 => println!("El ataque no tuvo efecto"),
                    _ => {
                        if (life - y) <= 0 {
                        life = 0;
                        }
                        else{
                            life = life - y;
                        } 
                    },
                }
            }

            if life <= 0 {
                println!("El monstruo fue derrotado en el campo de batalla");
                let _ = tx_h.send((0, 0)).unwrap();
                return;
            };

            let action : i32;
            if life <= (max_life/2) {
                action = dice.gen_range(1..=3);
            }
            else{
                action = 2;
            }

            let action_factor = d20_action_validation(dice.gen_range(1..=20 as i32));
            // La accion que realizara este turno el jefe
            match action {
                1 => { // Healing
                    let healing = (50.0 * action_factor) as i32;
                    life = life + healing;
                    if (life + healing) >= max_life {
                        life = max_life;
                        println!("Monstruo [{}/{}] : El monstruo ya curó todas sus heridas", life, max_life);
                    }
                    else{
                        life = life + healing;
                        println!("Monstruo [{}/{}] : El monstruo se curo {}, pasa a tener {} de vida", life, max_life, healing, life);
                    }
                    
                },
                2 => { // Attack
                    let attack : i32 = (((dice.gen_range(1..=6) as f32) + (dice.gen_range(1..=6) as f32)) * action_factor) as i32;
                    println!("Monstruo [{}/{}] : El monstruo ataco al heroe con {}", life, max_life, attack);
                    let _ = tx_h.send((life, attack)).unwrap();

                },
                3 => println!("Monstruo [{}/{}] : El monstruo decidio descansar este turno", life, max_life,),
                _ => println!("Monstruo [{}/{}] : Error generating an action!!", life, max_life,),
            }

            thread::sleep(Duration::from_secs(3));
        };
    });

    // ----------------- Hero -----------------
    let hero_handle = thread::spawn( move || {
        // Handler de comunicación

        let mut dice = rand::thread_rng();
        let max_life = dice.gen_range(100..=200);
        let mut life = max_life;

        let mut potions = 3;

        loop{
            while let Ok((x,y)) = rx_h.try_recv(){
                if ( x == 0 ) && ( y == 0 ) {
                    println!("Heroe: Caiste bestia");
                    return;
                }

                match y {
                    0 => println!("El ataque no tuvo efecto"),
                    _ => {
                        if (life - y) <= 0 {
                        life = 0;
                        }
                        else{
                            life = life - y;
                        } 
                    },
                }
            }

            if life <= 0 {
                println!("El heroe fallecio en el campo de batalla");
                let _ = tx_m.send((0, 0)).unwrap();
                return;
            }

            let action : i32;
            if life <= (max_life/2) {
                action = dice.gen_range(1..=3);
            }
            else{
                action = 2;
            }

            match action {
                1 => {
                    println!("Heroe [{}/{}] : El heroe esta tratando de currarse", life, max_life);
                    if life >= (max_life/2) {
                        println!("Heroe [{}/{}] : El heroe aun tiene suficiente vida", life, max_life);
                        
                    }
                    else{
                        if potions > 0 {
                            potions = potions - 1;
                            if max_life <= (life + 50) {
                                life = max_life;
                            }
                            else{
                                life = life + 50;
                            }
                            println!("Heroe [{}/{}] : El heroe tomo una pocion, le quedan {}", life, max_life, potions);
                        }
                        else {
                            println!("Heroe [{}/{}] : El heroe trato de curarse, pero ya no tiene pociones",life, max_life);
                        }   
                        
                    }
                },
                2 => {
                    let _d20 = dice.gen_range(1..=20);
                    let _d8_1 = dice.gen_range(1..=8);
                    let _d8_2 = dice.gen_range(1..=8);
                    let attack = (((_d8_1 + _d8_2) as f32) * d20_action_validation(_d20)) as i32;
                    println!("Heroe [{}/{}] : El heroe esta atacando con {}", life, max_life, attack);
                    let _ = tx_m.send((life, attack));
                },
                3 => println!("Heroe [{}/{}] : El heroe esta tomando un descanso", life, max_life,),
                _ => println!("Heroe [{}/{}] : Error generando accion de acción", life, max_life,),
            };
            thread::sleep(Duration::from_secs(1));
        };
    });

    hero_handle.join().expect("Error en el hilo heroe");
    monster_handle.join().expect("Error en el hilo monstruo");

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
