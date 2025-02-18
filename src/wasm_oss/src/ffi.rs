unsafe extern "C" {
    // System
    pub fn env_now() -> u64;  // Get current timestamp
    pub fn env_print(s: *const u8, len: u32);  // Print string
    
    // Keyboard
    pub fn env_key_pressed() -> i32;  // Returns keycode or -1 if no key
    
    // Network
    pub fn env_setup_network() -> i32;
    pub fn env_teardown_network();
    pub fn env_lwip_rx() -> i32;

    pub fn env_max_sockets() -> i32;
    pub fn env_used_sockets() -> i32;

    pub fn env_socket_create() -> u64;
    pub fn env_socket_close(socket: u64) -> i32;

    pub fn env_socket_connect(socket: u64, addr: u32, port: u32) -> i32;
    pub fn env_socket_check_connection(socket: u64) -> i32;

    pub fn env_socket_read(socket: u64, buf: *const u8, len: u32) -> i32;
    pub fn env_socket_write(socket: u64, buf: *const u8, len: u32) -> i32;
    pub fn env_socket_all_writes_acked(socket: u64) -> i32;

    pub fn env_socket_accept(socket: u64) -> i32;
    pub fn env_socket_bind(socket: u64, addr: u32, port: u32) -> i32;
    pub fn env_socket_listen(socket: u64, backlog: u32) -> u64;

    pub fn env_socket_accept_claim_connection(socket: u64) -> u64;

}