// This file is @generated by prost-build.
/// Help command (cmdPattern: "help")
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct HelpClientRequest {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HelpClientResponse {
    #[prost(string, tag = "1")]
    pub message: ::prost::alloc::string::String,
}
/// Print command (cmdPattern: "print <message>")
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PrintClientRequest {
    #[prost(string, tag = "1")]
    pub message: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PrintClientResponse {
    #[prost(string, tag = "1")]
    pub message: ::prost::alloc::string::String,
}
/// Nonce command (cmdPattern: "nonce")
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct NonceClientRequest {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NonceClientResponse {
    #[prost(string, tag = "1")]
    pub nonce: ::prost::alloc::string::String,
}
/// Client request
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientRequest {
    /// The inner payload of the client request
    #[prost(message, optional, tag = "1")]
    pub inner: ::core::option::Option<client_request::ClientRequestInner>,
    #[prost(oneof = "client_request::SignatureType", tags = "2")]
    pub signature_type: ::core::option::Option<client_request::SignatureType>,
}
/// Nested message and enum types in `ClientRequest`.
pub mod client_request {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ClientRequestInner {
        /// A unique identifier for the request, used to prevent replay attacks
        #[prost(string, tag = "4")]
        pub nonce: ::prost::alloc::string::String,
        #[prost(oneof = "client_request_inner::Payload", tags = "1, 2, 3")]
        pub payload: ::core::option::Option<client_request_inner::Payload>,
    }
    /// Nested message and enum types in `ClientRequestInner`.
    pub mod client_request_inner {
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        pub enum Payload {
            #[prost(message, tag = "1")]
            HelpRequest(super::super::HelpClientRequest),
            #[prost(message, tag = "2")]
            PrintRequest(super::super::PrintClientRequest),
            #[prost(message, tag = "3")]
            NonceRequest(super::super::NonceClientRequest),
        }
    }
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum SignatureType {
        #[prost(message, tag = "2")]
        Signature(super::FullSignature),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientResponse {
    /// The inner payload of the client response
    #[prost(message, optional, tag = "1")]
    pub inner: ::core::option::Option<client_response::ClientResponseInner>,
    #[prost(oneof = "client_response::SignatureType", tags = "2")]
    pub signature_type: ::core::option::Option<client_response::SignatureType>,
}
/// Nested message and enum types in `ClientResponse`.
pub mod client_response {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ClientResponseInner {
        /// The same nonce as in the client request
        #[prost(string, tag = "4")]
        pub nonce: ::prost::alloc::string::String,
        #[prost(oneof = "client_response_inner::Payload", tags = "1, 2, 3")]
        pub payload: ::core::option::Option<client_response_inner::Payload>,
    }
    /// Nested message and enum types in `ClientResponseInner`.
    pub mod client_response_inner {
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        pub enum Payload {
            #[prost(message, tag = "1")]
            HelpResponse(super::super::HelpClientResponse),
            #[prost(message, tag = "2")]
            PrintResponse(super::super::PrintClientResponse),
            #[prost(message, tag = "3")]
            NonceResponse(super::super::NonceClientResponse),
        }
    }
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum SignatureType {
        #[prost(message, tag = "2")]
        Signature(super::ClientSignature),
    }
}
/// Whoami command
///
/// Empty message
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct WhoamiServerRequest {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WhoamiServerResponse {
    #[prost(string, tag = "1")]
    pub whoami: ::prost::alloc::string::String,
}
/// Nonce command
///
/// Empty message
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct NonceServerRequest {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NonceServerResponse {
    #[prost(string, tag = "1")]
    pub nonce: ::prost::alloc::string::String,
}
/// Server request
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ServerRequest {
    /// The inner payload of the server request
    #[prost(message, optional, tag = "1")]
    pub inner: ::core::option::Option<server_request::ServerRequestInner>,
    #[prost(oneof = "server_request::SignatureType", tags = "2")]
    pub signature_type: ::core::option::Option<server_request::SignatureType>,
}
/// Nested message and enum types in `ServerRequest`.
pub mod server_request {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ServerRequestInner {
        /// The nonce of the server request
        #[prost(string, tag = "3")]
        pub nonce: ::prost::alloc::string::String,
        #[prost(oneof = "server_request_inner::Payload", tags = "1, 2")]
        pub payload: ::core::option::Option<server_request_inner::Payload>,
    }
    /// Nested message and enum types in `ServerRequestInner`.
    pub mod server_request_inner {
        #[derive(Clone, Copy, PartialEq, ::prost::Oneof)]
        pub enum Payload {
            #[prost(message, tag = "1")]
            NonceRequest(super::super::NonceServerRequest),
            #[prost(message, tag = "2")]
            WhoamiRequest(super::super::WhoamiServerRequest),
        }
    }
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum SignatureType {
        #[prost(message, tag = "2")]
        Signature(super::ClientSignature),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ServerResponse {
    /// The inner payload of the server response
    #[prost(message, optional, tag = "1")]
    pub inner: ::core::option::Option<server_response::ServerResponseInner>,
    #[prost(oneof = "server_response::SignatureType", tags = "2")]
    pub signature_type: ::core::option::Option<server_response::SignatureType>,
}
/// Nested message and enum types in `ServerResponse`.
pub mod server_response {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ServerResponseInner {
        /// The nonce of the client response
        #[prost(string, tag = "3")]
        pub nonce: ::prost::alloc::string::String,
        #[prost(oneof = "server_response_inner::Payload", tags = "1, 2")]
        pub payload: ::core::option::Option<server_response_inner::Payload>,
    }
    /// Nested message and enum types in `ServerResponseInner`.
    pub mod server_response_inner {
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        pub enum Payload {
            #[prost(message, tag = "1")]
            NonceResponse(super::super::NonceServerResponse),
            #[prost(message, tag = "2")]
            WhoamiResponse(super::super::WhoamiServerResponse),
        }
    }
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum SignatureType {
        #[prost(message, tag = "2")]
        Signature(super::ClientSignature),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct X509Chain {
    /// The chain of certificates
    #[prost(string, repeated, tag = "1")]
    pub chain: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// The roles of the certificates, as a JWT
    #[prost(string, tag = "2")]
    pub certificate_roles: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FullSignature {
    /// The payload hash
    #[prost(string, tag = "1")]
    pub payload_sha256: ::prost::alloc::string::String,
    /// The chain of certificates
    #[prost(message, optional, tag = "2")]
    pub certificate_chain: ::core::option::Option<X509Chain>,
    /// The user signature, as a JWT of the payload hash
    #[prost(string, tag = "3")]
    pub user_signature: ::prost::alloc::string::String,
    /// The server signature, as a JWT of the user signature
    #[prost(string, tag = "4")]
    pub server_signature: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientSignature {
    /// The payload hash
    #[prost(string, tag = "1")]
    pub payload_sha256: ::prost::alloc::string::String,
    /// The client certificate
    #[prost(string, tag = "2")]
    pub client_certificate: ::prost::alloc::string::String,
    /// A signature of the hash of the client certificate
    #[prost(message, optional, tag = "3")]
    pub client_certificate_signature: ::core::option::Option<FullSignature>,
    /// The payload signature, as a JWT of the payload hash
    #[prost(string, tag = "4")]
    pub payload_signature: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ServerSignature {
    /// The payload hash
    #[prost(string, tag = "1")]
    pub payload_sha256: ::prost::alloc::string::String,
    /// The chain of certificates
    #[prost(message, optional, tag = "2")]
    pub certificate_chain: ::core::option::Option<X509Chain>,
    /// The server signature, as a JWT of the payload hash
    #[prost(string, tag = "3")]
    pub server_signature: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserSignature {
    /// The payload hash
    #[prost(string, tag = "1")]
    pub payload_sha256: ::prost::alloc::string::String,
    /// The chain of certificates
    #[prost(message, optional, tag = "2")]
    pub certificate_chain: ::core::option::Option<X509Chain>,
    /// The user signature, as a JWT of the payload hash
    #[prost(string, tag = "3")]
    pub user_signature: ::prost::alloc::string::String,
}
