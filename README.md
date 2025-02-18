# NeoBoot

Welcome to the NeoBoot source code monorepo.

## Getting Started

To get started, make sure you have Docker or Podman installed and running. Then build U-Boot using `make u-boot`, this will clone down the U-Boot source code and place it in `vendor/u-boot`, with the NeoBoot patches applied. It will then build the bootloader for the default arch(x86_64), and the wasm spl, and finally boot them using QEMU using Docker. The resulting binaries are placed in `dist/(arch)`.

If you want to make any changes to the U-Boot source code, it can be helpful to have the compile-commands.json file, which will be used by clangd. To generate this file run `make u-boot-ide`. After making some changes and building it, then build system will export the patch set and place it in the source tree, ensuring deterministic and declarative builds.

To speed up builds using ccache, you can start Redis with `u-boot-redis-up`, the build system will automatically start using it, if your environment is set up correctly.


## TODO:
- convert tcp to netconn for dual udp, tcp support
- create dns function on top of rustdns
- support embedded_io_async and embedded_nal_async traits
- integrate with reqwless(reqwest) and picoserve(axum)
- websockets using tungstenite
- autobahn test suite
- enhance performance using mio
- create cli using shellfish with following commands:
    - stats
    - chain
- use binman to bundle wasm and u-boot and support booting from integrated wasm payload
- try on real hardware
- ???
- profit




Invalid Opcode (Undefined Opcode)
EIP: 0010:[<020c02c6>] EFLAGS: 00000206
Original EIP :[<c30a52c6>]
EAX: ffffffba EBX: 00000000 ECX: 020c02c4 EDX: d5c9fb31
ESI: 3eff77c0 EDI: 2ef09988 EBP: 2eed8a48 ESP: 2eed8a1c
 DS: 0018 ES: 0018 FS: 0020 GS: 0018 SS: 0018
CR0: 00000033 CR2: 00000000 CR3: 00000000 CR4: 00000000
DR0: 00000000 DR1: 00000000 DR2: 00000000 DR3: 00000000
DR6: ffff0ff0 DR7: 00000400
Stack:
    0x2eed8a5c : 0x2f039b2c
    0x2eed8a58 : 0x00000000
    0x2eed8a54 : 0x00000000
    0x2eed8a50 : 0x000ffde0
    0x2eed8a4c : 0x2ef19a28
    0x2eed8a48 : 0x00000000
    0x2eed8a44 : 0x2f048334
    0x2eed8a40 : 0x2f048334
    0x2eed8a3c : 0x00000000
    0x2eed8a38 : 0x00000000
    0x2eed8a34 : 0x00000000
    0x2eed8a30 : 0x2f049ce4
    0x2eed8a2c : 0x3efa6a66
    0x2eed8a28 : 0x2ef09988
    0x2eed8a24 : 0x3ef3bb57
    0x2eed8a20 : 0x2ef047d0
--->0x2eed8a1c : 0x3ef3bb72
    0x2eed8a18 : 0x00000206
    0x2eed8a14 : 0x00000010
    0x2eed8a10 : 0x020c02c6

You really shined some light on the situation

Can we have stack traces
We have stack traces at home
Stack traces at home:



/* Current network interface */
struct netif *current_netif = NULL;

/*
 * Maximum number of network sockets that can be tracked simultaneously.
 */
#define MAX_NETWORK_SOCKETS 64

/* Timeout for a connection attempt in milliseconds */
#define CONNECTION_TIMEOUT_MS 4000

/*
 * Structure to track the state of a network socket.
 */
typedef struct NetworkSocket
{
    struct tcp_pcb *pcb;            // TCP protocol control block associated with the event
    bool is_connected;              // Flag indicating whether the socket is connected
    uint64_t connection_start_time; // Time when the connection was established
    err_t err;                      // Error code (if any) for the socket

    uint32_t bytes_sent;       // Number of sent bytes
    uint32_t bytes_sent_acked; // Number of sent bytes acknowledged

    struct pbuf *recv_buffer; // Receive buffer
    uint32_t recv_bytes;      // Number of received bytes

    uint64_t listening_pcb; // Listening protocol control block
    bool is_claimed;        // Flag indicating whether the socket is claimed
} NetworkSocket;


typedef struct {
    NetworkSocket connections[MAX_NETWORK_SOCKETS];
    uint64_t active_connection_bitfield;  
} net_context_t;

// network_setup
// network_teardown

// socket_new
// socket_get
// socket_free

// socket_connect
// socket_bind
// socket_listen
// socket_accept
// socket_accept_claim
// socket_poll

// socket_read
// socket_write
// socket_write_ack



/*
 * Array to store active network sockets.
 */
static NetworkSocket network_sockets[MAX_NETWORK_SOCKETS];


static void tcp_err_callback(void *arg, err_t err)
{
    printf("tcp_err_callback: %d\n", err);
    for (int i = 0; i < MAX_NETWORK_SOCKETS; i++)
    {
        if (network_sockets[i].pcb == (struct tcp_pcb *)arg)
        {
            network_sockets[i].err = err;
        }
    }
}

static err_t tcp_sent_callback(void *arg, struct tcp_pcb *pcb, u16_t len)
{
    for (int i = 0; i < MAX_NETWORK_SOCKETS; i++)
    {
        if (network_sockets[i].pcb != pcb)
            continue;

        network_sockets[i].bytes_sent_acked += len;
        break;
    }

    return ERR_OK;
}

static err_t tcp_recv_callback(void *arg, struct tcp_pcb *pcb, struct pbuf *p, err_t err)
{
    printf("tcp_recv_callback: %d\n", err);
    if (p != NULL)
    {
        printf("tcp_recv_callback: p->len=%d\n", p->len);
    }

    if (err != ERR_OK)
    {
        printf("tcp_recv_callback error: %d\n", err);
        return err;
    }

    for (int i = 0; i < MAX_NETWORK_SOCKETS; i++)
    {
        if (network_sockets[i].pcb != (struct tcp_pcb *)arg)
            continue;

        if (network_sockets[i].recv_buffer != NULL)
        {
            pbuf_cat(network_sockets[i].recv_buffer, p);
        }
        else
        {
            network_sockets[i].recv_buffer = p;
        }

        break;
    }

    return ERR_OK;
}

static void teardown_network(void)
{
    if (current_netif != NULL)
    {
        net_lwip_remove_netif(current_netif);
        current_netif = NULL;
    }
}

static err_t check_socket_exists(uint64_t socket)
{
    if (socket == 0)
    {
        return ERR_ARG;
    }

    for (int i = 0; i < MAX_NETWORK_SOCKETS; i++)
    {
        if (network_sockets[i].pcb != (struct tcp_pcb *)socket)
            continue;

        if (network_sockets[i].err != ERR_OK)
        {
            return network_sockets[i].err;
        }

        return ERR_OK;
    }

    return ERR_VAL;
}

static err_t get_socket_connection_error(uint64_t socket)
{
    if (socket == 0)
    {
        return ERR_ARG;
    }

    for (int i = 0; i < MAX_NETWORK_SOCKETS; i++)
    {
        if (network_sockets[i].pcb != (struct tcp_pcb *)socket)
            continue;

        if (network_sockets[i].is_connected == false)
        {
            return ERR_CONN;
        }

        if (network_sockets[i].err != ERR_OK)
        {
            return network_sockets[i].err;
        }

        return ERR_OK;
    }

    return ERR_VAL;
}

/*
 * Setup the network environment and initialize the network interface.
 * This function ensures that the network is properly configured before use.
 */
m3ApiRawFunction(env_setup_network)
{
    m3ApiReturnType(int32_t); // Declare the return type as 32-bit integer
    int32_t ret = 0;          // Default return value indicating success

    /* Reset the network state */
    teardown_network();

    /* Set the current Ethernet device */
    eth_set_current();

    /* Retrieve the current Ethernet device */
    struct udevice *udev = eth_get_dev();
    if (!udev)
    {
        /* If no valid device is found, return an error */
        m3ApiReturn(-1);
    }

    /* Create a new network interface for the device */
    current_netif = net_lwip_new_netif(udev);
    if (current_netif == NULL)
    {
        /* If network interface creation fails, return an error */
        m3ApiReturn(-1);
    }

    /* Return success */
    m3ApiReturn(ret);
}

/*
 * Teardown the network environment and close all network sockets.
 *
 * This function iterates through all network sockets and closes them if they
 * are not already closed. It then calls the teardown_network function to
 * clean up the network interface.
 */
m3ApiRawFunction(env_teardown_network)
{
    for (int i = 0; i < MAX_NETWORK_SOCKETS; i++)
    {
        if (network_sockets[i].pcb == NULL)
            continue;

        if (network_sockets[i].pcb->state != CLOSED)
            tcp_abort(network_sockets[i].pcb);

        if (network_sockets[i].recv_buffer != NULL)
            pbuf_free(network_sockets[i].recv_buffer);

        network_sockets[i] = (NetworkSocket){0};
    }

    teardown_network();
    m3ApiSuccess();
}

/*
 * Process incoming Ethernet frames.
 *
 * This function retrieves the current network interface and processes incoming
 * Ethernet frames. It returns the result of the net_lwip_rx function, which
 * handles the actual reception of frames.
 */
m3ApiRawFunction(env_lwip_rx)
{
    m3ApiReturnType(int32_t); // Declare the return type as 32-bit integer

    if (current_netif == NULL)
        m3ApiReturn(-1);

    int result = net_lwip_rx(eth_get_dev(), current_netif);
    sys_check_timeouts();
    m3ApiReturn(result);
}

/*
 * Create a new TCP socket.
 * This function initializes a new TCP protocol control block (PCB) and returns
 * a handle to the newly created socket. If the socket cannot be created, a
 * zero value is returned.
 */
m3ApiRawFunction(env_socket_create)
{
    m3ApiReturnType(uint64_t); // Declare the return type as 64-bit unsigned integer

    if (current_netif == NULL)
        m3ApiReturn(0);

    /* Attempt to create a new TCP PCB */
    struct tcp_pcb *socket = tcp_new();

    /* Find an available slot in the network I/O events array */
    bool slot_found = false;
    for (int i = 0; i < MAX_NETWORK_SOCKETS; i++)
    {
        if (network_sockets[i].pcb != NULL)
            continue;

        /* Ensure the socket is not already being tracked */
        if (network_sockets[i].pcb == (struct tcp_pcb *)socket)
            m3ApiReturn(0);

        /* Assign the socket to the slot */
        network_sockets[i].pcb = (struct tcp_pcb *)socket;
        network_sockets[i].is_connected = false;
        network_sockets[i].connection_start_time = get_timer(0);
        network_sockets[i].err = ERR_OK;
        slot_found = true;
        break;
    }

    /* Return an error if no slot is available */
    if (!slot_found)
    {
        m3ApiReturn(0);
    }

    if (socket != NULL)
    {
        tcp_arg(socket, socket);
        tcp_err(socket, tcp_err_callback);
        tcp_sent(socket, tcp_sent_callback);
        tcp_recv(socket, tcp_recv_callback);
    }

    /* Return the handle to the newly created socket */
    m3ApiReturn((uint64_t)socket);
}

/*
 * Close a TCP socket.
 * This function attempts to close the specified TCP socket. If the socket
 * cannot be closed, an error code is returned. On success, zero is returned.
 */
m3ApiRawFunction(env_socket_close)
{
    m3ApiReturnType(int32_t); // Declare the return type as 32-bit integer
    m3ApiGetArg(uint64_t,
                socket); // Retrieve the socket handle from the arguments

    /* Iterate through the network sockets array to find the matching socket */
    for (int i = 0; i < MAX_NETWORK_SOCKETS; i++)
    {
        if (network_sockets[i].pcb != (struct tcp_pcb *)socket)
            continue;

        if (network_sockets[i].recv_buffer != NULL)
        {
            pbuf_free(network_sockets[i].recv_buffer);
            network_sockets[i].recv_buffer = NULL;
        }

        /* Clear the socket state */
        network_sockets[i] = (NetworkSocket){0};
        break;
    }

    /* Attempt to close the specified socket */
    err_t err = tcp_close((struct tcp_pcb *)socket);

    /* Verify if the socket was successfully closed */
    if (err != ERR_OK)
    {
        /* Return an error code if the socket could not be closed */
        m3ApiReturn(err);
    }

    /* Return success */
    m3ApiReturn(0);
}

/*
 * Callback function invoked when a TCP connection attempt completes.
 */
static err_t tcp_connect_callback(void *arg, struct tcp_pcb *pcb, err_t error)
{
    printf("tcp_connect_callback: %d\n", error);
    /* Iterate through the network sockets array to find the matching PCB */
    for (int i = 0; i < MAX_NETWORK_SOCKETS; i++)
    {
        if (network_sockets[i].pcb == arg)
        {
            /* Update the sockets with the result of the connection attempt */
            network_sockets[i].is_connected = (error == ERR_OK);
            network_sockets[i].err = error;
            break;
        }
    }

    return ERR_OK;
}

/*
 * Connect a TCP socket to a remote address and port.
 * This function initiates a TCP connection to the specified address and port.
 * The result of the connection attempt can be checked using env_socket_poll.
 */
m3ApiRawFunction(env_socket_connect)
{
    m3ApiReturnType(int32_t); // Declare the return type as 32-bit integer
    m3ApiGetArg(uint64_t, socket);
    m3ApiGetArg(uint32_t,
                addr);           //  Retrieve the IP address from big endian to network byte order
    m3ApiGetArg(uint32_t, port); // Retrieve the port number from the arguments

    err_t error = check_socket_exists(socket);
    if (error != ERR_OK)
    {
        m3ApiReturn(error);
    }

    /* Attempt to establish the TCP connection */
    ip_addr_t ip_addr = {.addr = addr};
    error = tcp_connect((struct tcp_pcb *)socket, &ip_addr, port, tcp_connect_callback);

    /* Return an error if the connection attempt fails immediately */
    if (error != ERR_OK)
    {
        m3ApiReturn(error);
    }

    /* Return success to indicate the connection attempt is in progress */
    m3ApiReturn(0);
}

/*
 * Check the status of a TCP socket.
 */
m3ApiRawFunction(env_socket_check_connection)
{
    m3ApiReturnType(int32_t); // Declare the return type as 32-bit integer
    m3ApiGetArg(uint64_t,
                socket); // Retrieve the socket handle from the arguments

    /* Iterate through the network sockets array to find the matching socket */
    for (int i = 0; i < MAX_NETWORK_SOCKETS; i++)
    {
        if (network_sockets[i].pcb != (struct tcp_pcb *)socket)
            continue;

        /* Return the result if the connection attempt has completed */
        if (network_sockets[i].is_connected)
        {
            m3ApiReturn(ERR_OK);
        }

        /* Return the error code if the connection attempt failed */
        if (network_sockets[i].err != ERR_OK)
            m3ApiReturn(network_sockets[i].err);

        /* Return the error code if the connection attempt timed out */
        if (get_timer(0) - network_sockets[i].connection_start_time > CONNECTION_TIMEOUT_MS)
            m3ApiReturn(ERR_TIMEOUT);

        /* Return a special code to indicate the operation is still in progress */
        m3ApiReturn(ERR_INPROGRESS);
    }

    /* Return an error if the socket is not being tracked */
    m3ApiReturn(ERR_VAL);
}

/**
 * Returns the maximum number of network sockets supported.
 *
 * @return Maximum number of sockets (MAX_NETWORK_SOCKETS)
 */
m3ApiRawFunction(env_max_sockets)
{
    m3ApiReturnType(int32_t);
    m3ApiReturn(MAX_NETWORK_SOCKETS);
}

/**
 * Returns the current number of active network sockets.
 *
 * Counts sockets that have a valid PCB (Protocol Control Block).
 *
 * @return Number of active sockets
 */
m3ApiRawFunction(env_used_sockets)
{
    m3ApiReturnType(int32_t);

    int active_sockets = 0;
    for (int i = 0; i < MAX_NETWORK_SOCKETS; i++)
    {
        if (network_sockets[i].pcb != NULL)
        {
            active_sockets++;
        }
    }

    m3ApiReturn(active_sockets);
}

/*
 * Write data to a TCP socket.
 * This function writes data to a TCP socket and returns the number of bytes
 * written. If the socket is not connected or there is an error, it returns an
 * error code.
 */
m3ApiRawFunction(env_socket_write)
{
    m3ApiReturnType(int32_t);
    m3ApiGetArg(uint64_t, socket);
    m3ApiGetArgMem(void *, buf);
    m3ApiGetArg(uint32_t, len);
    m3ApiCheckMem(buf, len);

    struct tcp_pcb *pcb = (struct tcp_pcb *)socket;

    /* Check if socket has any errors */
    err_t err = get_socket_connection_error(socket);
    if (err != ERR_OK)
    {
        m3ApiReturn(err);
    }

    /* Check if there's enough send buffer space */
    uint16_t available_space = tcp_sndbuf(pcb);
    if (len > available_space)
    {
        m3ApiReturn(ERR_MEM);
    }

    /* Write data to socket */
    err = tcp_write(pcb, buf, len, TCP_WRITE_FLAG_COPY);
    if (err != ERR_OK)
    {
        m3ApiReturn(err);
    }

    /* Flush output buffer */
    err = tcp_output(pcb);
    if (err != ERR_OK)
    {
        m3ApiReturn(err);
    }

    /* Update bytes sent counter */
    for (int i = 0; i < MAX_NETWORK_SOCKETS; i++)
    {
        if (network_sockets[i].pcb == pcb)
        {
            network_sockets[i].bytes_sent += len;
            break;
        }
    }

    m3ApiReturn(ERR_OK);
}

/*
 * Check if all writes to a socket have been acknowledged.
 * This function checks if all data sent to a socket has been acknowledged by
 * the remote end. If all data has been acknowledged, it returns ERR_OK.
 * Otherwise, it returns ERR_WOULDBLOCK.
 */
m3ApiRawFunction(env_socket_all_writes_acked)
{
    m3ApiReturnType(int32_t);
    m3ApiGetArg(uint64_t, socket);
    struct tcp_pcb *pcb = (struct tcp_pcb *)socket;

    /* Check for socket errors first */
    err_t socket_err = get_socket_connection_error(socket);
    if (socket_err != ERR_OK)
    {
        m3ApiReturn(socket_err);
    }

    /* Find the socket in our tracking array */
    for (int i = 0; i < MAX_NETWORK_SOCKETS; i++)
    {
        if (network_sockets[i].pcb == pcb)
        {
            /* Check if all bytes have been acknowledged */
            if (network_sockets[i].bytes_sent == network_sockets[i].bytes_sent_acked)
            {
                m3ApiReturn(ERR_OK);
            }
            m3ApiReturn(ERR_WOULDBLOCK);
        }
    }

    /* Socket not found */
    m3ApiReturn(ERR_VAL);
}

/*
 * Read data from a TCP socket.
 * This function reads data from a TCP socket and returns the number of bytes
 * read. If no data is available, it returns ERR_WOULDBLOCK.
 */
m3ApiRawFunction(env_socket_read)
{
    m3ApiReturnType(int32_t);
    m3ApiGetArg(uint64_t, socket);
    m3ApiGetArgMem(void *, buf);
    m3ApiGetArg(uint32_t, len);

    /* Check if socket has any errors */
    err_t socket_err = get_socket_connection_error(socket);
    if (socket_err != ERR_OK)
    {
        m3ApiReturn(socket_err);
    }

    /* Find the socket in our tracking array */
    struct NetworkSocket *sock = NULL;
    for (int i = 0; i < MAX_NETWORK_SOCKETS; i++)
    {
        if (network_sockets[i].pcb == (struct tcp_pcb *)socket)
        {
            sock = &network_sockets[i];
            break;
        }
    }

    /* Return error if socket not found */
    if (!sock)
    {
        m3ApiReturn(ERR_VAL);
    }

    /* Check if there's data available to read */
    struct pbuf *p = sock->recv_buffer;
    if (!p)
    {
        m3ApiReturn(ERR_WOULDBLOCK);
    }

    /* Copy data from receive buffer to user buffer */
    uint16_t read_len = pbuf_copy_partial(p, buf, len, sock->recv_bytes);
    sock->recv_bytes += read_len;

    /* If we've read all data, free the buffer */
    if (sock->recv_bytes == p->tot_len)
    {
        pbuf_free(p);
        sock->recv_buffer = NULL;
        sock->recv_bytes = 0;
    }

    m3ApiReturn(read_len);
}

/*
 * Bind a TCP socket to a local address and port.
 * This function associates the socket with a specific local address and port.
 */
m3ApiRawFunction(env_socket_bind)
{
    m3ApiReturnType(int32_t);
    m3ApiGetArg(uint64_t, socket);
    m3ApiGetArg(uint32_t, addr);
    m3ApiGetArg(uint32_t, port);

    err_t error = check_socket_exists(socket);
    if (error != ERR_OK)
    {
        m3ApiReturn(error);
    }

    ip_addr_t ip_addr = {.addr = addr};
    err_t err = tcp_bind((struct tcp_pcb *)socket, &ip_addr, port);
    m3ApiReturn(err);
}

/*
 * Place a TCP socket in listening mode.
 * This function puts the socket in a state where it can accept incoming
 * connections. The backlog parameter specifies the maximum length of the queue
 * of pending connections. Returns the new listening socket.
 */
m3ApiRawFunction(env_socket_listen)
{
    m3ApiReturnType(uint64_t);
    m3ApiGetArg(uint64_t, socket);
    m3ApiGetArg(uint32_t, backlog);

    err_t error = check_socket_exists(socket);
    if (error != ERR_OK)
    {
        m3ApiReturn(error);
    }

    struct tcp_pcb *new_pcb = tcp_listen_with_backlog((struct tcp_pcb *)socket, backlog);
    if (new_pcb == NULL)
    {
        m3ApiReturn(0);
    }

    /* Update the PCB in our socket tracking array */
    for (int i = 0; i < MAX_NETWORK_SOCKETS; i++)
    {
        if (network_sockets[i].pcb == (struct tcp_pcb *)socket)
        {
            network_sockets[i].pcb = new_pcb;
            break;
        }
    }

    m3ApiReturn((uint64_t)new_pcb);
}

/*
 * Accept callback function for handling new TCP connections.
 */
static err_t tcp_accept_callback(void *arg, struct tcp_pcb *newpcb, err_t err)
{
    printf("tcp_accept_callback: %d, listening socket: %lld, client socket: %lld\n", err, (uint64_t)arg, (uint64_t)newpcb);


    if (err != ERR_OK || newpcb == NULL)
    {
        return ERR_VAL;
    }

    /* Find a free slot for the new connection */
    for (int i = 0; i < MAX_NETWORK_SOCKETS; i++)
    {
        if (network_sockets[i].pcb != NULL && network_sockets[i].pcb != arg)
            continue;

        /* Initialize the new connection */
        network_sockets[i].pcb = newpcb;
        network_sockets[i].is_connected = true;
        network_sockets[i].err = ERR_OK;
        network_sockets[i].listening_pcb = (uint64_t)arg;
        network_sockets[i].is_claimed = false;

        /* Set up callbacks for the new connection */
        tcp_arg(newpcb, newpcb);
        tcp_err(newpcb, tcp_err_callback);
        tcp_sent(newpcb, tcp_sent_callback);
        tcp_recv(newpcb, tcp_recv_callback);

        return ERR_OK;
    }

    return ERR_MEM;
}

/*
 * Accept a new incoming connection on a listening socket.
 * This function sets up the accept callback for the listening socket.
 *
 * @param socket	Socket handle to accept connections on
 * @return ERR_OK if successful, error code otherwise
 */
m3ApiRawFunction(env_socket_accept)
{
    m3ApiReturnType(int32_t);
    m3ApiGetArg(uint64_t, socket);

    /* Validate socket exists in tracking array */
    err_t error = check_socket_exists(socket);
    if (error != ERR_OK)
    {
        m3ApiReturn(error);
    }

    /* Socket must be in LISTEN state */
    struct tcp_pcb *pcb = (struct tcp_pcb *)socket;
    if (pcb->state != LISTEN)
        m3ApiReturn(ERR_CLSD);

    /* Set up accept callback */
    tcp_accept(pcb, tcp_accept_callback);
    m3ApiReturn(ERR_OK);
}

/*
 * Claim an accepted connection on a listening socket.
 *
 * @param socket	Socket handle to claim connection on
 * @return socket id if successful, 0 otherwise
 */
m3ApiRawFunction(env_socket_accept_claim_connection)
{
    m3ApiReturnType(uint64_t);
    m3ApiGetArg(uint64_t, socket);
    struct tcp_pcb *pcb = (struct tcp_pcb *)socket;

    /* Validate socket exists in tracking array */
    err_t error = check_socket_exists(socket);
    if (error != ERR_OK)
    {
        m3ApiReturn(0);
    }

    /* Find an unclaimed connection for this listening socket */
    for (int i = 0; i < MAX_NETWORK_SOCKETS; i++)
    {
        if (network_sockets[i].listening_pcb != (uint64_t)pcb)
            continue;

        if (!network_sockets[i].is_connected)
            continue;

        if (network_sockets[i].is_claimed)
            continue;

        /* Found an unclaimed connection - mark it as claimed */
        network_sockets[i].is_claimed = true;

        /* Return the new socket PCB */
        m3ApiReturn((uint64_t)network_sockets[i].pcb);
    }

    /* No unclaimed connections found */
    m3ApiReturn(0);
}
