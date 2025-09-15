# Authentication System - Version Update

<!--toc:start-->
- [Authentication System - Version Update](#authentication-system-version-update)
  - [Overview](#overview)
  - [🔐 Supported Authentication Types](#🔐-supported-authentication-types)
    - [1. API Key Authentication (Existing)](#1-api-key-authentication-existing)
    - [2. JWT Token Authentication (New)](#2-jwt-token-authentication-new)
  - [🚀 New Features](#🚀-new-features)
    - [JWT Token Authentication with Auto-Refresh](#jwt-token-authentication-with-auto-refresh)
  - [📋 Configuration Guide](#📋-configuration-guide)
    - [Service Registration](#service-registration)
    - [Authentication Configuration](#authentication-configuration)
  - [🛠️ Implementation Examples](#🛠️-implementation-examples)
    - [Example 1: API Key Authentication](#example-1-api-key-authentication)
    - [Example 2: JWT Token Authentication](#example-2-jwt-token-authentication)
  - [📡 Protocol Buffer Definition (Required)](#📡-protocol-buffer-definition-required)
  - [🔄 How Token Refresh Works](#🔄-how-token-refresh-works)
    - [Flow Diagram](#flow-diagram)
    - [Key Benefits](#key-benefits)
  - [🔧 Configuration Types](#🔧-configuration-types)
    - [Static Configuration](#static-configuration)
    - [Dynamic Configuration](#dynamic-configuration)
  - [📝 Setup Checklist](#📝-setup-checklist)
    - [For API Key Authentication](#for-api-key-authentication)
    - [For JWT Token Authentication](#for-jwt-token-authentication)
  - [🚨 Important Notes](#🚨-important-notes)
  - [🔍 Troubleshooting](#🔍-troubleshooting)
    - [Common Issues](#common-issues)
  - [📊 Monitoring](#📊-monitoring)
  - [🔄 Migration Guide](#🔄-migration-guide)
    - [From API Key to JWT Token](#from-api-key-to-jwt-token)
<!--toc:end-->

## Overview

This version introduces **JWT Token Authentication** support for internal communication between gateway and services, alongside the existing API Key authentication. The system now supports multiple authentication methods with automatic token refresh capabilities.

## 🔐 Supported Authentication Types

### 1. API Key Authentication (Existing)

- **Type**: `AuthType::ApiKey`
- **Usage**: Simple API key-based authentication
- **Configuration**: Static API key in headers

### 2. JWT Token Authentication (New)

- **Type**: `AuthType::JwtToken`
- **Usage**: JWT-based authentication with automatic refresh
- **Configuration**: Dynamic token management with refresh capabilities

## 🚀 New Features

### JWT Token Authentication with Auto-Refresh

The system now supports **pull-based orchestration** for JWT token management:

- ✅ **Runtime Token Validation**: Token expiry is checked on every API call
- ✅ **Automatic Token Refresh**: When token expires, system automatically calls refresh endpoint
- ✅ **Centralized Protocol**: Standardized protobuf message for all services
- ✅ **Zero Downtime**: Seamless token refresh without service interruption

## 📋 Configuration Guide

### Service Registration

Before starting your service, you must register it with the gateway using the following configuration:

```rust
#[derive(Debug, Clone)]
pub struct ServiceRegisterRequest {
    pub service_name: String,
    pub host: String,
    pub port: String,
    pub health_check_endpoint: String,
    pub oauth_config: InternalAuthConfig,
}
```

### Authentication Configuration

```rust
#[derive(Debug, Clone)]
pub struct InternalAuthConfig {
    pub auth_type: AuthType,
    pub auth_refresh_config: Option<AuthRefreshConfig>,
}

#[derive(Debug, Clone)]
pub struct AuthRefreshConfig {
    pub service_name: String,
    pub method: String,
    pub header_name: String,
    pub access_token: String,
    pub expired_at: u64,
    pub refresh_token: String,
}
```

## 🛠️ Implementation Examples

### Example 1: API Key Authentication

```rust
let service_request = ServiceRegisterRequest {
    service_name: "user-service".to_string(),
    host: "localhost".to_string(),
    port: "8080".to_string(),
    health_check_endpoint: "/health".to_string(),
    oauth_config: InternalAuthConfig {
        auth_type: AuthType::ApiKey,
        auth_refresh_config: None, // No refresh needed for API keys
    },
};
```

### Example 2: JWT Token Authentication

```rust
let service_request = ServiceRegisterRequest {
    service_name: "payment-service".to_string(),
    host: "localhost".to_string(),
    port: "8081".to_string(),
    health_check_endpoint: "/health".to_string(),
    oauth_config: InternalAuthConfig {
        auth_type: AuthType::JwtToken,
        auth_refresh_config: Some(AuthRefreshConfig {
            service_name: "auth-service".to_string(),
            method: "RefreshAuth".to_string(),
            header_name: "Authorization".to_string(),
            access_token: "initial_jwt_token".to_string(),
            expired_at: 1641024000, // Unix timestamp
            refresh_token: "refresh_jwt_token".to_string(),
        }),
    },
};
```

## 📡 Protocol Buffer Definition (Required)

**⚠️ Important**: All services using JWT authentication must implement this exact protobuf definition. **Do not modify this protocol**:

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

## 🔄 How Token Refresh Works

### Flow Diagram

```
1. Service makes API call to Gateway
2. Gateway checks token expiry (runtime validation)
3. If token expired:
   ┌─ Gateway calls RefreshAuth service
   ├─ Gets new access_token and refresh_token
   ├─ Updates internal configuration
   └─ Retries original API call with new token
4. If token valid: Process request normally
```

### Key Benefits

- **Zero Configuration Changes**: Once set up, refresh happens automatically
- **Pull-Based**: Gateway initiates refresh when needed
- **Resilient**: System continues working even during token transitions
- **Standardized**: All services use the same refresh protocol

## 🔧 Configuration Types

### Static Configuration

- Service name, host, port
- Health check endpoint
- Authentication type selection
- Initial token values

### Dynamic Configuration

- Access tokens (refreshed automatically)
- Token expiry times (updated on refresh)
- Refresh tokens (rotated on refresh)

## 📝 Setup Checklist

### For API Key Authentication

- [ ] Set `auth_type` to `AuthType::ApiKey`
- [ ] Set `auth_refresh_config` to `None`
- [ ] Provide static API key in your service

### For JWT Token Authentication

- [ ] Set `auth_type` to `AuthType::JwtToken`
- [ ] Implement the required protobuf service
- [ ] Configure `AuthRefreshConfig` with initial tokens
- [ ] Ensure refresh service is accessible from gateway
- [ ] Set proper token expiry times

## 🚨 Important Notes

1. **Registration Required**: Services must register **before** starting the server
2. **Protocol Compliance**: The protobuf definition must not be modified
3. **Token Management**: Gateway handles all token refresh logic automatically
4. **Health Checks**: Ensure health check endpoints are properly configured
5. **Error Handling**: Gateway will retry failed refresh attempts

## 🔍 Troubleshooting

### Common Issues

**Token Refresh Fails**

- Verify refresh service is running and accessible
- Check protobuf implementation matches exactly
- Ensure refresh tokens are valid

**Service Registration Fails**

- Verify all required fields are provided
- Check network connectivity between services
- Ensure health check endpoint responds correctly

**Authentication Errors**

- Verify `auth_type` matches your implementation
- Check token format and expiry times
- Ensure header names match configuration

## 📊 Monitoring

The system provides automatic monitoring for:

- Token refresh events
- Authentication failures
- Service health status
- Token expiry warnings

## 🔄 Migration Guide

### From API Key to JWT Token

1. Implement the required protobuf service
2. Update service registration configuration
3. Change `auth_type` from `ApiKey` to `JwtToken`
4. Add `AuthRefreshConfig` with initial tokens
5. Re-register your service

---

**Version**: Latest
**Compatibility**: Backward compatible with existing API Key authentication
**Support**: JWT Token authentication is now the recommended approach for new services
