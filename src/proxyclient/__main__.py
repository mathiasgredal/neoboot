"""This is a proxy client for the wasm_oss server."""

import base64
import hashlib
from pathlib import Path

import requests

import proto_py.schema_pb2 as schema_pb2

long_body = Path('dist/wasm_oss/main.wasm').read_bytes()

req = schema_pb2.ClientRequest(
    inner=schema_pb2.ClientRequest.ClientRequestInner(
        chain_request=schema_pb2.ChainClientRequest(
            payload_size=len(long_body),
            payload_sha256=hashlib.sha256(long_body).hexdigest(),
        )
    ),
    signature=None,
)

msg_base64 = base64.b64encode(req.SerializeToString())
print(msg_base64)

# Read the wasm file using pathlib

# print length in megabytes
print(f'{len(long_body) / 1024 / 1024} MB')

resp = requests.post(
    'http://localhost:8080/api/v1/rpc',
    headers={'X-Client-Request': msg_base64},
    data=long_body,
)

print(resp.headers)
