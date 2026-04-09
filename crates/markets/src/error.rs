use thiserror::Error;
use tonic::Status;
use tracing::error;
#[derive(Debug, Error)]
pub enum ProgramError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

pub type ProgramResult<T> = Result<T, ProgramError>;


impl From<ProgramError> for Status {
    fn from(err: ProgramError) -> Self{
       match err {
         ProgramError::NotFound(e) => {
              error!("Not found error: {:?}", e);
              Status::not_found("Resource not found")
         }

         ProgramError::Validation(msg) => Status::invalid_argument(msg),

         ProgramError::Database(e) => {
            error!("Database error: {}", e);
            Status::internal("Failed to process request")
         }

       }
 
    }
}