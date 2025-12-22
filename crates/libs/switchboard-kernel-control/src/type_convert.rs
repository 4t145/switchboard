use switchboard_model::{
    self as model,
    chrono::{DateTime, Utc},
};

use crate::kernel::{KernelStateKind, RunningState};

impl From<super::kernel::KernelInfo> for model::kernel::KernelInfo {
    fn from(info: super::kernel::KernelInfo) -> Self {
        model::kernel::KernelInfo {
            name: info.name,
            id: info.id,
            description: info.description,
            meta: info
                .meta
                .map(model::kernel::KernelMeta::from)
                .unwrap_or_default(),
        }
    }
}

impl Into<super::kernel::KernelInfo> for model::kernel::KernelInfo {
    fn into(self) -> super::kernel::KernelInfo {
        super::kernel::KernelInfo {
            name: self.name,
            id: self.id,
            description: self.description,
            meta: Some(self.meta.into()),
        }
    }
}

impl From<super::kernel::KernelMeta> for model::kernel::KernelMeta {
    fn from(value: super::kernel::KernelMeta) -> Self {
        model::kernel::KernelMeta {
            version: value.version,
            build: value.build,
        }
    }
}

impl Into<super::kernel::KernelMeta> for model::kernel::KernelMeta {
    fn into(self) -> super::kernel::KernelMeta {
        super::kernel::KernelMeta {
            version: self.version,
            build: self.build,
        }
    }
}

impl From<super::kernel::ErrorStack> for model::error::ErrorStack {
    fn from(value: super::kernel::ErrorStack) -> Self {
        model::error::ErrorStack {
            frames: value
                .frames
                .into_iter()
                .map(model::error::ErrorStackFrame::from)
                .collect(),
        }
    }
}

impl Into<super::kernel::ErrorStack> for model::error::ErrorStack {
    fn into(self) -> super::kernel::ErrorStack {
        super::kernel::ErrorStack {
            frames: self.frames.into_iter().map(|f| f.into()).collect(),
        }
    }
}

impl From<super::kernel::ErrorFrame> for model::error::ErrorStackFrame {
    fn from(value: super::kernel::ErrorFrame) -> Self {
        model::error::ErrorStackFrame {
            error: value.message,
            type_name: value.type_name,
        }
    }
}

impl Into<super::kernel::ErrorFrame> for model::error::ErrorStackFrame {
    fn into(self) -> super::kernel::ErrorFrame {
        super::kernel::ErrorFrame {
            message: self.error,
            type_name: self.type_name,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TryFromProtoKernelStateError {
    #[error("Failed to parse datetime: {0}")]
    DateTimeParseError(#[from] switchboard_model::chrono::ParseError),
    #[error("Missing field kind")]
    MissingFieldKind,
}

impl TryFrom<super::kernel::KernelState> for model::kernel::KernelState {
    type Error = TryFromProtoKernelStateError;
    fn try_from(value: super::kernel::KernelState) -> Result<Self, Self::Error> {
        let since = DateTime::parse_from_rfc2822(&value.since)?.to_utc();
        Ok(model::kernel::KernelState {
            kind: match value.kind.and_then(|k| k.kind) {
                Some(super::kernel::kernel_state_kind::Kind::Running(RunningState {
                    config_version,
                })) => model::kernel::KernelStateKind::Running { config_version },
                Some(super::kernel::kernel_state_kind::Kind::WaitingConfig(_)) => {
                    model::kernel::KernelStateKind::WaitingConfig
                }
                Some(super::kernel::kernel_state_kind::Kind::Updating(update)) => {
                    model::kernel::KernelStateKind::Updating {
                        original_config_version: update.current_version,
                        new_config_version: update.updating_to_version,
                    }
                }
                Some(super::kernel::kernel_state_kind::Kind::ShuttingDown(_)) => {
                    model::kernel::KernelStateKind::ShuttingDown
                }
                Some(super::kernel::kernel_state_kind::Kind::Stopped(_)) => {
                    model::kernel::KernelStateKind::Stopped
                }
                None => return Err(TryFromProtoKernelStateError::MissingFieldKind),
            },
            since,
        })
    }
}

impl Into<super::kernel::KernelState> for model::kernel::KernelState {
    fn into(self) -> super::kernel::KernelState {
        let kind = match self.kind {
            model::kernel::KernelStateKind::Running { config_version } => {
                super::kernel::kernel_state_kind::Kind::Running(RunningState { config_version })
            }
            model::kernel::KernelStateKind::WaitingConfig => {
                super::kernel::kernel_state_kind::Kind::WaitingConfig(
                    super::kernel::WaitingConfigState {},
                )
            }
            model::kernel::KernelStateKind::Updating {
                original_config_version,
                new_config_version,
            } => super::kernel::kernel_state_kind::Kind::Updating(super::kernel::UpdatingState {
                current_version: original_config_version,
                updating_to_version: new_config_version,
            }),
            model::kernel::KernelStateKind::ShuttingDown => {
                super::kernel::kernel_state_kind::Kind::ShuttingDown(
                    super::kernel::ShuttingDownState {},
                )
            }
            model::kernel::KernelStateKind::Stopped => {
                super::kernel::kernel_state_kind::Kind::Stopped(super::kernel::StoppedState {})
            }
        };
        super::kernel::KernelState {
            kind: Some(KernelStateKind { kind: Some(kind) }),
            since: self.since.to_rfc2822(),
        }
    }
}
