# gRPC Gateway SDK

A lightweight SDK that allows you to expose your **gRPC services** over **HTTP/JSON**.  
This SDK accepts HTTP requests, invokes the corresponding gRPC service methods, and returns JSON responses generated from protobuf messages.

SDK doesn't need .proto files to user gRPC based services

---

## 🚀 Features
- Expose gRPC services via HTTP endpoints.
- Automatically translates:
  - HTTP request → gRPC method call
  - Protobuf response → JSON response
- **Authentication support** for internal communication between Gateway ↔ Service:
  - Supported auth types:
    - `API_KEY`
    - `JWT_TOKEN`
  - Services can define their own header name and value during registration.
  - Services can re-register with a new token anytime, and the Gateway will automatically use the updated token.
- Simple service registration API.
- Runs as a standalone HTTP service.
- Easy integration with existing gRPC applications.

---

## 🔑 Authentication Support

The Gateway now supports authentication for service communication.  
When a service registers itself, it can specify the type of authentication and provide the required header and value.

Currently supported auth types:
- **API_KEY**
- **JWT_TOKEN**

### Example: Registering a Service with API_KEY Authentication

```rust
let result = gateway.service_registry.register(ServiceRegisterRequest {
    service_name: String::from("users.UserService"),
    host: String::from("127.0.0.1"),
    port: String::from("50051"), 
    oauth_config: Some(AuthType::APIKey {
        header_name: String::from("x-api-key"),
        value: String::from("test_api_token"),
    }),
});

## 📦 Installation Add this crate to your Cargo.toml:
toml
[dependencies]
grpc_gateway = "0.1.2"

