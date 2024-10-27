use hyper::server::Server;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response};
use lazy_static::lazy_static;
use rand::Rng;
use rayon::prelude::*;
use std::convert::Infallible;
use std::io;
use std::thread;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::task;
use tokio::time::{sleep, Duration};
use url::{form_urlencoded, Url};
use U::util::RenderMsg;
use U::worldgen::{worldgen, News, CHUNK_SIZE, WORLD_SIZE};
lazy_static! {
    pub static ref PARTITION_SIZE: usize = (*WORLD_SIZE as usize * *WORLD_SIZE as usize) / 16;
}
fn main2() {}

// Function to handle requests and route to the appropriate response
async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    match req.uri().path() {
        "/render_msg" => {
            // The binary content to be returned
            let binary_message: &[u8] = b"Hello, this is a binary response!";

            // Create a response with the binary content
            Ok(Response::new(Body::from(binary_message)))
        }
        _ => {
            // Default 404 for any other routes
            Ok(Response::builder()
                .status(404)
                .body(Body::from("Not Found"))
                .unwrap())
        }
    }
}

#[tokio::main]
async fn main() {
    // Create a broadcast channel (tx: sender, rx: receiver)
    let (tx, _rx) = broadcast::channel(256);

    thread::spawn(move || {});
    let mut worlds = vec![];
    let mut rng = rand::thread_rng();
    for _ in 0..1 {
        let seed = rng.gen_range(0..1000);
        worlds.push(worldgen(seed));
    }
    let mut state: Vec<RenderMsg> = vec![];
    let mut step = 0;
    let mut step_increment = 1;
    let mut vic_world = 0;
    let mut render = false;
    thread::spawn(move || {
        //plot();
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
    // Spawn a worker thread to send "world data" every few seconds
    task::spawn(async move {
        let mut counter = 0;
        loop {
            worlds
                .par_iter_mut()
                .for_each(|c| c.resolve(step_increment));
            worlds
                .par_iter_mut()
                .for_each(|c| c.resolve_between(step_increment));
            let worlds_clone = worlds.clone(); 
            counter += 1;
            // Send the world data through the broadcast channel
            if let Err(_) = tx.send(worlds_clone) {
                println!("No active receivers left");
            }
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 1));
        }
    });

    // Create a Hyper service that will serve the data received from the broadcast channel
    let make_svc = make_service_fn(move |_conn| {
        // Clone the broadcast receiver for each connection
        let mut rx = _rx.resubscribe(); // Creates a new receiver from the existing broadcast channel
        let service = service_fn(move |req: Request<Body>| {
            let mut rx = rx.resubscribe(); // Clone the receiver inside the async block

            async move {
		while rx.is_empty() {
		}
                match req.uri().path() {
                    "/chunk" => {
                        let uri_string = req.uri().to_string();
                        let base_url = "http://dummy.com";

                        // Assuming `uri_string` is a relative path

                        // Create the full URL by appending the relative URI to the base
                        let request_url = Url::parse(base_url)
                            .unwrap()
                            .join(&uri_string) // This method joins the base URL with the relative path
                            .unwrap();
                        let params = request_url.query_pairs();

                        // Initialize variables for optional parameters
                        let mut x: Option<f32> = None;
                        let mut y: Option<f32> = None;
                        let mut index: Option<usize> = None;

                        // Iterate over parameters to assign values
                        for (key, value) in params {
                            match key.into_owned().as_str() {
                                "x" => x = value.parse().ok(),
                                "y" => y = value.parse().ok(),
                                "index" => index = value.parse().ok(),
                                _ => {}
                            }
                        }

                        // Process the parameters (this is just a demonstration)
                        let response_message =
                            format!("Received - x: {:?}, y: {:?}, index: {:?}", x, y, index);
                        // Receive data from the broadcast channel (non-blocking)
                        match rx.recv().await {
                            Ok(worlds_clone) => {
				
                                // Return the received world data in the response
				let chunk = worlds_clone[index.unwrap()].fetch_chunk_x_y(x.unwrap(), y.unwrap());
                                Ok::<_, Infallible>(Response::new(Body::from(bincode::serialize(chunk).unwrap())))
                            }
                            Err(_) => {
                                // If no data is available or there's an error, return a 500 error
                                Ok::<_, Infallible>(
                                    Response::builder()
                                        .status(500)
                                        .body(Body::from("Failed to receive world data"))
                                        .unwrap(),
                                )
                            }
                        }
                    }
                    "/worlds" => {
                        // Receive data from the broadcast channel (non-blocking)
                        match rx.recv().await {
                            Ok(worlds_clone) => {
                                // Return the received world data in the response
                                Ok::<_, Infallible>(Response::new(Body::from(bincode::serialize(&worlds_clone).unwrap())))
                            }
                            Err(_) => {
                                // If no data is available or there's an error, return a 500 error
                                Ok::<_, Infallible>(
                                    Response::builder()
                                        .status(500)
                                        .body(Body::from("Failed to receive world data"))
                                        .unwrap(),
                                )
                            }
                        }
                    }
                    _ => {
                        // Default 404 for any other routes
                        Ok::<_, Infallible>(
                            Response::builder()
                                .status(404)
                                .body(Body::from("Not Found"))
                                .unwrap(),
                        )
                    }
                }
            }
        });

        async { Ok::<_, Infallible>(service) }
    });

    // Bind the server to an address
    let addr = ([127, 0, 0, 1], 3000).into();
    let server = Server::bind(&addr).serve(make_svc);

    // Print server info
    println!("Listening on http://{}", addr);

    // Run the server
    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}
