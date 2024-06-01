use epoll::{ControlOptions::*, Event, Events};
use std::collections::VecDeque;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::fd::AsRawFd;
use std::sync::Mutex;
use std::{cell::RefCell, collections::HashMap, os::fd::RawFd, sync::Arc};

trait Future {
    type Output;

    fn poll(&mut self, waker: Waker) -> Option<Self::Output>;
}
struct Waker(Arc<dyn Fn() + Send + Sync>);

impl Waker {
    fn wake(&self) {
        (self.0)()
    }
}
static SCHEDULER: Scheduler = Scheduler {
    runnable: Mutex::new(VecDeque::new()),
};

thread_local! {
    static REACTOR: Reactor = Reactor::new();
}
struct Reactor {
    epoll: RawFd,
    tasks: RefCell<HashMap<RawFd, Waker>>,
}

impl Reactor {
    pub fn new() -> Reactor {
        Reactor {
            epoll: epoll::create(false).unwrap(),
            tasks: RefCell::new(HashMap::new()),
        }
    }
    // Add a file descriptor with read and write interest.
    //
    // `waker` will be called when the descriptor becomes ready.
    pub fn add(&self, fd: RawFd, waker: Waker) {
        let event = epoll::Event::new(Events::EPOLLIN | Events::EPOLLOUT, fd as u64);
        epoll::ctl(self.epoll, EPOLL_CTL_ADD, fd, event).unwrap();
        self.tasks.borrow_mut().insert(fd, waker);
    }
    // Remove the given descriptor from epoll.
    //
    // It will no longer receive any notifications.
    pub fn remove(&self, fd: RawFd) {
        self.tasks.borrow_mut().remove(&fd);
    }
    // Drive tasks forward, blocking forever until an event arrives.
    pub fn wait(&self) {
        let mut events = [Event::new(Events::empty(), 0); 1024];
        let timeout = -1; // forever
        let num_events = epoll::wait(self.epoll, timeout, &mut events).unwrap();

        for event in &events[..num_events] {
            let fd = event.data as i32;

            // wake the task
            if let Some(waker) = self.tasks.borrow().get(&fd) {
                waker.wake();
            }
        }
    }
}

type SharedTask = Arc<Mutex<dyn Future<Output = ()> + Send>>;

#[derive(Default)]
struct Scheduler {
    runnable: Mutex<VecDeque<SharedTask>>,
}

impl Scheduler {
    pub fn spawn(&self, task: impl Future<Output = ()> + Send + 'static) {
        self.runnable
            .lock()
            .unwrap()
            .push_back(Arc::new(Mutex::new(task)));
    }
}
impl Scheduler {
    fn run(&self) {
        loop {
            loop {
                // pop a runnable task off the queue
                let Some(task) = self.runnable.lock().unwrap().pop_front() else {
                    break;
                };
                let t2 = task.clone();

                // create a waker that pushes the task back on
                let wake = Arc::new(move || {
                    SCHEDULER.runnable.lock().unwrap().push_back(t2.clone());
                });

                // poll the task
                task.try_lock().unwrap().poll(Waker(wake));
            }
            // if there are no runnable tasks, block on epoll until something becomes ready
            REACTOR.with(|reactor| reactor.wait()); // 👈
        }
    }
}
// =================================above is the runtime we create============================================
#[allow(unused)]
pub fn main_sync_server() {
    SCHEDULER.spawn(Main::Start);
    SCHEDULER.run();
}

enum Main {
    Start,
    Accept { listener: TcpListener },
}
struct Handler {
    connection: TcpStream,
    state: HandlerState,
}

enum HandlerState {
    Start,
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

impl Future for Handler {
    type Output = ();

    fn poll(&mut self, waker: Waker) -> Option<Self::Output> {
        if let HandlerState::Start = self.state {
            // start by registering our connection for notifications
            REACTOR.with(|reactor| {
                reactor.add(self.connection.as_raw_fd(), waker);
            });

            self.state = HandlerState::Read {
                request: [0u8; 1024],
                read: 0,
            };
            // // I deliberately add this to show poll change state step by step
            // return Some(());
        }

        // read the request
        if let HandlerState::Read { request, read } = &mut self.state {
            loop {
                match self.connection.read(&mut request[*read..]) {
                    Ok(0) => {
                        println!("client disconnected unexpectedly");
                        return Some(());
                    }
                    Ok(n) => *read += n,
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => return None, // 👈
                    Err(e) => panic!("{e}"),
                }

                // did we reach the end of the request?
                let read = *read;
                if read >= 4 && &request[read - 4..read] == b"\r\n\r\n" {
                    break;
                }
            }

            // we're done, print the request
            let request = String::from_utf8_lossy(&request[..*read]);
            println!("{}", request);

            // and move into the write state
            // move into the write state
            let response = concat!(
                "HTTP/1.1 200 OK\r\n",
                "Content-Length: 13\n",
                "Connection: close\r\n\r\n",
                "Hello world!\n"
            );

            self.state = HandlerState::Write {
                response: response.as_bytes(),
                written: 0,
            };
        }

        // write the response
        if let HandlerState::Write { response, written } = &mut self.state {
            // self.connection.write...
            loop {
                match self.connection.write(&response[*written..]) {
                    Ok(0) => {
                        println!("client disconnected unexpectedly");
                        return Some(());
                    }
                    Ok(n) => {
                        *written += n;
                    }
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                        return None;
                    }
                    Err(e) => panic!("{e}"),
                }
                if *written == response.len() {
                    break;
                }
            }
            // successfully wrote the response, try flushing next
            self.state = HandlerState::Flush;
        }

        // flush the response
        if let HandlerState::Flush = self.state {
            match self.connection.flush() {
                Ok(_) => {}
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => return None, // 👈
                Err(e) => panic!("{e}"),
            }
        }
        REACTOR.with(|reactor| {
            reactor.remove(self.connection.as_raw_fd());
        });
    
        Some(())
    }
}

impl Future for Main {
    type Output = ();

    fn poll(&mut self, waker: Waker) -> Option<Self::Output> {
        if let Main::Start = self {
            let listener = TcpListener::bind("localhost:3000").unwrap();
            listener.set_nonblocking(true).unwrap();
            REACTOR.with(|reactor| {
                reactor.add(listener.as_raw_fd(), waker);
            });
            *self = Main::Accept { listener };
        }
        if let Main::Accept { listener } = self {
            match listener.accept() {
                Ok((connection, _)) => {
                    connection.set_nonblocking(true).unwrap(); // 👈
                    SCHEDULER.spawn(Handler {
                        connection,
                        state: HandlerState::Start,
                    });
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => return None,
                Err(e) => panic!("{e}"),
            }
        }

        None
    }
}
