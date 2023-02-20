use std::fs;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use anyhow::Ok;
use openssl::ssl::{SslConnector, SslMethod};
use std::io;


fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let firstarg = &args[0];
    let mut req = request::new();
    let mut port: usize;
    let c: Vec<&str> = firstarg.split("/").collect();
    let mut stream: TcpStream;
    req.c_hostname(c[2].to_string());
    req.c_file("/".to_owned() +  &c[3..].join("/"));
    req.c_protocol(c[0].to_string());
    if req.protocol.contains("s"){
        port = 443;
    }
    else{
        port = 80;
    }
    if req.hostname.contains(":"){
        let temp: Vec<&str> = c[2].trim().split(":").collect();
        req.c_hostname(temp[0].to_string());
        port = temp[1].parse().unwrap();
    }
    let mut message = getmessage(req.file.clone(), req.hostname.clone(), req.protocol.clone());
    println!("{message}");
    println!("{port}");
    let mut response = String::new();
    if req.protocol.contains("s"){
        let tcp = TcpStream::connect(format!("{}:{}", &req.hostname.clone(), port)).unwrap();
        let connector = SslConnector::builder(SslMethod::tls()).unwrap().build();
        let mut stream = connector.connect(&req.hostname.clone(), tcp).unwrap();
        stream.write(message.as_bytes());
        stream.read_to_string(&mut response).unwrap();
    }
    else{
        stream = TcpStream::connect(req.hostname.clone() + &":".to_string() + &port.to_string()).unwrap();
        stream.write(message.as_bytes());     
        
        stream.read_to_string(&mut response).unwrap();
    }
    let filepath = &req.file.clone()[1..].to_string();
    println!("{filepath}");
    let mut f = File::create(filepath.clone().replace("/", "_")).unwrap();
    
    let mut keep: bool = false;
    let mut newString = String::new();
    let v: Vec<&str> = response.split("\n").collect();
    for line in v{
        
        if !keep{
            println!("{}",line.to_string());
            if line.to_string().trim().is_empty(){
                keep = true;
                continue;
            }
        }
        else{
            newString.push_str(&(line.to_string() + "\n").to_owned());

        }

    }
    f.write_all(newString.as_bytes());
    
}

fn getmessage(file: String, host: String, protocol: String) -> String{
    let mut s = String::new();
    if protocol.contains("s"){
        s = "GET ".to_owned() + &file + &" HTTP/1.1\r\nHost: ".to_owned() + &host +&"\r\nConnection: Close\r\n\r\n".to_owned();
    }
    else{
        s = "GET ".to_owned() + &file + &" HTTP/1.1\r\nHost: ".to_owned() + &host +&"\r\nConnection: Open\r\n\r\n".to_owned();
    }
    return s
}
#[derive(Debug)]
struct request {
    file: String,
    hostname: String,
    protocol: String
}

impl request {
    fn new() -> Self {
        request { file: String::new(), hostname: String::new(), protocol: String::new() }
    }

    fn c_file (&mut self, f: String) {
        self.file = f;
    }

    fn c_hostname (&mut self, h: String) {
        self.hostname = h;
    }

    fn c_protocol (&mut self, p: String) {
        self.protocol = p;
    }

}

