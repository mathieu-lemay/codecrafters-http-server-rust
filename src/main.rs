use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};

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

    let mut buf: Vec<u8> = Vec::new();

    if let Err(e) = stream.read(&mut buf) {
        eprintln!("Error reading stream: {:?}", e);
        return;
    }
    println!("Successfully read {} bytes from stream", buf.len());

    let resp = "HTTP/1.1 200 OK\r\n\r\n".as_bytes();
    match stream.write(resp) {
        Ok(n) => println!("Successfully wrote {} bytes to stream", n),
        Err(e) => eprintln!("Error writing to stream: {:?}", e),
    }
}
