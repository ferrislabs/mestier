pub mod model;
pub mod repository;
#[cfg(test)]
mod tests;

pub use repository::PgKnowledgeArticleRepository;
