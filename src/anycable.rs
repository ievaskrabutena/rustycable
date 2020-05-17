#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConnectionRequest {
    #[prost(string, tag = "1")]
    pub path: std::string::String,
    #[prost(map = "string, string", tag = "2")]
    pub headers: ::std::collections::HashMap<std::string::String, std::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConnectionResponse {
    #[prost(enumeration = "Status", tag = "1")]
    pub status: i32,
    #[prost(string, tag = "2")]
    pub identifiers: std::string::String,
    #[prost(string, repeated, tag = "3")]
    pub transmissions: ::std::vec::Vec<std::string::String>,
    #[prost(string, tag = "4")]
    pub error_msg: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommandMessage {
    #[prost(string, tag = "1")]
    pub command: std::string::String,
    #[prost(string, tag = "2")]
    pub identifier: std::string::String,
    #[prost(string, tag = "3")]
    pub connection_identifiers: std::string::String,
    #[prost(string, tag = "4")]
    pub data: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommandResponse {
    #[prost(enumeration = "Status", tag = "1")]
    pub status: i32,
    #[prost(bool, tag = "2")]
    pub disconnect: bool,
    #[prost(bool, tag = "3")]
    pub stop_streams: bool,
    #[prost(string, repeated, tag = "4")]
    pub streams: ::std::vec::Vec<std::string::String>,
    #[prost(string, repeated, tag = "5")]
    pub transmissions: ::std::vec::Vec<std::string::String>,
    #[prost(string, tag = "6")]
    pub error_msg: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DisconnectRequest {
    #[prost(string, tag = "1")]
    pub identifiers: std::string::String,
    #[prost(string, repeated, tag = "2")]
    pub subscriptions: ::std::vec::Vec<std::string::String>,
    #[prost(string, tag = "3")]
    pub path: std::string::String,
    #[prost(map = "string, string", tag = "4")]
    pub headers: ::std::collections::HashMap<std::string::String, std::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DisconnectResponse {
    #[prost(enumeration = "Status", tag = "1")]
    pub status: i32,
    #[prost(string, tag = "2")]
    pub error_msg: std::string::String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Status {
    Error = 0,
    Success = 1,
    Failure = 2,
}
#[doc = r" Generated client implementations."]
pub mod rpc_client {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    pub struct RpcClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl RpcClient<tonic::transport::Channel> {
        #[doc = r" Attempt to create a new client by connecting to a given endpoint."]
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> RpcClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::ResponseBody: Body + HttpBody + Send + 'static,
        T::Error: Into<StdError>,
        <T::ResponseBody as HttpBody>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor(inner: T, interceptor: impl Into<tonic::Interceptor>) -> Self {
            let inner = tonic::client::Grpc::with_interceptor(inner, interceptor);
            Self { inner }
        }
        pub async fn connect_client(
            &mut self,
            request: impl tonic::IntoRequest<super::ConnectionRequest>,
        ) -> Result<tonic::Response<super::ConnectionResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/anycable.RPC/Connect");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn command(
            &mut self,
            request: impl tonic::IntoRequest<super::CommandMessage>,
        ) -> Result<tonic::Response<super::CommandResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/anycable.RPC/Command");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn disconnect(
            &mut self,
            request: impl tonic::IntoRequest<super::DisconnectRequest>,
        ) -> Result<tonic::Response<super::DisconnectResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/anycable.RPC/Disconnect");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
    impl<T: Clone> Clone for RpcClient<T> {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
            }
        }
    }
    impl<T> std::fmt::Debug for RpcClient<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "RpcClient {{ ... }}")
        }
    }
}
