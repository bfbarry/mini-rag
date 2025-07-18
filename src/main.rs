pub mod parser;
pub mod models;
pub mod utils;
pub mod app;
pub mod server;
pub mod client;
pub mod grep;
// use app::AgentI;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::io;
use std::net::TcpStream;

#[allow(dead_code)]
enum LogLevel {
    DEBUG,
    NONE
}

const LOGLEVEL: LogLevel = LogLevel::NONE;

fn is_port_in_use(port: u16) -> io::Result<bool> {
    match TcpStream::connect(("127.0.0.1", port)) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

fn main() -> ort::Result<()> {

    match LOGLEVEL {
        LogLevel::DEBUG => {
            tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,ort=debug".into()))
            .with(tracing_subscriber::fmt::layer())
            .init();

        }
        LogLevel::NONE => ()
    }

    // let mut open_apiagent = app::OpenAPIAgent::new("src/data/openapi_eda.json")?;

    // let user_input = "what is the name of the task that creates edarequest?";
    
    
    // let ans =  open_apiagent.execute(user_input)?;
    // println!("Thinking...:\n\n{}", ans);

    match is_port_in_use(server::PORT) {
        Ok(in_use) => {
            if in_use {
                let _ = client::repl();
            } else {
                let mut api_ = server::API::new().unwrap();
                api_.listen();
            }
        }
        Err(e) => println!("Failed to check port {}: {}", server::PORT, e),
    }

    Ok(())
}