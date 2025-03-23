"""This is a proxy client for the wasm_oss server."""

import base64
import hashlib
from pathlib import Path

import msgpack
import requests

import proto_py.schema_pb2 as schema_pb2

output: bytes = msgpack.packb(
    {
        'kernel_addr_r': Path('dist/linux/aarch64/Image').read_bytes(),
        'ramdisk_addr_r': Path('dist/initramfs/aarch64/initramfs.cpio.gz').read_bytes(),
    }
)

req = schema_pb2.ClientRequest(
    inner=schema_pb2.ClientRequest.ClientRequestInner(
        boot_request=schema_pb2.BootClientRequest(
            boot_type=schema_pb2.BootClientRequest.BootType.BOOT_TYPE_LINUX,
            payload_size=len(output),
            payload_sha256=hashlib.sha256(output).hexdigest(),
        )
    ),
    signature=None,
)


msg_base64 = base64.b64encode(req.SerializeToString())

resp = requests.post(
    'http://localhost:8080/api/v1/rpc',
    headers={'X-Client-Request': msg_base64},
    data=output,
)

print(resp.headers)

# long_body = Path('dist/wasm_oss/main.wasm').read_bytes()


# msg_base64 = base64.b64encode(req.SerializeToString())
# print(msg_base64)

# # Read the wasm file using pathlib

# # print length in megabytes
# print(f'{len(long_body) / 1024 / 1024} MB')
