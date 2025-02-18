unsafe extern "C" {
    // System
    pub fn env_print(s: *const u8, len: u32);  // Print string
    pub fn env_key_pressed() -> i32;  // Returns keycode or -1 if no key
    pub fn env_now() -> u64;  // Get current timestamp
    
    // Network
    pub fn env_net_setup() -> i32;
    pub fn env_net_teardown() -> i32;
    pub fn env_net_rx() -> i32;

    // pub fn env_max_sockets() -> i32;
    // pub fn env_used_sockets() -> i32;

    pub fn env_net_socket_new() -> i32;
    pub fn env_net_socket_free(socket: i32) -> i32;

    pub fn env_net_socket_connect(socket: i32, addr: u32, port: u32) -> i32;
    pub fn env_net_socket_connect_poll(socket: i32) -> i32;

    pub fn env_net_socket_bind(socket: i32, addr: u32, port: u32) -> i32;
    pub fn env_net_socket_listen(socket: i32, backlog: u32) -> i32;

    pub fn env_net_socket_accept(socket: i32) -> i32;
    pub fn env_net_socket_accept_poll(socket: i32) -> i32;
    pub fn env_net_socket_read(socket: i32, buf: *const u8, len: u32) -> i32;
    pub fn env_net_socket_write(socket: i32, buf: *const u8, len: u32) -> i32;
    pub fn env_net_socket_write_poll(socket: i32) -> i32;
}