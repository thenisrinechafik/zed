use std::sync::atomic::{AtomicBool, Ordering};

use windows::Win32::UI::Input::Ime::{
    CANDIDATEFORM, COMPOSITIONFORM, CFS_CANDIDATEPOS, CFS_POINT,
};
use windows::Win32::Foundation::POINT;

static IME_ACTIVE: AtomicBool = AtomicBool::new(false);

pub(crate) fn mark_ime_active(active: bool) {
    IME_ACTIVE.store(active, Ordering::SeqCst);
}

pub(crate) fn is_ime_active() -> bool {
    IME_ACTIVE.load(Ordering::SeqCst)
}

pub(crate) fn composition_form(pt: POINT) -> COMPOSITIONFORM {
    COMPOSITIONFORM {
        dwStyle: CFS_POINT,
        ptCurrentPos: pt,
        ..Default::default()
    }
}

pub(crate) fn candidate_form(pt: POINT) -> CANDIDATEFORM {
    CANDIDATEFORM {
        dwStyle: CFS_CANDIDATEPOS,
        ptCurrentPos: pt,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggles_ime_state() {
        mark_ime_active(true);
        assert!(is_ime_active());
        mark_ime_active(false);
        assert!(!is_ime_active());
    }

    #[test]
    fn builds_candidate_form() {
        let point = POINT { x: 10, y: 20 };
        let form = candidate_form(point);
        assert_eq!(form.dwStyle, CFS_CANDIDATEPOS);
        assert_eq!(form.ptCurrentPos.x, 10);
        assert_eq!(form.ptCurrentPos.y, 20);
    }
}
