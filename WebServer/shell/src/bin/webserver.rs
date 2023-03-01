use std::fs;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::vec;
use anyhow::Ok;
use openssl::ssl::{SslConnector, SslMethod};
use std::io;
use std::{thread, sync::{Mutex, Arc}};
use std::str::from_utf8;
use std::path::PathBuf;


fn main() {

    let listener = TcpListener::bind("localhost:8888").unwrap();
    let count = Arc::new(Mutex::new(Counter::new()));
    

    for stream in listener.incoming() {
        let count = count.clone();
        thread::spawn(move || {
            let mut valid = handle_incoming(stream.unwrap());
            let mut count = count.lock().unwrap();
            if valid{
                count.valid_inc();
            }
            count.total_inc();
            println!("Valid Requests: {}. Total Requests: {}", count.valid_requests, count.total_requests);
        });
    }
}

fn handle_incoming(mut stream: TcpStream) -> bool{
    println!("Client IP Address is: {}",stream.peer_addr().unwrap());
    let mut client_message = String::new();
    let mut buff: Vec<u8> = vec![0;500];
    loop {
        stream.read(&mut buff).unwrap();
        let s = from_utf8(&buff).unwrap();
        client_message.push_str(s);
        client_message = client_message.trim_end_matches("\0").to_string();
        if client_message.contains("\r\n\r\n") || client_message.contains("\n\n") {
            break;
        }
    }
    println!("{}", client_message);
    let mut requested_file = String::new();
    let split_message: Vec<&str> = client_message.split(" ").collect();
    requested_file = split_message[1].to_string();

    let path = env::current_dir().unwrap().into_os_string().into_string().unwrap() + &requested_file.to_string();
    let mut whole_path = PathBuf::from(&path);
    //whole_path.push(&requested_file);
    let mut test = &whole_path.as_path();
    if test.exists(){

        let absolute_path = fs::canonicalize(&path).unwrap();

        if !absolute_path.starts_with(env::current_dir().unwrap()){

            let mut html_message = "<html>\n<body>\n<h1>HTTP/1.1 403 Forbidden</h1>\nRequested file: ".to_string() +&requested_file.to_string() + "<br>\n</body>\n</html>";
            let content_length = html_message.as_bytes().len();
            let whole_message = "HTTP/1.1 403 Forbidden\nContent-Type: text/html; charset=UTF-8\nContent-Length: ".to_string() + &content_length.to_string() + "\n\n" + &html_message.to_string(); 
            stream.write(whole_message.as_bytes());

        }
        else{
            

            if test.is_dir(){
                // requested_file = requested_file + "/index.html";
                // println!("{}", requested_file);
                let mut html_message = "<html>\n<body>\n<h1>HTTP/1.1 404 Not Found</h1>\nRequested file: ".to_string() +&requested_file.to_string() + "<br>\n</body>\n</html>";
                let content_length = html_message.as_bytes().len();
                let whole_message = "HTTP/1.1 404 Not Found\nContent-Type: text/html; charset=UTF-8\nContent-Length: ".to_string() + &content_length.to_string() + "\n\n" + &html_message.to_string(); 
                stream.write(whole_message.as_bytes());
            }
            else{
                let mut file = File::open(&path).unwrap();
                let mut to_client = fs::read_to_string(whole_path).unwrap();
                let byte_len = to_client.as_bytes().len();
                let return_message = "HTTP/1.1 200 OK\nContent-Type: text/html; charset=UTF-8\nContent-Length: ".to_string() + &byte_len.to_string() + "\n\n" + &to_client.to_string();

                stream.write(return_message.as_bytes());
                return true;
            }
            
        }
        
    }
    else{
        let mut html_message = "<html>\n<body>\n<h1>HTTP/1.1 404 Not Found</h1>\nRequested file: ".to_string() +&requested_file.to_string() + "<br>\n</body>\n</html>";
                let content_length = html_message.as_bytes().len();
                let whole_message = "HTTP/1.1 404 Not Found\nContent-Type: text/html; charset=UTF-8\nContent-Length: ".to_string() + &content_length.to_string() + "\n\n" + &html_message.to_string(); 
                stream.write(whole_message.as_bytes());
    }  
    return false;
}

struct Counter {
    total_requests: i64,
    valid_requests: i64
}

impl Counter {
    fn new() -> Self {
        Counter { total_requests: 0, valid_requests: 0}
    }

    fn total_inc (&mut self) {
        self.total_requests +=1;
    }

    fn valid_inc (&mut self) {
        self.valid_requests +=1;
    }

}

