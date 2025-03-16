from jose import jwt
from OpenSSL import crypto
import base64

# Load the sub-key private key
with open("sub_private_key.pem", "rb") as f:
    sub_private_key = f.read()

# Load the sub-key certificate and root certificate
with open("sub_cert.pem", "rb") as f:
    sub_cert = crypto.load_certificate(crypto.FILETYPE_PEM, f.read())
with open("root_cert.pem", "rb") as f:
    root_cert = crypto.load_certificate(crypto.FILETYPE_PEM, f.read())

# Convert certificates to Base64-encoded DER format for x5c
sub_cert_der = base64.b64encode(crypto.dump_certificate(crypto.FILETYPE_ASN1, sub_cert)).decode("utf-8")
root_cert_der = base64.b64encode(crypto.dump_certificate(crypto.FILETYPE_ASN1, root_cert)).decode("utf-8")

# Define the JWT headers, including x5c
headers = {
    "x5c": [sub_cert_der, root_cert_der]  # Certificate chain (sub-cert first, then root-cert)
}

import time

# Define the JWT payload
payload = {
    "sub": "user123",
    # "iat": int(time.time()),
    # "exp": int(time.time()) + 600,  # 10 minutes
    "iss": "example.com"
}

# Sign the JWT with the sub-key private key
token = jwt.encode(
    claims=payload,
    key=sub_private_key,
    algorithm="RS256",
    headers=headers
)

print("JWT:", token)