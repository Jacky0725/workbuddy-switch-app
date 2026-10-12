//! Conservative UI check-in state machine. A platform driver may only mark a
//! result successful after observing the official client's post-action state.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiCheckinPhase {
    SwitchingAccount,
    WaitingForClient,
    CheckingStatus,
    ConfirmingIdentityBeforeClick,
    CheckingIn,
    ConfirmingIdentityAfterClick,
    VerifyingResult,
    Finished,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiCheckinStatus {
    Success,
    AlreadyCheckedIn,
    Failed,
    NeedsUserAction,
    Unsupported,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservedCheckinState {
    Ready,
    CheckedIn,
    NeedsUserAction,
    Unsupported,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiCheckinAction {
    WaitForClient,
    CheckStatus,
    VerifyIdentity,
    InvokeCheckin,
    VerifyResult,
    Finish(UiCheckinStatus),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiCheckinFlow {
    expected_uid: String,
    phase: UiCheckinPhase,
    status: Option<UiCheckinStatus>,
}

impl UiCheckinFlow {
    pub fn new(expected_uid: String) -> Result<Self, &'static str> {
        if expected_uid.trim().is_empty() {
            return Err("目标账号身份缺失");
        }
        Ok(Self {
            expected_uid,
            phase: UiCheckinPhase::SwitchingAccount,
            status: None,
        })
    }

    pub fn phase(&self) -> UiCheckinPhase {
        self.phase
    }

    pub fn status(&self) -> Option<UiCheckinStatus> {
        self.status
    }

    pub fn switched(&mut self, completed: bool) -> UiCheckinAction {
        if self.phase != UiCheckinPhase::SwitchingAccount {
            return self.fail_unknown();
        }
        if !completed {
            return self.finish(UiCheckinStatus::Failed);
        }
        self.phase = UiCheckinPhase::WaitingForClient;
        UiCheckinAction::WaitForClient
    }

    /// The platform driver must observe this identity in the official client,
    /// not infer it from the Manager account selected for switching.
    pub fn client_identity(&mut self, observed_uid: Option<&str>) -> UiCheckinAction {
        if observed_uid != Some(self.expected_uid.as_str()) {
            return self.fail_unknown();
        }
        match self.phase {
            UiCheckinPhase::WaitingForClient => {
                self.phase = UiCheckinPhase::CheckingStatus;
                UiCheckinAction::CheckStatus
            }
            UiCheckinPhase::ConfirmingIdentityBeforeClick => {
                self.phase = UiCheckinPhase::CheckingIn;
                UiCheckinAction::InvokeCheckin
            }
            UiCheckinPhase::ConfirmingIdentityAfterClick => {
                self.phase = UiCheckinPhase::VerifyingResult;
                UiCheckinAction::VerifyResult
            }
            _ => self.fail_unknown(),
        }
    }

    /// Identity and check-in state must come from the same client observation.
    pub fn observed_status(
        &mut self,
        observed_uid: Option<&str>,
        state: ObservedCheckinState,
    ) -> UiCheckinAction {
        if observed_uid != Some(self.expected_uid.as_str()) {
            return self.fail_unknown();
        }
        match self.phase {
            UiCheckinPhase::CheckingStatus => match state {
                ObservedCheckinState::CheckedIn => self.finish(UiCheckinStatus::AlreadyCheckedIn),
                ObservedCheckinState::Ready => {
                    self.phase = UiCheckinPhase::ConfirmingIdentityBeforeClick;
                    UiCheckinAction::VerifyIdentity
                }
                other => self.finish(Self::unresolved_status(other)),
            },
            UiCheckinPhase::VerifyingResult => match state {
                ObservedCheckinState::CheckedIn => self.finish(UiCheckinStatus::Success),
                other => self.finish(Self::unresolved_status(other)),
            },
            _ => self.fail_unknown(),
        }
    }

    /// Clicking is never evidence of success. A fresh client observation is required.
    pub fn invoked(&mut self, _reported_complete: bool) -> UiCheckinAction {
        if self.phase != UiCheckinPhase::CheckingIn {
            return self.fail_unknown();
        }
        self.phase = UiCheckinPhase::ConfirmingIdentityAfterClick;
        // Even a failed/uncertain click may have reached the client. Verify
        // identity and state before retrying or reporting a result.
        UiCheckinAction::VerifyIdentity
    }

    fn unresolved_status(state: ObservedCheckinState) -> UiCheckinStatus {
        match state {
            ObservedCheckinState::NeedsUserAction => UiCheckinStatus::NeedsUserAction,
            ObservedCheckinState::Unsupported => UiCheckinStatus::Unsupported,
            ObservedCheckinState::Ready | ObservedCheckinState::Unknown => UiCheckinStatus::Unknown,
            ObservedCheckinState::CheckedIn => UiCheckinStatus::Unknown,
        }
    }

    fn fail_unknown(&mut self) -> UiCheckinAction {
        self.finish(UiCheckinStatus::Unknown)
    }

    fn finish(&mut self, status: UiCheckinStatus) -> UiCheckinAction {
        self.phase = UiCheckinPhase::Finished;
        self.status = Some(status);
        UiCheckinAction::Finish(status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ready_flow() -> UiCheckinFlow {
        let mut flow = UiCheckinFlow::new("account-a".into()).unwrap();
        assert_eq!(flow.switched(true), UiCheckinAction::WaitForClient);
        assert_eq!(
            flow.client_identity(Some("account-a")),
            UiCheckinAction::CheckStatus
        );
        flow
    }

    #[test]
    fn requires_observed_identity_before_status_or_click() {
        let mut flow = UiCheckinFlow::new("account-a".into()).unwrap();
        assert_eq!(flow.switched(true), UiCheckinAction::WaitForClient);
        assert_eq!(
            flow.client_identity(Some("account-b")),
            UiCheckinAction::Finish(UiCheckinStatus::Unknown)
        );
        assert_eq!(flow.phase(), UiCheckinPhase::Finished);
        assert!(UiCheckinFlow::new(" ".into()).is_err());
    }

    #[test]
    fn click_without_post_action_confirmation_is_unknown() {
        let mut flow = ready_flow();
        assert_eq!(
            flow.observed_status(Some("account-a"), ObservedCheckinState::Ready),
            UiCheckinAction::VerifyIdentity
        );
        assert_eq!(
            flow.client_identity(Some("account-a")),
            UiCheckinAction::InvokeCheckin
        );
        assert_eq!(flow.invoked(true), UiCheckinAction::VerifyIdentity);
        assert_eq!(
            flow.client_identity(Some("account-a")),
            UiCheckinAction::VerifyResult
        );
        assert_eq!(flow.status(), None);
        assert_eq!(
            flow.observed_status(Some("account-a"), ObservedCheckinState::Ready),
            UiCheckinAction::Finish(UiCheckinStatus::Unknown)
        );
    }

    #[test]
    fn only_verified_post_action_state_is_success() {
        let mut flow = ready_flow();
        assert_eq!(
            flow.observed_status(Some("account-a"), ObservedCheckinState::Ready),
            UiCheckinAction::VerifyIdentity
        );
        assert_eq!(
            flow.client_identity(Some("account-a")),
            UiCheckinAction::InvokeCheckin
        );
        assert_eq!(flow.invoked(false), UiCheckinAction::VerifyIdentity);
        assert_eq!(
            flow.client_identity(Some("account-a")),
            UiCheckinAction::VerifyResult
        );
        assert_eq!(
            flow.observed_status(Some("account-a"), ObservedCheckinState::CheckedIn),
            UiCheckinAction::Finish(UiCheckinStatus::Success)
        );
        assert_eq!(flow.status(), Some(UiCheckinStatus::Success));
    }

    #[test]
    fn identity_change_after_click_cannot_report_success() {
        let mut flow = ready_flow();
        assert_eq!(
            flow.observed_status(Some("account-a"), ObservedCheckinState::Ready),
            UiCheckinAction::VerifyIdentity
        );
        assert_eq!(
            flow.client_identity(Some("account-a")),
            UiCheckinAction::InvokeCheckin
        );
        assert_eq!(flow.invoked(true), UiCheckinAction::VerifyIdentity);
        assert_eq!(
            flow.client_identity(Some("account-b")),
            UiCheckinAction::Finish(UiCheckinStatus::Unknown)
        );
    }

    #[test]
    fn status_for_another_account_cannot_report_success() {
        let mut flow = ready_flow();
        assert_eq!(
            flow.observed_status(Some("account-b"), ObservedCheckinState::CheckedIn),
            UiCheckinAction::Finish(UiCheckinStatus::Unknown)
        );
    }

    #[test]
    fn already_checked_in_never_requests_a_click() {
        let mut flow = ready_flow();
        assert_eq!(
            flow.observed_status(Some("account-a"), ObservedCheckinState::CheckedIn),
            UiCheckinAction::Finish(UiCheckinStatus::AlreadyCheckedIn)
        );
    }

    #[test]
    fn blocked_client_requires_user_action() {
        let mut flow = ready_flow();
        assert_eq!(
            flow.observed_status(Some("account-a"), ObservedCheckinState::NeedsUserAction),
            UiCheckinAction::Finish(UiCheckinStatus::NeedsUserAction)
        );
    }
}
