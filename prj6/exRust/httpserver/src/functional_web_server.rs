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
    fn chain<F, T>(self, transition: F) -> Chain<Self, F, T>
    where
        F: FnOnce(Self::Output) -> T,
        T: Future,
        Self: Sized,
    {
        Chain::First {
            future1: self,
            transition: Some(transition),
        }
    }
}

fn poll_fn<F, T>(f: F) -> impl Future<Output = T>
where
    F: FnMut(Waker) -> Option<T>,
{
    struct FromFn<F>(F);

    impl<F, T> Future for FromFn<F>
    where
        F: FnMut(Waker) -> Option<T>,
    {
        type Output = T;

        fn poll(&mut self, waker: Waker) -> Option<Self::Output> {
            (self.0)(waker)
        }
    }

    FromFn(f)
}

enum Chain<T1, F, T2> {
    First { future1: T1, transition: Option<F> }, // 👈
    Second { future2: T2 },
}

impl<T1, F, T2> Future for Chain<T1, F, T2>
where
    T1: Future,
    F: FnOnce(T1::Output) -> T2,
    T2: Future,
{
    type Output = T2::Output;

    fn poll(&mut self, waker: Waker) -> Option<Self::Output> {
        if let Chain::First {
            future1,
            transition,
        } = self
        {
            // poll the first future
            match future1.poll(waker.clone()) {
                Some(value) => {
                    // first future is done, transition into the second
                    let future2 = (transition.take().unwrap())(value); // 👈
                    *self = Chain::Second { future2 };
                }
                // first future is not ready, return
                None => return None,
            }
        }

        if let Chain::Second { future2 } = self {
            // first future is already done, poll the second
            return future2.poll(waker); // 👈
        }

        None
    }
}

#[derive(Clone)]
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
pub fn main_functional_server() {
    SCHEDULER.spawn(listen());
    SCHEDULER.run();
}

fn listen() -> impl Future<Output = ()> {
    poll_fn(|waker| {
        let listener = TcpListener::bind("localhost:3000").unwrap();

        listener.set_nonblocking(true).unwrap();

        REACTOR.with(|reactor| {
            reactor.add(listener.as_raw_fd(), waker);
        });

        Some(listener)
    })
    .chain(|listener| {
        poll_fn(move |_| match listener.accept() {
            Ok((connection, _)) => {
                connection.set_nonblocking(true).unwrap();
                SCHEDULER.spawn(handle(connection));

                None
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => None,
            Err(e) => panic!("{e}"),
        })
    })
}

fn handle(connection: TcpStream) -> impl Future<Output = ()> {
    let mut connection = Some(connection);
    poll_fn(move |waker| {
        REACTOR.with(|reactor| {
            reactor.add(connection.as_ref().unwrap().as_raw_fd(), waker);
        });

        Some(connection.take())
    })
    .chain(move |mut connection| {
        let mut request = [0u8; 1024];
        let mut read = 0;

        poll_fn(move |_| {
            loop {
                // try reading from the stream
                match connection.as_mut().unwrap().read(&mut request) {
                    Ok(0) => {
                        println!("client disconnected unexpectedly");
                        return Some(connection.take());
                    }
                    Ok(n) => read += n,
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => return None,
                    Err(e) => panic!("{e}"),
                }

                // did we reach the end of the request?
                let read = read;
                if read >= 4 && &request[read - 4..read] == b"\r\n\r\n" {
                    break;
                }
            }

            // we're done, print the request
            let request = String::from_utf8_lossy(&request[..read]);
            println!("{request}");

            Some(connection.take())
        })
    })
    .chain(move |mut connection| {
        let response = concat!(
            "HTTP/1.1 200 OK\r\n",
            "Content-Length: 13\n",
            "Connection: close\r\n\r\n",
            "Hello world!\n"
        );
        let mut written = 0;

        poll_fn(move |_| {
            loop {
                match connection
                    .as_mut()
                    .unwrap()
                    .write(&response.as_bytes()[written..])
                {
                    Ok(0) => {
                        println!("client disconnected unexpectedly");
                        return Some(connection.take());
                    }
                    Ok(n) => written += n,
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => return None,
                    Err(e) => panic!("{e}"),
                }

                // did we write the whole response yet?
                if written == response.len() {
                    break;
                }
            }

            Some(connection.take())
        })
    })
    .chain(move |mut connection| {
        poll_fn(move |_| {
            match connection.as_mut().unwrap().flush() {
                Ok(_) => {}
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    return None;
                }
                Err(e) => panic!("{e}"),
            };

            REACTOR.with(|reactor| {
                reactor.remove(connection.as_ref().unwrap().as_raw_fd());
            });

            Some(())
        })
    })
}
