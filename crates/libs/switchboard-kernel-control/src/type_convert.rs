use switchboard_model::{self as model, chrono::DateTime};

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

impl From<model::kernel::KernelInfo> for super::kernel::KernelInfo {
    fn from(val: model::kernel::KernelInfo) -> Self {
        super::kernel::KernelInfo {
            name: val.name,
            id: val.id,
            description: val.description,
            meta: Some(val.meta.into()),
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

impl From<model::kernel::KernelMeta> for super::kernel::KernelMeta {
    fn from(val: model::kernel::KernelMeta) -> Self {
        super::kernel::KernelMeta {
            version: val.version,
            build: val.build,
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

impl From<model::error::ErrorStack> for super::kernel::ErrorStack {
    fn from(val: model::error::ErrorStack) -> Self {
        super::kernel::ErrorStack {
            frames: val.frames.into_iter().map(|f| f.into()).collect(),
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

impl From<model::error::ErrorStackFrame> for super::kernel::ErrorFrame {
    fn from(val: model::error::ErrorStackFrame) -> Self {
        super::kernel::ErrorFrame {
            message: val.error,
            type_name: val.type_name,
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

impl From<model::kernel::KernelState> for super::kernel::KernelState {
    fn from(val: model::kernel::KernelState) -> Self {
        let kind = match val.kind {
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
            since: val.since.to_rfc2822(),
        }
    }
}
