
/// Why an order was not forwarded. Sent back to the client as a reject
/// execution report; every variant is a normal business outcome, not a fault.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectReason {
    InvalidPrice,
    InvalidQuantity,
    InvalidSide,
    /// Outbound pipeline saturated: order was NOT accepted.
    SystemBusy,
}

/// The fate of one order. `Err(ProgramError)` is reserved for "the pipeline
/// itself is broken and this thread cannot continue" — never for a
/// per-order outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Disposition {
    Forwarded,
    Rejected(RejectReason),
}
