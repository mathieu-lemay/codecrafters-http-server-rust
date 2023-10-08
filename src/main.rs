use std::io::{Read, Result, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::str;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                handle(&mut _stream);
                _stream
                    .shutdown(Shutdown::Both)
                    .expect("Error closing stream");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle(stream: &mut TcpStream) {
    let addr = stream.peer_addr().expect("Unable to get peer_addr");
    println!("accepted new connection from {:?}", addr);

    let req = match get_request(stream) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error reading stream: {:?}", e);
            return;
        }
    };

    let resp = if req.path == "/" {
        "HTTP/1.1 200 OK\r\n\r\n"
    } else {
        "HTTP/1.1 404 Not Found\r\n\r\n"
    };

    match stream.write(resp.as_bytes()) {
        Ok(n) => println!("Successfully wrote {} bytes to stream", n),
        Err(e) => eprintln!("Error writing to stream: {:?}", e),
    }
}

struct Request {
    method: String,
    path: String,
}

fn get_request(stream: &mut TcpStream) -> Result<Request> {
    let mut buf = [0; 2048];

    let n = stream.read(&mut buf)?;
    println!("Successfully read {} bytes from stream", n);

    let start_line = str::from_utf8(&buf)
        .expect("Invalid http request")
        .split("\r\n")
        .next()
        .expect("No start line found");

    println!("Start line: {}", start_line);
    let parts: Vec<&str> = start_line.split(" ").collect();

    return Ok(Request {
        method: String::from(parts[0]),
        path: String::from(parts[1]),
    });
}
