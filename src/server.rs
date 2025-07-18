use std::{
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    error::Error
};
use std::collections::HashMap;
use crate::app;
// TODO: use a temporal LRU

pub const PORT: u16 = 7878;
pub struct API{
    agents: HashMap<String,  Box<dyn app::AgentI>>,
    curr_agent: String, // the agent the user is currently querying
}

impl API {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let agents = HashMap::new();
        Ok(Self {
            agents,
            curr_agent: String::new()
        })
    }
    
    pub fn listen(&mut self) {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", PORT)).unwrap();
        println!("Listening on {}", PORT);
        for stream in listener.incoming() {
            self.handle_connection(stream.unwrap());
        }
    }

    pub fn handle_connection(&mut self, stream: TcpStream) {
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
    }

    pub fn execute(&mut self, line: &str, stream: & mut TcpStream) {
        let words: Vec<&str> = match line.split_whitespace().collect::<Vec<&str>>().as_slice() {
            [] => vec![line],
            split => split.to_vec()
        };
        // need an actual parser e.g., add api
        let ans = match words[0] {
            "apiadd" => {
                self.init_openapiagent(words[1])
            },
            "ask" => {
                let q = &words[1..].join(" ");
                self.ask_agent(q)
            }
            "set" => self.set_agent(&words[1..].join(" ")),
            "ls" => self.list_agents(),
            _ => Ok("Unknown input".to_string())
        }.unwrap();

        let msg = format!("{}\n", ans);
        let size = msg.len() as u32;
        stream.write_all(&size.to_be_bytes()).unwrap();
        stream.write_all(msg.as_bytes()).unwrap();
        stream.flush().unwrap();
    }

    fn set_agent(&mut self, ag: &str) -> Result<String, Box<dyn Error>> {
        self.curr_agent = ag.to_string();
        Ok("Success".to_string())
    }
    fn list_agents(&self) -> Result<String, Box<dyn Error>> {
        let mut res = String::new();

        for (i, k) in self.agents.keys().enumerate() {
            let mut line = format!("[{}] {}\n", i+1, k);
            if *k == self.curr_agent {
                line.push_str("(current)");
            }
            res.push_str(&line);
        }

        Ok(res)
    }

    fn init_openapiagent(&mut self, url: &str) -> Result<String, Box<dyn Error>> {
        // let test_key = "/Users/brianbarry/Desktop/ucsd-its/mini_rag/src/data/openapi_eda.json";
        // fpath = test_key;
        let agent = app::OpenAPIAgent::new(url).unwrap();

        let key = format!("OpenAPI agent @ {}", url);
        if self.agents.contains_key(&key) {
            return Ok("Key already exists".to_string())
        }
        self.agents.insert(key.to_string(), Box::new(agent));
        self.curr_agent = key.to_string(); // TODO: use ref instead
        Ok("Success".to_string())
    }

    fn ask_agent(&mut self, query: &str) -> Result<String, Box<dyn Error>> {
        // let test_key = "/Users/brianbarry/Desktop/ucsd-its/mini_rag/src/data/openapi_eda.json";

        let ag: & mut Box<dyn app::AgentI> = self.agents.get_mut(&self.curr_agent).unwrap();
        let res = ag.execute(query)?;
        Ok(res)

    }
}