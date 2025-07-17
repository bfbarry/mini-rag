pub mod parser;
pub mod models;
pub mod utils;
pub mod app;
pub mod server;
// use app::AgentI;
// use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};


use std::io::{self, BufReader, Read, Write};
use std::net::TcpStream;

fn is_port_in_use(port: u16) -> io::Result<bool> {
    match TcpStream::connect(("127.0.0.1", port)) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

fn repl() -> Result<(), Box<dyn std::error::Error>> {
    let server_addr = &format!("127.0.0.1:{}", server::PORT);
    let stream = TcpStream::connect(server_addr)?;
    let mut writer = stream.try_clone()?; // for writing
    let mut reader = BufReader::new(stream); // for reading

    println!("Connected to server at {}", server_addr);

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input == "exit" {
            println!("Exiting REPL.");
            break;
        }
        writer.write_all(input.as_bytes())?;
        writer.write_all(b"\n")?;
        writer.flush()?;

        let mut len_buf = [0u8; 4];
        let _ = reader.read_exact(&mut len_buf);
        let len = u32::from_be_bytes(len_buf) as usize;

        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf)?;
        let res = String::from_utf8_lossy(&buf);


        println!("{}", res.trim_end());
    }

    Ok(())
}

fn main() -> ort::Result<()> {
    // tracing_subscriber::registry()
    // .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,ort=debug".into()))
    // .with(tracing_subscriber::fmt::layer())
    // .init();

    // let mut open_apiagent = app::OpenAPIAgent::new("src/data/openapi_eda.json")?;

    // let user_input = "what is the name of the task that creates edarequest?";
    
    
    // let ans =  open_apiagent.execute(user_input)?;
    // println!("Thinking...:\n\n{}", ans);

    match is_port_in_use(server::PORT) {
        Ok(in_use) => {
            if in_use {
                let _ = repl();
            } else {
                let mut api_ = server::API::new().unwrap();
                api_.listen();
            }
        }
        Err(e) => println!("Failed to check port {}: {}", server::PORT, e),
    }

    Ok(())
}