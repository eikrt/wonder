use async_std::task;
use crossbeam::channel::unbounded;
use rand::Rng;
use rayon::prelude::*;
use std::io;
use std::thread;
use std::time::Duration;
use U::net::{fetch_chunk, send_client_data, ClientData};
use U::plot::plot;
use U::renderer::{render_server, Camera};
use U::util::{ClientMsg, MainMsg, RenderMsg};
use U::worldgen::{worldgen, Entity, News, CHUNK_SIZE, WORLD_SIZE};

use lazy_static::lazy_static;
lazy_static! {
    pub static ref PARTITION_SIZE: usize = (*WORLD_SIZE as usize * *WORLD_SIZE as usize) / 16;
}
fn main() {
    let (tx, rx) = unbounded();
    let (tx2, rx2): (
        crossbeam::channel::Sender<MainMsg>,
        crossbeam::channel::Receiver<MainMsg>,
    ) = unbounded();
    let (tx3, rx3) = unbounded();
    let (tx4, rx4): (
        crossbeam::channel::Sender<ClientMsg>,
        crossbeam::channel::Receiver<ClientMsg>,
    ) = unbounded();
    let (tx_c, rx_c): (
        crossbeam::channel::Sender<ClientMsg>,
        crossbeam::channel::Receiver<ClientMsg>,
    ) = unbounded();
    let rx4_clone = rx4.clone();
    let rx2_clone = rx2.clone();
    let rx3_clone = rx3.clone();
    let tx3_clone = tx3.clone();
    let mut state: Vec<RenderMsg> = vec![];
    let mut rng = rand::thread_rng();
    let random_number = rng.gen_range(0..=100_000);
    let mut player: Entity = Entity::gen_player(random_number, 0.0, 0.0);
    let mut step = 0;
    let mut step_increment = 1;
    let mut camera = Camera::new();
    let mut vic_world = 0;
    let mut render = false;
    thread::spawn(move || loop {
        let _ = tx3.send(ClientMsg::from(player.clone()));
        let _ = tx_c.send(ClientMsg::from(player.clone()));
        if let Ok(p) = rx4.recv() {
            let mut player_from = p.player;
            player = player_from;
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    });
    thread::spawn(move || loop {
        // Continuously read from the channel until there are no more messages
        let mut latest_message = None;
        while let Ok(p) = rx_c.try_recv() {
            latest_message = Some(p);
        }
        // Process only the latest message, if any
        if let Some(p) = latest_message {
            let s = ClientData { entity: p.player };
            let result = task::block_on(send_client_data(s));
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    });
    thread::spawn(move || {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(e) => e,
            Err(e) => panic!("End..."),
        };
        match input.as_str() {
            "" => "joinks",
            &_ => todo!(),
        }
    });
    let mut partition = 0;

    thread::spawn(move || loop {
        state.clear();
        let result = task::block_on(fetch_chunk(0, 0, 0));
        match result {
            Ok(chunk) => {
                state.push(RenderMsg::from(chunk.clone(), chunk.inquire_news()));
            }
            Err(e) => eprintln!("Error fetching chunk: {}", e),
        }
        let _ = tx.send(state.clone());
        step += step_increment;
        if let Ok(x) = rx2_clone.recv() {
            camera = x.camera;
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        partition += 1;
    });
    render_server(&tx2, &rx, &tx4, &rx3);
}
