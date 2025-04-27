# NeoBoot

Welcome to the NeoBoot source code monorepo.

## Getting Started

To get started, make sure you have Docker or Podman installed and running. Then build U-Boot using `make u-boot`, this will clone down the U-Boot source code and place it in `vendor/u-boot`, with the NeoBoot patches applied. It will then build the bootloader for the default arch(x86_64), and the wasm payload, and finally boot them using QEMU using Docker. The resulting binaries are placed in `dist/(arch)`.

If you want to make any changes to the U-Boot source code, it can be helpful to have the compile-commands.json file, which will be used by clangd. To generate this file run `make u-boot-ide`. After making some changes and building it, then build system will export the patch set and place it in the source tree, ensuring deterministic and declarative builds.

To speed up builds using ccache, you can start Redis with `u-boot-redis-up`, the build system will automatically start using it, if your environment is set up correctly.

## Wasm Bootloader
Designed like Novel Netware



## TODO:
- check that env_net_setup has been called
- support embedded_io_async and embedded_nal_async traits [DONE]
- integrate with reqwless(reqwest alternative)[DONE] and picoserve(axum alternative)
- websockets using tungstenite
- autobahn test suite
- use flatbuffers to create an rpc protocol
- make a python proxy client that communicates with the rpc server
- make a pytest test suite using the proxy client
- enhance performance using mio
- create cli using shellfish with following commands:
    - stats
    - chain
- use binman to bundle wasm and u-boot and support booting from integrated wasm payload
- try on real hardware
- ???
- profit

## Threat Model
NeoBoot is designed to be both secure and easy to use. Due to the restricted environment of the bootloader, there are some things that limits the design choices, e.g. we don't have a trusted time source, we might not have high entropy randomness, etc.

There are two primary security operating modes:
- Standard
- Hardened

### Standard Mode
In this mode, all encryption keys are managed by the server, which means that if the server is compromised, the attacker has access to the infrastructure. Otherwise, the standard mode operates like the hardened mode.

### Hardened Mode
In this mode, the client manages the encryption keys, based on the X.509 certificate system. The following is the design of the cryptographic system:
1. The client generates a root certificate, consisting of a public key and a private key.
2. The client generates 2 leaf certificates, which will be used for the primary operations:
    - One certificate will be given to the server, and will be used to sign any data sent to the bootloader.
    - The other certificate will be used by the client to sign any data sent to the server. This includes commands, images, updates, etc.
3. The bootloader will verify the validity of the signature of both the server and the client leaf certificates.
4. When the client wants to send a command to the bootloader, it will recieve a request client nonce, which it will attach to the command payload. It will the sign this payload and send it to the server, which will also sign it, and finally it will send it to the bootloader.
5. The bootloader will verify the validity of the signature of the client, server and the request nonce.

This system ensures that we cannot have replay attacks, and that the server cannot impersonate the client. Also there is not single point of failure, as the server can be compromised, but will not have the authority to sign any payload by itself. If the client is compromised, we can revoke the compromised certificate on the server, and the server will refuse to sign any payloads from the compromised client.

### Payload header format
The payload header is a JWT object, which is signed by the server, and contains a signature of the payload from the client.
```json5
{
    "rpc_id": "RPC_ID",
    "response_type": "RESPONSE_TYPE", // Can either be standard, stream, or out-of-band
    // The payload contains the stream hash and size if response_type is stream
    "payload": "PAYLOAD",
    "hash": "PAYLOAD_HASH",
    "nonce": "NONCE", // Does not exist if response_type is out-of-band
    "client_payload_signature": "CLIENT_SIGNATURE",
    "certificate_chain": ["INTERMEDIATE_CERTIFICATE", "ROOT_CERTIFICATE"],
    "leaf_certificates":  {
        "client": "CLIENT_CERTIFICATE",
        "server": "SERVER_CERTIFICATE"
    },
    // Certificate roles are the roles of the leaf certificates in the certificate chain
    // These must be signed by the outermost intermediate certificate, which created the client and server certificates
    "certificate_roles": {
        "client": "CLIENT_CERTIFICATE_FINGERPRINT",
        "server": "SERVER_CERTIFICATE_FINGERPRINT"
    }
}
```

### Verification
The client will verify the payload using the following steps:
1. Verify the certificate chain
2. Verify the certificate roles with INTERMEDIATE_CERTIFICATE
3. Verify the leaf certificates with INTERMEDIATE_CERTIFICATE
4. Verify the nonce with EXPECTED_NONCE, if response_type is standard or stream
5. Verify the client payload signature with CLIENT_CERTIFICATE
6. Verify the entire payload header using the SERVER_CERTIFICATE
7. Verify the payload hash with SHA256
8. Verify the stream hash with SHA256 if RESPONSE_TYPE is stream

### Communication protocol
The above payload model, can be used for communication over many different mediums, due to the flexibility of the NeoBoot bootloader. We have the following paradigms of communication:
- RPC:
    - Any communication involving the network, which must be signed by both a client and a server
        - Bootloader makes a request to the server, with a generated nonce
        - Server sends a response to the bootloader, with a client-signed payload (e.g. an image) wrapped in a server-signed payload header including the nonce from the bootloader
- Out-of-band
    - Out-of-band refers to when we sign a blob of data using the client certificate, and make it available to the bootloader through some means(e.g. a file on the filesystem, and update file, etc.)

### Limitations
- Physical attacks are not considered in this threat model, e.g. if you can modify the bootloader, you can just change the root public key and server hostname. Integrity must be ensured by UEFI Secure Boot.
- If both client and server certificates are compromised, then there is not way to revoke access without updating the bootloader
- If the above happens, then we can still salvage the situation as long as we control the dns server, as we can point it to a trusted server with a different server certificate, and update all the clients to explicitly ignore the compromised server certificate
- If either the root certificate is compromised or the dns, server and client certificates are compromised, then you are SOL


## Logbook

### 2025-02-20
My TCP bindings weren't slow, it was just that qemu's e1000 driver was slow. Using virtio-net instead of e1000 fixed the issue.
I just wasted the entire day on this.

### 2025-02-21
In addition to u-boot, i will also need to support iPXE as a bootloader runtime.
Look into using release-please to manage releases.
Implement all traits required to use hickory resolver
Use hyper to implement http client

### 2025-02-23
Due to the use of tokio inside both hickory and hyper, it doesn't seem possible to these libraries.
Since there is no good recursive dns resolver in rust, i will just allow calls to the u-boots lwip resolver.
We need better controls for DHCP and static IP configuration and network error handling.
Reqwless supports TLS, but it doesn't support verifying certificates. Using rusttls with rusttls-rustcrypto would work, but requires a lot of work.
We could also ignore certificate verification, and do our own certificate verification using JWT's:
- Client makes a request to a server, including a request token
- Server responds, and in the header it has signed the original request using x509 certificate
- Client verifies the signature using the public key in the certificate
- If the signature is valid, the client trusts the server

### 2025-02-24
I have made a new executor, which handles wakers better.
It seems that we can get hyper and rustls to work. But there is an issue with the virtio-net driver which causes the tls stream to fail halfway through.

### 2025-03-01
Rusttls and hyper now work together.

### 2025-03-14
Brainstorm Commands:
- help
- chain
- date
- getenv
- help
- ifconfig
- load
- ping
- print
- quit
- stats
- wget

### 2025-03-15
Implement command dispatch for the console service.

### 2025-03-18
Observed a potential bug, where posting a body from requests using a stream iterator causes the network layer to hang.

### 2025-03-19
Chainload now works. But there is a ref cycle bug which causes resource leaks for ports.

### 2025-03-20
Fixed the ref cycle bug, and removed a bunch of reference counters replacing them with lifetimes.

Next steps are:
- Load linux kernel, initramfs and device tree
- Embed and run the wasm payload into u-boot using mkimage FIT format
- Embed and load the configuration file into u-boot
- Validate hash of the chainload payload

distro.rst is a very good reference for the u-boot configuration.
We can use get_boot_src(), to get the disk which we booted from.

### 2025-03-23
We can now boot a linux kernel. Whats missing:
- Cmdline
    - This can be done by using the `bootargs` environment variable or by modifying the device tree
    - Initially we probably want to write to the bootargs environment variable
- Better device tree handling
- Verifying the length of the payloads
- Making a progress bar for loading
- Handle recovery from failed boot
- Disk boot
- Build initramfs from docker image [Project Bootfile]
- Improve proxyclient to allow multiple commands [DONE]
- Make the boot typed, so we can handle multiple boot types(Linux, OpenBSD, etc.)
- Multiple boot strategies depending on platform(e.g. 'booti' for aarch64, 'bootz' for x86_64)
- Move the payload type enum to protobuf schema
- Implement the gemini boot log

