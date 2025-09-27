
# Gateway SDK – Circuit Breaker Integration

This SDK includes a **centralized circuit breaker** to protect services from unnecessary invalid calls and cascading failures.  

The circuit breaker is applied **at the gateway level** and **cannot be configured by individual services**. This ensures consistent reliability and fault tolerance across all downstream service calls.

---

## 🔹 Why Circuit Breaker?

In distributed systems, repeated calls to failing services can:

- Waste resources (CPU, memory, network).
- Increase latency for end-users.
- Amplify failures across the system.

A circuit breaker pattern helps to **fail fast**, prevent overload, and allow services time to recover.

---

## 🔹 Default Configuration

The following defaults are applied to all services:

```rust
failure_threshold: 5,                   // After 5 consecutive failures, the circuit opens
recovery_timeout: Duration::from_secs(30), // Wait 30s before attempting recovery
half_open_max_calls: 2,                 // Allow 2 test calls in half-open state
# Gateway SDK – Circuit Breaker Integration

This SDK includes a **centralized circuit breaker** to protect services from unnecessary invalid calls and cascading failures.  

The circuit breaker is applied **at the gateway level** and **cannot be configured by individual services**. This ensures consistent reliability and fault tolerance across all downstream service calls.

---

## 🔹 Why Circuit Breaker?
In distributed systems, repeated calls to failing services can:
- Waste resources (CPU, memory, network).
- Increase latency for end-users.
- Amplify failures across the system.

A circuit breaker pattern helps to **fail fast**, prevent overload, and allow services time to recover.

---

## 🔹 Default Configuration
The following defaults are applied to all services:

```rust
failure_threshold: 5,                   // After 5 consecutive failures, the circuit opens
recovery_timeout: Duration::from_secs(30), // Wait 30s before attempting recovery
half_open_max_calls: 2,                 // Allow 2 test calls in half-open state
