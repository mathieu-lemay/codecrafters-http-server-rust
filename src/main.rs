use std::collections::HashMap;
use std::io::{Read, Result, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::str;
use std::thread;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                thread::spawn(|| handle(_stream));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle(mut stream: TcpStream) {
    let addr = stream.peer_addr().expect("Unable to get peer_addr");
    println!("accepted new connection from {:?}", addr);

    let req = match get_request(&mut stream) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error reading stream: {:?}", e);
            return;
        }
    };

    let resp = if req.path == "/" {
        "HTTP/1.1 200 OK\r\n\r\n".to_string()
    } else if req.path.starts_with("/echo/") {
        let value = &req.path[6..];
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/plain\r\n\r\n{}",
            value.len(),
            value
        )
    } else if req.path == "/user-agent" {
        if let Some(agent) = req.headers.get("user-agent") {
            format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/plain\r\n\r\n{}",
                agent.len(),
                agent
            )
        } else {
            "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
        }
    } else {
        "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
    };

    match stream.write(resp.as_bytes()) {
        Ok(n) => println!("Successfully wrote {} bytes to stream", n),
        Err(e) => eprintln!("Error writing to stream: {:?}", e),
    }

    stream
        .shutdown(Shutdown::Both)
        .expect("Error closing stream");
}

#[derive(Debug)]
#[allow(dead_code)]
struct Request {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: String,
}

fn get_request(stream: &mut TcpStream) -> Result<Request> {
    let mut buf = [0; 2048];

    let n = stream.read(&mut buf)?;
    println!("Successfully read {} bytes from stream", n);

    let raw_request = str::from_utf8(&buf[..n]).expect("Invalid http request");
    println!("\nRaw request:\n======\n{}\n======\n", raw_request);

    let mut data = raw_request.split("\r\n");

    let start_line = data.next().expect("No start line");

    let mut headers = HashMap::new();

    loop {
        let header = data.next().expect("request truncated");
        if header.is_empty() {
            break;
        }

        if let [k, v] = header.splitn(2, ": ").collect::<Vec<&str>>()[..] {
            headers.insert(k.to_lowercase(), v.to_string());
        } else {
            eprintln!("Error parsing header: {}", header);
        }
    }

    let body = data.next().expect("request truncated");

    let parts: Vec<&str> = start_line.split(" ").collect();

    let request = Request {
        method: String::from(parts[0]),
        path: String::from(parts[1]),
        headers,
        body: String::from(body),
    };
    println!("Request: {:#?}", request);

    return Ok(request);
}
