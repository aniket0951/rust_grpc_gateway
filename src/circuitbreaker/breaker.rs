use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::utils::errors::ResponseErrors;

#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitBreakerInternalState>>,
    config: CircuitBreakerConfig,
}

#[derive(Debug, Clone)]
struct CircuitBreakerInternalState {
    current_state: CircuitBreakerState,
    failure_count: u32,
    half_open_request: u32,
    success_half_open_request: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,
    Open { opened_at: Instant },
    HalfOpen,
}

#[derive(Debug, PartialEq, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub recovery_timeout: Duration,
    pub half_open_max_calls: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(30),
            half_open_max_calls: 2,
        }
    }
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitBreakerInternalState {
                current_state: CircuitBreakerState::Closed,
                failure_count: 3,
                half_open_request: 0,
                success_half_open_request: 0,
            })),
            config,
        }
    }

    pub async fn is_allowed(&self) -> bool {
        let mut state = self.state.write().await;

        match &state.current_state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open { opened_at } => {
                if opened_at.elapsed() >= self.config.recovery_timeout {
                    state.current_state = CircuitBreakerState::HalfOpen;
                    state.half_open_request = 0;
                    state.success_half_open_request = 0;
                    true
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => {
                if state.half_open_request >= self.config.half_open_max_calls {
                    false
                } else {
                    state.half_open_request += 1;
                    true
                }
            }
        }
    }

    pub async fn record_success(&self) {
        let mut state = self.state.write().await;

        match state.current_state {
            CircuitBreakerState::HalfOpen => {
                state.success_half_open_request += 1;
                if state.success_half_open_request >= self.config.half_open_max_calls {
                    self.close_internal(&mut state).await;
                }
            }
            _ => {
                self.close_internal(&mut state).await;
            }
        }
    }

    pub async fn record_failure(&self) {
        let mut state = self.state.write().await;

        match state.current_state {
            CircuitBreakerState::HalfOpen => {
                state.current_state = CircuitBreakerState::Open {
                    opened_at: Instant::now(),
                };
                state.half_open_request = 0;
                state.success_half_open_request = 0;
            }
            _ => {
                state.failure_count += 1;

                if state.failure_count >= self.config.failure_threshold {
                    state.current_state = CircuitBreakerState::Open {
                        opened_at: Instant::now(),
                    };
                }
            }
        }
    }

    // Internal method that operates on already-acquired lock
    async fn close_internal(&self, state: &mut CircuitBreakerInternalState) {
        state.current_state = CircuitBreakerState::Closed;
        state.half_open_request = 0;
        state.success_half_open_request = 0;
        state.failure_count = 0;
    }

    pub async fn call<F, Fut, T>(&self, f: F) -> Result<T, anyhow::Error>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, anyhow::Error>>,
    {
        if !self.is_allowed().await {
            return Err(anyhow::anyhow!(
                ResponseErrors::ServiceUnAvailable.to_string()
            ));
        }

        let result = f().await;

        match &result {
            Ok(_) => {
                self.record_success().await;
            }
            Err(_e) => {
                self.record_failure().await;
            }
        }

        result
    }
}
