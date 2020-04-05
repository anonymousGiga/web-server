use mylib::ThreadPool;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::{thread, time};

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();
    let get = b"GET / HTTP/1.1\r\n";
    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "main.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();
    let response = format!("{}{}", status_line, contents);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();

    let te = time::Duration::from_millis(10000);
    thread::sleep(te); //睡眠一段时间，模拟处理时间很长
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    let pool = ThreadPool::new(4);

    for stream in listener.incoming().take(4) {
        let stream = stream.unwrap();
        //thread pool
        pool.execute(|| handle_client(stream));
    }

    Ok(())
}
