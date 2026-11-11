use super::{ConfirmDialogEvent, DecisionGate};

#[test]
fn every_close_path_is_a_single_decision_and_cannot_confirm_after_cancel() {
    for first in [ConfirmDialogEvent::Confirm, ConfirmDialogEvent::Cancel] {
        let gate = DecisionGate::default();
        assert_eq!(gate.decide(first), Some(first));
        assert_eq!(gate.decide(ConfirmDialogEvent::Cancel), None);
        assert_eq!(gate.decide(ConfirmDialogEvent::Confirm), None);
    }
}
