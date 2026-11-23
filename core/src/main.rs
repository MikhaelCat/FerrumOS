use ferrum_core::api::{handle_request, AppState};
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::thread;
use tiny_http::Server;

fn main() {
    println!("========================================================");
    println!("  FerrumOS Core Daemon v1.0.0 (Rust)                    ");
    println!("  Architecture: MediaTek MT7620A (600MHz MIPS 24Kc)    ");
    println!("  Anti-Brick Protection: ACTIVE (MTD0/MTD2 Locked)      ");
    println!("  Protocols: VLESS Reality (XTLS-Vision) & Hysteria 2   ");
    println!("========================================================");

    let bind_addr = env::var("FERRUM_BIND").unwrap_or_else(|_| "0.0.0.0:8080".into());
    let addr: SocketAddr = bind_addr
        .parse()
        .unwrap_or_else(|_| "0.0.0.0:8080".parse().unwrap());

    let server = match Server::http(addr) {
        Ok(s) => {
            println!("[FerrumOS] HTTP Web UI & REST API listening on http://{}", addr);
            Arc::new(s)
        }
        Err(e) => {
            eprintln!("[FerrumOS] ERROR: Failed to bind HTTP server to {}: {}", addr, e);
            std::process::exit(1);
        }
    };

    let state = AppState::new();

    // Spawn 2 lightweight worker threads for MT7620 single-core / low RAM footprint
    let num_threads = 2;
    let mut handles = Vec::new();

    for i in 0..num_threads {
        let server_clone = Arc::clone(&server);
        let state_clone = state.clone();

        let handle = thread::Builder::new()
            .name(format!("ferrum-worker-{}", i))
            .spawn(move || {
                loop {
                    match server_clone.recv() {
                        Ok(request) => {
                            handle_request(request, &state_clone);
                        }
                        Err(e) => {
                            eprintln!("[FerrumOS] Worker {} recv error: {}", i, e);
                            break;
                        }
                    }
                }
            })
            .expect("Failed to spawn worker thread");

        handles.push(handle);
    }

    println!("[FerrumOS] Started {} worker threads. Ready for connections.", num_threads);

    for handle in handles {
        let _ = handle.join();
    }
}
