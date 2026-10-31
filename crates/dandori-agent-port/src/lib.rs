//! Cổng giao tiếp agent (trait AgentAdapter và kiểu liên quan).

#![warn(missing_docs)]

/// Khởi tạo module agent port.
#[must_use]
pub fn init() -> &'static str {
    "dandori-agent-port initialized"
}
