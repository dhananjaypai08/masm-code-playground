use std::env;

#[cfg(feature = "web_server")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse::<u16>()
        .unwrap_or(3001);

    println!("🔧 Starting Miden VM Web API Server on port {}...", port);
    
    miden_app_dj_lib::web_server::start_server(port).await
}

#[cfg(not(feature = "web_server"))]
fn main() {
    eprintln!("ERROR: web_server feature not enabled!");
    eprintln!("Build with: cargo run --bin web_server --features web_server");
    std::process::exit(1);
}