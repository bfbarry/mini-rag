use std::{
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
};
use std::collections::HashMap;
use crate::app::{self, OpenAPIAgent};

// TODO: use a temporal LRU

pub const PORT: u16 = 7878;
pub struct API {
    // listener: TcpListener,
    agents: HashMap<String,  Box<dyn app::AgentI>>
}

impl API {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let agents = HashMap::new();
        Ok(Self {
            agents
        })
    }
    
    pub fn listen(&mut self) {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", PORT)).unwrap();
        println!("Listening on {}", PORT);
        for stream in listener.incoming() {
            self.handle_connection(stream.unwrap());
        }
    }

    pub fn handle_connection(&mut self, mut stream: TcpStream) {
        // let buf_reader = BufReader::new(&stream);
        // let mut request = Vec::new();

        // for line in buf_reader.lines() {
        //     println!("new line!");
        //     let line = line.unwrap();
        //     if line == "\n" || line.trim().is_empty() {
        //         break;
        //     }
        //     request.push(line);
        // }
    
        // for line in request {
        //     println!("executing...");
        //     self.execute(&line, &mut stream);
        // }

        let reader_stream = stream.try_clone().unwrap(); // For reading
        let mut buf_reader = BufReader::new(reader_stream); // Owns the cloned stream
        let mut writer = stream; // The original stream is now for writing
    
        loop {
            let mut line = String::new();
            let bytes_read = buf_reader.read_line(&mut line).unwrap();
    
            if bytes_read == 0 {
                break; // client disconnected
            }
    
            let line = line.trim_end();
            if line.is_empty() {
                continue;
            }
    
            self.execute(line, &mut writer); // Now safe: writer is a separate stream
        }
        // println!("Request: {:#?}", request);
    }

    pub fn execute(&mut self, line: &str, stream: & mut TcpStream) {
        let split: Vec<&str> = line.split_whitespace().collect();
        if split.len() < 2 {
            let err = "Invalid input";
            stream.write(err.as_bytes()).unwrap();
            return;
        }

        // need an actual parser
        let ans = match split[0] {
            "api" => {
                // stream.write("Good job".as_bytes()).unwrap();
                self.init_openapiagent(split[1])
            }, 
            "apiask" => {
                let s = &split[1..].join(" ");
                self.ask_apiagent(s)
            }
            _ => Ok("Unknown input".to_string())
        }.unwrap();

        let msg = format!("{}\n", ans);
        let size = msg.len() as u32;
        stream.write_all(&size.to_be_bytes()).unwrap();
        stream.write_all(msg.as_bytes()).unwrap();
        stream.flush().unwrap();
    }

    fn init_openapiagent(&mut self, mut fpath: &str) -> Result<String, Box<dyn std::error::Error>> {
        let test_key = "/Users/brianbarry/Desktop/ucsd-its/mini_rag/src/data/openapi_eda.json";
        fpath = test_key;
        let agent = app::OpenAPIAgent::new(fpath).unwrap();

        self.agents.insert(fpath.to_string(), Box::new(agent));
        Ok("Success".to_string())
    }

    fn ask_apiagent(&mut self, query: &str) -> Result<String, Box<dyn std::error::Error>> {
        let test_key = "/Users/brianbarry/Desktop/ucsd-its/mini_rag/src/data/openapi_eda.json";
        let ag: & mut Box<dyn app::AgentI> = self.agents.get_mut(test_key).unwrap();
        let res = ag.execute(query)?;
        Ok(res)

    }
}