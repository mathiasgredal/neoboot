from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class HelpClientRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class HelpClientResponse(_message.Message):
    __slots__ = ("message",)
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    message: str
    def __init__(self, message: _Optional[str] = ...) -> None: ...

class PrintClientRequest(_message.Message):
    __slots__ = ("message",)
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    message: str
    def __init__(self, message: _Optional[str] = ...) -> None: ...

class PrintClientResponse(_message.Message):
    __slots__ = ("message",)
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    message: str
    def __init__(self, message: _Optional[str] = ...) -> None: ...

class NonceClientRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class NonceClientResponse(_message.Message):
    __slots__ = ("nonce",)
    NONCE_FIELD_NUMBER: _ClassVar[int]
    nonce: str
    def __init__(self, nonce: _Optional[str] = ...) -> None: ...

class QuitClientRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class QuitClientResponse(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class StatusClientRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class StatusClientResponse(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ChainClientRequest(_message.Message):
    __slots__ = ("payload_size", "payload_sha256")
    PAYLOAD_SIZE_FIELD_NUMBER: _ClassVar[int]
    PAYLOAD_SHA256_FIELD_NUMBER: _ClassVar[int]
    payload_size: int
    payload_sha256: str
    def __init__(self, payload_size: _Optional[int] = ..., payload_sha256: _Optional[str] = ...) -> None: ...

class ChainClientResponse(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class BootClientRequest(_message.Message):
    __slots__ = ("boot_type", "payload_size", "payload_sha256")
    class BootType(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
        __slots__ = ()
        BOOT_TYPE_LINUX: _ClassVar[BootClientRequest.BootType]
    BOOT_TYPE_LINUX: BootClientRequest.BootType
    BOOT_TYPE_FIELD_NUMBER: _ClassVar[int]
    PAYLOAD_SIZE_FIELD_NUMBER: _ClassVar[int]
    PAYLOAD_SHA256_FIELD_NUMBER: _ClassVar[int]
    boot_type: BootClientRequest.BootType
    payload_size: int
    payload_sha256: str
    def __init__(self, boot_type: _Optional[_Union[BootClientRequest.BootType, str]] = ..., payload_size: _Optional[int] = ..., payload_sha256: _Optional[str] = ...) -> None: ...

class BootClientResponse(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ErrorClientResponse(_message.Message):
    __slots__ = ("error",)
    ERROR_FIELD_NUMBER: _ClassVar[int]
    error: str
    def __init__(self, error: _Optional[str] = ...) -> None: ...

class ClientRequest(_message.Message):
    __slots__ = ("inner", "signature")
    class ClientRequestInner(_message.Message):
        __slots__ = ("nonce", "help_request", "print_request", "nonce_request", "quit_request", "chain_request", "status_request", "boot_request")
        NONCE_FIELD_NUMBER: _ClassVar[int]
        HELP_REQUEST_FIELD_NUMBER: _ClassVar[int]
        PRINT_REQUEST_FIELD_NUMBER: _ClassVar[int]
        NONCE_REQUEST_FIELD_NUMBER: _ClassVar[int]
        QUIT_REQUEST_FIELD_NUMBER: _ClassVar[int]
        CHAIN_REQUEST_FIELD_NUMBER: _ClassVar[int]
        STATUS_REQUEST_FIELD_NUMBER: _ClassVar[int]
        BOOT_REQUEST_FIELD_NUMBER: _ClassVar[int]
        nonce: str
        help_request: HelpClientRequest
        print_request: PrintClientRequest
        nonce_request: NonceClientRequest
        quit_request: QuitClientRequest
        chain_request: ChainClientRequest
        status_request: StatusClientRequest
        boot_request: BootClientRequest
        def __init__(self, nonce: _Optional[str] = ..., help_request: _Optional[_Union[HelpClientRequest, _Mapping]] = ..., print_request: _Optional[_Union[PrintClientRequest, _Mapping]] = ..., nonce_request: _Optional[_Union[NonceClientRequest, _Mapping]] = ..., quit_request: _Optional[_Union[QuitClientRequest, _Mapping]] = ..., chain_request: _Optional[_Union[ChainClientRequest, _Mapping]] = ..., status_request: _Optional[_Union[StatusClientRequest, _Mapping]] = ..., boot_request: _Optional[_Union[BootClientRequest, _Mapping]] = ...) -> None: ...
    INNER_FIELD_NUMBER: _ClassVar[int]
    SIGNATURE_FIELD_NUMBER: _ClassVar[int]
    inner: ClientRequest.ClientRequestInner
    signature: FullSignature
    def __init__(self, inner: _Optional[_Union[ClientRequest.ClientRequestInner, _Mapping]] = ..., signature: _Optional[_Union[FullSignature, _Mapping]] = ...) -> None: ...

class ClientResponse(_message.Message):
    __slots__ = ("inner", "signature")
    class ClientResponseInner(_message.Message):
        __slots__ = ("nonce", "error_response", "help_response", "print_response", "nonce_response", "quit_response", "chain_response", "status_response", "boot_response")
        NONCE_FIELD_NUMBER: _ClassVar[int]
        ERROR_RESPONSE_FIELD_NUMBER: _ClassVar[int]
        HELP_RESPONSE_FIELD_NUMBER: _ClassVar[int]
        PRINT_RESPONSE_FIELD_NUMBER: _ClassVar[int]
        NONCE_RESPONSE_FIELD_NUMBER: _ClassVar[int]
        QUIT_RESPONSE_FIELD_NUMBER: _ClassVar[int]
        CHAIN_RESPONSE_FIELD_NUMBER: _ClassVar[int]
        STATUS_RESPONSE_FIELD_NUMBER: _ClassVar[int]
        BOOT_RESPONSE_FIELD_NUMBER: _ClassVar[int]
        nonce: str
        error_response: ErrorClientResponse
        help_response: HelpClientResponse
        print_response: PrintClientResponse
        nonce_response: NonceClientResponse
        quit_response: QuitClientResponse
        chain_response: ChainClientResponse
        status_response: StatusClientResponse
        boot_response: BootClientResponse
        def __init__(self, nonce: _Optional[str] = ..., error_response: _Optional[_Union[ErrorClientResponse, _Mapping]] = ..., help_response: _Optional[_Union[HelpClientResponse, _Mapping]] = ..., print_response: _Optional[_Union[PrintClientResponse, _Mapping]] = ..., nonce_response: _Optional[_Union[NonceClientResponse, _Mapping]] = ..., quit_response: _Optional[_Union[QuitClientResponse, _Mapping]] = ..., chain_response: _Optional[_Union[ChainClientResponse, _Mapping]] = ..., status_response: _Optional[_Union[StatusClientResponse, _Mapping]] = ..., boot_response: _Optional[_Union[BootClientResponse, _Mapping]] = ...) -> None: ...
    INNER_FIELD_NUMBER: _ClassVar[int]
    SIGNATURE_FIELD_NUMBER: _ClassVar[int]
    inner: ClientResponse.ClientResponseInner
    signature: ClientSignature
    def __init__(self, inner: _Optional[_Union[ClientResponse.ClientResponseInner, _Mapping]] = ..., signature: _Optional[_Union[ClientSignature, _Mapping]] = ...) -> None: ...

class WhoamiServerRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class WhoamiServerResponse(_message.Message):
    __slots__ = ("whoami",)
    WHOAMI_FIELD_NUMBER: _ClassVar[int]
    whoami: str
    def __init__(self, whoami: _Optional[str] = ...) -> None: ...

class NonceServerRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class NonceServerResponse(_message.Message):
    __slots__ = ("nonce",)
    NONCE_FIELD_NUMBER: _ClassVar[int]
    nonce: str
    def __init__(self, nonce: _Optional[str] = ...) -> None: ...

class ServerRequest(_message.Message):
    __slots__ = ("inner", "signature")
    class ServerRequestInner(_message.Message):
        __slots__ = ("nonce_request", "whoami_request", "nonce")
        NONCE_REQUEST_FIELD_NUMBER: _ClassVar[int]
        WHOAMI_REQUEST_FIELD_NUMBER: _ClassVar[int]
        NONCE_FIELD_NUMBER: _ClassVar[int]
        nonce_request: NonceServerRequest
        whoami_request: WhoamiServerRequest
        nonce: str
        def __init__(self, nonce_request: _Optional[_Union[NonceServerRequest, _Mapping]] = ..., whoami_request: _Optional[_Union[WhoamiServerRequest, _Mapping]] = ..., nonce: _Optional[str] = ...) -> None: ...
    INNER_FIELD_NUMBER: _ClassVar[int]
    SIGNATURE_FIELD_NUMBER: _ClassVar[int]
    inner: ServerRequest.ServerRequestInner
    signature: ClientSignature
    def __init__(self, inner: _Optional[_Union[ServerRequest.ServerRequestInner, _Mapping]] = ..., signature: _Optional[_Union[ClientSignature, _Mapping]] = ...) -> None: ...

class ServerResponse(_message.Message):
    __slots__ = ("inner", "signature")
    class ServerResponseInner(_message.Message):
        __slots__ = ("nonce_response", "whoami_response", "nonce")
        NONCE_RESPONSE_FIELD_NUMBER: _ClassVar[int]
        WHOAMI_RESPONSE_FIELD_NUMBER: _ClassVar[int]
        NONCE_FIELD_NUMBER: _ClassVar[int]
        nonce_response: NonceServerResponse
        whoami_response: WhoamiServerResponse
        nonce: str
        def __init__(self, nonce_response: _Optional[_Union[NonceServerResponse, _Mapping]] = ..., whoami_response: _Optional[_Union[WhoamiServerResponse, _Mapping]] = ..., nonce: _Optional[str] = ...) -> None: ...
    INNER_FIELD_NUMBER: _ClassVar[int]
    SIGNATURE_FIELD_NUMBER: _ClassVar[int]
    inner: ServerResponse.ServerResponseInner
    signature: ClientSignature
    def __init__(self, inner: _Optional[_Union[ServerResponse.ServerResponseInner, _Mapping]] = ..., signature: _Optional[_Union[ClientSignature, _Mapping]] = ...) -> None: ...

class X509Chain(_message.Message):
    __slots__ = ("chain", "certificate_roles")
    CHAIN_FIELD_NUMBER: _ClassVar[int]
    CERTIFICATE_ROLES_FIELD_NUMBER: _ClassVar[int]
    chain: _containers.RepeatedScalarFieldContainer[str]
    certificate_roles: str
    def __init__(self, chain: _Optional[_Iterable[str]] = ..., certificate_roles: _Optional[str] = ...) -> None: ...

class FullSignature(_message.Message):
    __slots__ = ("payload_sha256", "certificate_chain", "user_signature", "server_signature")
    PAYLOAD_SHA256_FIELD_NUMBER: _ClassVar[int]
    CERTIFICATE_CHAIN_FIELD_NUMBER: _ClassVar[int]
    USER_SIGNATURE_FIELD_NUMBER: _ClassVar[int]
    SERVER_SIGNATURE_FIELD_NUMBER: _ClassVar[int]
    payload_sha256: str
    certificate_chain: X509Chain
    user_signature: str
    server_signature: str
    def __init__(self, payload_sha256: _Optional[str] = ..., certificate_chain: _Optional[_Union[X509Chain, _Mapping]] = ..., user_signature: _Optional[str] = ..., server_signature: _Optional[str] = ...) -> None: ...

class ClientSignature(_message.Message):
    __slots__ = ("payload_sha256", "client_certificate", "client_certificate_signature", "payload_signature")
    PAYLOAD_SHA256_FIELD_NUMBER: _ClassVar[int]
    CLIENT_CERTIFICATE_FIELD_NUMBER: _ClassVar[int]
    CLIENT_CERTIFICATE_SIGNATURE_FIELD_NUMBER: _ClassVar[int]
    PAYLOAD_SIGNATURE_FIELD_NUMBER: _ClassVar[int]
    payload_sha256: str
    client_certificate: str
    client_certificate_signature: FullSignature
    payload_signature: str
    def __init__(self, payload_sha256: _Optional[str] = ..., client_certificate: _Optional[str] = ..., client_certificate_signature: _Optional[_Union[FullSignature, _Mapping]] = ..., payload_signature: _Optional[str] = ...) -> None: ...

class ServerSignature(_message.Message):
    __slots__ = ("payload_sha256", "certificate_chain", "server_signature")
    PAYLOAD_SHA256_FIELD_NUMBER: _ClassVar[int]
    CERTIFICATE_CHAIN_FIELD_NUMBER: _ClassVar[int]
    SERVER_SIGNATURE_FIELD_NUMBER: _ClassVar[int]
    payload_sha256: str
    certificate_chain: X509Chain
    server_signature: str
    def __init__(self, payload_sha256: _Optional[str] = ..., certificate_chain: _Optional[_Union[X509Chain, _Mapping]] = ..., server_signature: _Optional[str] = ...) -> None: ...

class UserSignature(_message.Message):
    __slots__ = ("payload_sha256", "certificate_chain", "user_signature")
    PAYLOAD_SHA256_FIELD_NUMBER: _ClassVar[int]
    CERTIFICATE_CHAIN_FIELD_NUMBER: _ClassVar[int]
    USER_SIGNATURE_FIELD_NUMBER: _ClassVar[int]
    payload_sha256: str
    certificate_chain: X509Chain
    user_signature: str
    def __init__(self, payload_sha256: _Optional[str] = ..., certificate_chain: _Optional[_Union[X509Chain, _Mapping]] = ..., user_signature: _Optional[str] = ...) -> None: ...
