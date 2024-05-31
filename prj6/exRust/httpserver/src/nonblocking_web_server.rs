use std::io::{self, Read, Write};
use std::net::TcpListener;
// use std::time::Instant;

enum ConnectionState {
    Read {
        request: [u8; 1024],
        read: usize,
    },
    Write {
        response: &'static [u8],
        written: usize,
    },
    Flush,
}

pub fn main_nonblocking_server() {
    let listener = TcpListener::bind("localhost:3000").unwrap();
    listener.set_nonblocking(true).unwrap();
    let mut connections = Vec::new();
    loop {
        //计时
        // let start = Instant::now();
        match listener.accept() {
            Ok((connection, _)) => {
                connection.set_nonblocking(true).unwrap();
                let state = ConnectionState::Read {
                    // 👈
                    request: [0u8; 1024],
                    read: 0,
                };
                connections.push((connection, state));
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => panic!("{e}"),
        };

        let mut completed = Vec::new(); // 👈
        'next: for (i, (connection, state)) in connections.iter_mut().enumerate() {
            if let ConnectionState::Read { request, read } = state {
                loop {
                    // try reading from the stream
                    match connection.read(&mut request[*read..]) {
                        Ok(0) => {
                            println!("client disconnected unexpectedly");
                            completed.push(i);
                            continue 'next;
                        }
                        Ok(n) => {
                            // keep track of how many bytes we've read
                            *read += n
                        }
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                            // not ready yet, move on to the next connection
                            continue 'next; // 👈
                        }
                        Err(e) => panic!("{e}"),
                    }

                    // did we reach the end of the request?
                    if request.get(*read - 4..*read) == Some(b"\r\n\r\n") {
                        break;
                    }
                }

                // we're done, print the request
                let request = String::from_utf8_lossy(&request[..*read]);
                println!("{request}");

                // move into the write state
                let response = concat!(
                    "HTTP/1.1 200 OK\r\n",
                    "Content-Length: 13\n",
                    "Connection: close\r\n\r\n",
                    "Hello world!\n"
                );

                *state = ConnectionState::Write {
                    response: response.as_bytes(),
                    written: 0,
                };
            }
            if let ConnectionState::Write { response, written } = state {
                loop {
                    match connection.write(&response[*written..]) {
                        Ok(0) => {
                            println!("client disconnected unexpectedly");
                            completed.push(i);
                            continue 'next;
                        }
                        Ok(n) => {
                            *written += n;
                        }
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                            continue 'next;
                        }
                        Err(e) => panic!("{e}"),
                    }
                    if *written == response.len() {
                        break;
                    }
                }
                // successfully wrote the response, try flushing next
                *state = ConnectionState::Flush;
            }

            if let ConnectionState::Flush = state {
                match connection.flush() {
                    Ok(_) => {
                        completed.push(i); // 👈
                    },
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                        // not ready yet, move on to the next connection
                        continue 'next;
                    }
                    Err(e) => panic!("{e}"),
                }
            }
        }

        

        //计时
        // println!("time elapsed: {:?}", start.elapsed().as_secs_f32());
    }
}

// fn handle_connection(mut connection: TcpStream) -> io::Result<()> {
//     let mut read = 0;
//     let mut request = [0u8; 1024];

//     loop {
//         // try reading from the stream
//         let num_bytes = connection.read(&mut request[read..])?;

//         // the client disconnected
//         if num_bytes == 0 {
//             println!("client disconnected unexpectedly");
//             return Ok(());
//         }

//         // keep track of how many bytes we've read
//         read += num_bytes;
//         // have we reached the end of the request?
//         if request.get(read - 4..read) == Some(b"\r\n\r\n") {
//             break;
//         }
//     }
//     let request = String::from_utf8_lossy(&request[..read]);
//     println!("{request}");

//     // "Hello World!" in HTTP
//     let response = concat!(
//         "HTTP/1.1 200 OK\r\n",
//         "Content-Length: 26\n",
//         "Connection: close\r\n\r\n",
//         "Hello world!---from liamy\n"
//     );

//     let mut written = 0;
//     //停5s
//     sleep(std::time::Duration::from_secs(5));

//     loop {
//         // write the remaining response bytes
//         let num_bytes = connection.write(response[written..].as_bytes())?;

//         // the client disconnected
//         if num_bytes == 0 {
//             println!("client disconnected unexpectedly");
//             return Ok(());
//         }

//         written += num_bytes;

//         // have we written the whole response yet?
//         if written == response.len() {
//             break;
//         }
//     }
//     // flush the response
//     connection.flush()
// }
