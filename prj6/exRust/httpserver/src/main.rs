mod simple_web_server;
mod nonblocking_web_server;
mod multiplex_web_server;
mod sync_web_server;
fn main() {
    // simple_web_server::main_simple_server();
    // nonblocking_web_server::main_nonblocking_server();
    // multiplex_web_server::main_multiplex_server();
    sync_web_server::main_sync_server();
}
