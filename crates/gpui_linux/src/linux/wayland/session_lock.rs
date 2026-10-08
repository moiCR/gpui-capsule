pub use gpui::session_lock::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LockAction {
    Unlock,
    Destroy,
}

#[derive(Default)]
pub(crate) struct SessionLockLifecycle {
    locked: bool,
    closing: bool,
}

impl SessionLockLifecycle {
    pub fn is_closing(&self) -> bool {
        self.closing
    }

    pub fn close(&mut self) -> Option<LockAction> {
        self.closing = true;
        self.locked.then_some(LockAction::Unlock)
    }

    pub fn locked(&mut self) -> Option<LockAction> {
        self.locked = true;
        self.closing.then_some(LockAction::Unlock)
    }

    pub fn finished(&mut self) -> LockAction {
        self.closing = true;
        if self.locked {
            LockAction::Unlock
        } else {
            LockAction::Destroy
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_close_waits_for_compositor_decision() {
        let mut lock = SessionLockLifecycle::default();
        assert_eq!(lock.close(), None);
        assert!(lock.is_closing());
        assert_eq!(lock.locked(), Some(LockAction::Unlock));
    }

    #[test]
    fn rejected_pending_close_destroys_without_unlocking() {
        let mut lock = SessionLockLifecycle::default();
        assert_eq!(lock.close(), None);
        assert_eq!(lock.finished(), LockAction::Destroy);
    }

    #[test]
    fn acquired_lock_closes_with_unlock() {
        let mut lock = SessionLockLifecycle::default();
        assert_eq!(lock.locked(), None);
        assert_eq!(lock.close(), Some(LockAction::Unlock));
    }

    #[test]
    fn finished_acquired_lock_still_requires_unlock() {
        let mut lock = SessionLockLifecycle::default();
        assert_eq!(lock.locked(), None);
        assert_eq!(lock.finished(), LockAction::Unlock);
    }
}
