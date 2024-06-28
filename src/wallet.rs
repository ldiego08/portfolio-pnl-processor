#[derive(Debug, Default)]
pub(crate) struct WalletPnL {
    pub realized: f64,
    pub unrealized: f64,
}