pub mod model;
pub mod repository;

pub use repository::PgExtraPeriodRepository;

#[cfg(test)]
mod tests;
