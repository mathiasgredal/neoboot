unsafe extern "C" {
    // System
    pub fn env_print(s: *const u8, len: u32); // Print string
    pub fn env_key_pressed() -> i32; // Returns keycode or -1 if no key
    pub fn env_now() -> u64; // Get current timestamp
    pub fn env_malloc(size: u32) -> u64; // Allocate memory
    pub fn env_free(ptr: u64) -> i32; // Free memory
    pub fn env_memcpy(src: *const u8, dest: u64, len: u32) -> i32; // Copy memory
    pub fn env_set_wasm_chainload(src: u64, len: u32); // Set WASM chainload source

    // Network
    pub fn env_net_setup() -> i32;
    pub fn env_net_teardown() -> i32;
    pub fn env_net_rx() -> i32;
    // pub fn env_max_sockets() -> i32;
    // pub fn env_used_sockets() -> i32;

    // DNS
    pub fn env_net_dns_set_server(server_addr: u32);
    pub fn env_net_dns_lookup(hostname: *const u8, len: u32) -> i32;
    pub fn env_net_dns_lookup_poll() -> i32;
    pub fn env_net_dns_lookup_result() -> u32;

    // Socket
    pub fn env_net_socket_new_tcp() -> i32;
    pub fn env_net_socket_new_udp() -> i32;
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
