//! No configuration is supported until the Windows/OBS matrix in docs/overlay.md
//! has been executed. Affinity success is necessary but never sufficient.
#[cfg(windows)]
pub(super) fn may_show(validated_configuration: bool, affinity: bool, composition: bool) -> bool {
    validated_configuration && affinity && composition
}
#[cfg(windows)]
pub(super) fn configuration_validated() -> bool {
    false
}
