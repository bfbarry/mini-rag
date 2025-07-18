use std::io::{self, BufReader, Read, Write};
use std::net::TcpStream;
use crate::server;

pub fn repl() -> Result<(), Box<dyn std::error::Error>> {
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