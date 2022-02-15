use crate::ApparatusError;
use flexi_logger::{FileSpec, Logger as FlexiLogger, WriteMode};

pub(crate) struct Logger {
    _handle: flexi_logger::LoggerHandle,
}

impl Logger {
    pub(crate) fn init() -> Result<Self, ApparatusError> {
        let handle = FlexiLogger::try_with_str("debug")
            .map_err(|e| ApparatusError::Logger(e.into()))?
            .log_to_file(FileSpec::default().suppress_timestamp())
            .write_mode(WriteMode::Async)
            .start()
            .map_err(|e| ApparatusError::Logger(e.into()))?;

        let logger = Self { _handle: handle };

        Ok(logger)
    }
}
