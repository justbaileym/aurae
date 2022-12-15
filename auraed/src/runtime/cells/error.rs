use crate::runtime::cells::{CellName, ExecutableName};
use log::error;
use std::io;
use thiserror::Error;
use tonic::Status;

pub(crate) type Result<T> = std::result::Result<T, CellsError>;

#[derive(Error, Debug)]
pub(crate) enum CellsError {
    #[error("cell '{cell_name}' already exists'")]
    CellExists { cell_name: CellName },
    #[error("cell '{cell_name}' not found'")]
    CellNotFound { cell_name: CellName },
    #[error(
        "executable '{executable_name}' already exists in cell '{cell_name}'"
    )]
    ExecutableExists { cell_name: CellName, executable_name: ExecutableName },
    #[error("executable '{executable_name}' not found in '{cell_name}'")]
    ExecutableNotFound { cell_name: CellName, executable_name: ExecutableName },
    #[error("failed to lock cells cache")]
    FailedToObtainLock(),
    // TODO: ideally have the below be better
    #[error(transparent)]
    CgroupsError(#[from] cgroups_rs::error::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
}

impl From<CellsError> for Status {
    fn from(err: CellsError) -> Self {
        let msg = err.to_string();
        error!("{msg}");
        match err {
            CellsError::CellExists { .. }
            | CellsError::ExecutableExists { .. } => {
                Status::already_exists(msg)
            }
            CellsError::CellNotFound { .. }
            | CellsError::ExecutableNotFound { .. } => Status::not_found(msg),
            CellsError::FailedToObtainLock() => {
                // TODO (future-highway): I don't know what the conventions are of revealing
                //  messages that reveal the workings of the system to the api consumer
                //  in this type of application.
                //  For now, taking the safe route and not exposing the error message.
                Status::aborted("")
            }
            CellsError::CgroupsError(_) | CellsError::Io(_) => {
                Status::internal("")
            }
        }
    }
}
