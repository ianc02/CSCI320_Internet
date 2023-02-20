use std::fs;
use std::env;


fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let firstarg = &args[0];
    let mut firstPass = false;
    for file in &args{
        if firstPass{
            println!("{}", file);
            println!("");
            let contents = fs::read_to_string(file).expect("Usage: Something went wrong.");
            for line in contents.split("\n"){
                if line.contains(firstarg){
                    println!("{line}");
                }
            }
            println!("");
            println!("");
        }
        else{firstPass=true;}
    
    }
    
}

