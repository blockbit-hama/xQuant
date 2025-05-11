pub mod engine;
pub mod result;
pub mod scenario;
pub mod analyzer;
pub mod performance;
pub mod data_provider;
pub mod optimizer;
mod performance;

pub use engine::BacktestEngine;
pub use result::BacktestResult;
pub use scenario::{BacktestScenario, BacktestScenarioBuilder};
pub use analyzer::BacktestAnalyzer;
pub use performance::PerformanceMetrics;
pub use data_provider::HistoricalDataProvider;
pub use optimizer::PortfolioOptimizer;