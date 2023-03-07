use std::collections::HashMap;
use std::fs;
use std::env;
use std::fs::File;
use std::hash::Hash;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::process::exit;
use std::vec;
use libc::iscntrl;
use openssl::ssl::{SslConnector, SslMethod};
use std::io;
use std::{thread, sync::{Mutex, Arc}};
use std::str::from_utf8;
use std::path::PathBuf;
use std::result::Result::Ok;


fn main() {

    let listener = TcpListener::bind("localhost:8888").unwrap();
    let count = Arc::new(Mutex::new(Counter::new()));
    
    let args: Vec<String> = env::args().skip(1).collect();
    let mut is_streaming = false;
    let mut is_caching = false;
    let mut num_cache = 0;
    let cmap: Arc<Mutex<cacheMap>> = Arc::new(Mutex::new(cacheMap::new()));
    if !args.is_empty(){
        let firstarg = &args[0];
        if firstarg.chars().next().unwrap() == '-'{
            if firstarg.contains('s'){is_streaming=true;}
            if firstarg.contains('c'){
                is_caching=true;
                let mut tempnum:Vec<&str> = firstarg.split("=").collect();
                let a = tempnum.pop().unwrap();
                let mut t = String::new();
                for c in a.chars(){
                    if c.is_numeric(){
                        t += c.to_string().as_str();
                    }
                }
                num_cache = t.parse().unwrap();
                cmap.lock().unwrap().change_max(num_cache);

            }
        }
    }
    for stream in listener.incoming() {
        let count = count.clone();
        let cmap = cmap.clone();
        thread::spawn(move || {
            let temphash = cmap.lock().unwrap().files.clone();
            let (mut valid, mut req_file) = handle_incoming(stream.unwrap(), is_streaming, is_caching, temphash);
            let mut count = count.lock().unwrap();
            let mut cmap = cmap.lock().unwrap();
            if valid{
                count.valid_inc();
            }
            count.total_inc();
            cmap.add_to_map(req_file);
            cmap.update();
            println!("{:?}",cmap.popular);

            println!("Valid Requests: {}. Total Requests: {}", count.valid_requests, count.total_requests);
        });
    }
}
fn handle_incoming(mut stream: TcpStream, is_streaming: bool, is_caching: bool, cmap: HashMap<String, String>) -> (bool, String){
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
                if (is_caching){
                    if cmap.contains_key(&requested_file){
                        let mut to_client = cmap.get(&requested_file).unwrap();
                        let byte_len = to_client.as_bytes().len();
                        let return_message = "HTTP/1.1 200 OK\nContent-Type: text/html; charset=UTF-8\nContent-Length: ".to_string() + &byte_len.to_string() + "\n\n" + &to_client.to_string();
            
                        stream.write(return_message.as_bytes());
                        return (true, requested_file);
                    }
                }
                if is_streaming {
                    let mut buffer = [0; 1024];
                    let mut count = 0;
                    let mut response_sent = false;
                    loop {
                        let bytes_read = file.read(&mut buffer).unwrap();
                        if bytes_read == 0 {
                            break;
                        }
                        let to_client = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
                        let byte_len = to_client.as_bytes().len();
                        let content = to_client.to_string();
                        let mut return_message = String::new();
                        if !response_sent {
                            return_message += "HTTP/1.1 200 OK\nContent-Type: text/html; charset=UTF-8\n\n";
                            response_sent = true;
                        }
                        return_message += &content;
                        count += 1;
                        match stream.write(return_message.as_bytes()) {
                            Ok(bytes_written) => {
                                //println!("Successfully went {} loops and wrote: {} Bytes", count, bytes_written);
                            }
                            Err(e) => {
                                println!("Failed on loop {}, at error {}", count, e);
                                break;
                            }
                        }
                    }
                }
                else{
                let mut to_client = fs::read_to_string(whole_path).unwrap();
                let byte_len = to_client.as_bytes().len();
                let return_message = "HTTP/1.1 200 OK\nContent-Type: text/html; charset=UTF-8\nContent-Length: ".to_string() + &byte_len.to_string() + "\n\n" + &to_client.to_string();

                stream.write(return_message.as_bytes());
                }
                return (true, requested_file);
            }
            
        }
        
    }
    else{
        let mut html_message = "<html>\n<body>\n<h1>HTTP/1.1 404 Not Found</h1>\nRequested file: ".to_string() +&requested_file.to_string() + "<br>\n</body>\n</html>";
                let content_length = html_message.as_bytes().len();
                let whole_message = "HTTP/1.1 404 Not Found\nContent-Type: text/html; charset=UTF-8\nContent-Length: ".to_string() + &content_length.to_string() + "\n\n" + &html_message.to_string(); 
                stream.write(whole_message.as_bytes());
    }  
    return (false, requested_file);
}


struct cacheMap {
    map:HashMap<String, i32>,
    popular: Vec<String>,
    files: HashMap<String, String>,
    max: i32,
}

impl cacheMap{
    fn new() -> Self{
        cacheMap { 
            map: HashMap::new(), 
            popular: vec![], 
            files: HashMap::new(),
            max: 0,
        }
    }

    fn add_to_map(&mut self, s: String){
        *self.map.entry(s).or_insert(0) +=1;
    }

    fn update(&mut self) {
        let tempList = self.popular.clone();
        self.popular = self.map.iter().map(|(k, _)| k.clone()).collect();
        self.popular.sort_by_key(|k| std::cmp::Reverse(self.map.get(k).unwrap()));
        while self.popular.len() > self.max.try_into().unwrap(){
            self.popular.pop();
        }
        let counter = 0;
        let mut tempfile = self.files.clone();
        for (f,v) in self.files.iter_mut(){
            if !(self.popular.contains(f)){
                tempfile.remove(f);
            }
        }
        self.files = tempfile;
        for s in self.popular.clone(){
            if !tempList.contains(&s){ 
                let path = env::current_dir().unwrap().into_os_string().into_string().unwrap() + &s.to_string();
                let mut whole_path = PathBuf::from(&path);
                let mut to_client = fs::read_to_string(whole_path).unwrap();
                self.files.insert(s, to_client);
                
            }

        }
    }

    fn change_max(&mut self, n:i32){
        self.max = n;
    }
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

