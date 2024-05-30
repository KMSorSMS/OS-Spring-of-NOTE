use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::sleep;
// use std::time::Instant;

pub fn main_simple_server() {
    let listener = TcpListener::bind("localhost:3000").unwrap();
    loop {
        //计时
        // let start = Instant::now();
        let (connection, _) = listener.accept().unwrap();

        if let Err(e) = handle_connection(connection) {
            println!("failed to handle connection: {e}")
        }
        //计时
        // println!("time elapsed: {:?}", start.elapsed().as_secs_f32());
    }
}

fn handle_connection(mut connection: TcpStream) -> io::Result<()> {
    let mut read = 0;
    let mut request = [0u8; 1024];

    loop {
        // try reading from the stream
        let num_bytes = connection.read(&mut request[read..])?;

        // the client disconnected
        if num_bytes == 0 {
            println!("client disconnected unexpectedly");
            return Ok(());
        }

        // keep track of how many bytes we've read
        read += num_bytes;
        // have we reached the end of the request?
        if request.get(read - 4..read) == Some(b"\r\n\r\n") {
            break;
        }
    }
    let request = String::from_utf8_lossy(&request[..read]);
    println!("{request}");

    // "Hello World!" in HTTP
    let response = concat!(
        "HTTP/1.1 200 OK\r\n",
        "Content-Length: 26\n",
        "Connection: close\r\n\r\n",
        "Hello world!---from liamy\n"
    );

    let mut written = 0;
    //停5s
    sleep(std::time::Duration::from_secs(5));

    loop {
        // write the remaining response bytes
        let num_bytes = connection.write(response[written..].as_bytes())?;

        // the client disconnected
        if num_bytes == 0 {
            println!("client disconnected unexpectedly");
            return Ok(());
        }

        written += num_bytes;

        // have we written the whole response yet?
        if written == response.len() {
            break;
        }
    }
    // flush the response
    connection.flush()
}
