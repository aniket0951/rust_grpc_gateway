# gRPC Gateway SDK

A lightweight SDK that allows you to expose your **gRPC services** over **HTTP/JSON**.  
This SDK accepts HTTP requests, invokes the corresponding gRPC service methods, and returns JSON responses generated from protobuf messages.

👉 No `.proto` files are required to use your gRPC services with this SDK.

## 🚀 New in v0.1.5

This release introduces **JWT Token Authentication** with automatic refresh capabilities alongside existing API Key authentication.

### 🔐 Enhanced Authentication Features
- **JWT Token Authentication** with automatic token refresh
- **Pull-based orchestration** for token management
- **Runtime token validation** on every API call
- **Centralized protobuf protocol** for refresh operations
- **Zero downtime** token refresh

👉 See full details in the [Authentication System README](src/registry/README.md).

---

## 🚀 Features

- Expose gRPC services via HTTP endpoints.
- Automatically translates:
  - HTTP request → gRPC method call
  - Protobuf response → JSON response
- **Advanced Authentication support** for internal communication between Gateway ↔ Service:
  - Supported auth types:
    - `API_KEY` - Simple static authentication
    - `JWT_TOKEN` - Dynamic authentication with auto-refresh
  - **Automatic token refresh** when JWT tokens expire
  - Services can define their own header name and authentication configuration during registration.
  - **Runtime token validation** ensures secure communication
  - Services can re-register with new tokens anytime, and the Gateway will automatically use the updated configuration.
- **Service Registry** with health monitoring
- Simple service registration API with comprehensive configuration options
- Runs as a standalone HTTP service.
- Easy integration with existing gRPC applications.

---

## 🔑 Authentication Support

The Gateway now supports **two authentication methods** for secure service communication.  
When a service registers itself, it can specify the authentication type and provide the required configuration.

### Currently supported auth types

- **API_KEY** - Static authentication with custom headers
- **JWT_TOKEN** - Dynamic authentication with automatic token refresh

---

### Example: Registering a Service with API_KEY Authentication

```rust
let result = gateway.service_registry.register(ServiceRegisterRequest {
    service_name: String::from("users.UserService"),
    host: String::from("127.0.0.1"),
    port: String::from("50051"),
    health_check_endpoint: String::from("/health"),
    oauth_config: InternalAuthConfig {
        auth_type: AuthType::ApiKey,
        auth_refresh_config: None,
    },
});
```

### Example: Registering a Service with JWT_TOKEN Authentication

```rust
let result = gateway.service_registry.register(ServiceRegisterRequest {
    service_name: String::from("payment.PaymentService"),
    host: String::from("127.0.0.1"),
    port: String::from("50052"),
    health_check_endpoint: String::from("/health"),
    oauth_config: InternalAuthConfig {
        auth_type: AuthType::JwtToken,
        auth_refresh_config: Some(AuthRefreshConfig {
            service_name: String::from("auth-service"),
            method: String::from("RefreshAuth"),
            header_name: String::from("Authorization"),
            access_token: String::from("initial_jwt_token"),
            expired_at: 1641024000, // Unix timestamp
            refresh_token: String::from("refresh_jwt_token"),
        }),
    },
});
```

---

## 🔄 JWT Token Auto-Refresh

The new JWT authentication system features **automatic token refresh**:

- **Runtime Validation**: Token expiry checked on every API call
- **Automatic Refresh**: Gateway automatically refreshes expired tokens
- **Pull-based Orchestration**: Gateway initiates refresh when needed
- **Standardized Protocol**: All services must implement the same refresh protobuf service
- **Zero Downtime**: Seamless operation during token refresh

### Required Protobuf Implementation

All services using JWT authentication **must implement** this exact protobuf service:

```protobuf
syntax = "proto3";
package refresh;

message RefreshAuthTokenRequest {
    string refresh_token = 1;
}

message RefreshAuthTokenResponse {
    string access_token = 1;
    string refresh_token = 2;
    uint64 expired_at = 3;
}

service RefreshAuthService {
    rpc RefreshAuth (RefreshAuthTokenRequest) returns (RefreshAuthTokenResponse);
}
```

---

## 📋 Quick Start Guide

### 1. Choose Authentication Method

**For Simple Services (API Key)**:
- Use `AuthType::ApiKey`
- Set static header name and value
- No additional implementation required

**For Production Services (JWT Token)**:
- Use `AuthType::JwtToken`
- Implement the required refresh protobuf service
- Configure initial tokens and refresh endpoint

### 2. Register Your Service

Services **must register before starting** to enable Gateway communication.

### 3. Start Your Service

Once registered, the Gateway will:
- Route HTTP requests to your gRPC service
- Handle authentication automatically
- Refresh tokens when needed (JWT only)
- Monitor service health

---

## 🔍 Configuration Types

### Static Configuration
- Service connection details (host, port)
- Authentication type selection
- Health check endpoints

### Dynamic Configuration (JWT only)
- Access tokens (auto-refreshed)
- Token expiry times (auto-updated)
- Refresh tokens (auto-rotated)

---

## 📚 Documentation

- **[Complete Authentication Guide](./src/registry/README.md)** - Detailed setup and configuration
- **[Migration Guide](./src/registry/README.md#-migration-guide)** - Upgrading from API Key to JWT
- **[Troubleshooting](./src/registry/README.md#-troubleshooting)** - Common issues and solutions

---

## 🚨 Important Notes

- **Service Registration Required**: All services must register before starting
- **Protocol Compliance**: JWT services must implement the exact protobuf definition
- **Backward Compatibility**: Existing API Key authentication continues to work
- **Recommended**: JWT Token authentication for new production services

---

**Version**: v0.1.6  
**Compatibility**: Supports both API Key and JWT Token authentication  
**Migration**: Seamless upgrade path from API Key to JWT authentication