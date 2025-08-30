# rust_grpc_gateway
# gRPC Gateway SDK

A lightweight SDK that allows you to expose your **gRPC services** over **HTTP/JSON**.  
This SDK accepts HTTP requests, invokes the corresponding gRPC service methods, and returns JSON responses generated from protobuf messages.

---

## 🚀 Features

- Expose gRPC services via HTTP endpoints.
- Automatically translates:
  - HTTP request → gRPC method call
  - Protobuf response → JSON response
- Simple service registration API.
- Runs as a standalone HTTP service.
- Easy integration with existing gRPC applications.

---

## 📦 Installation

Add this crate to your `Cargo.toml`:

```toml
[dependencies]
grpc_gateway = "0.1.1"
