#[cfg(target_os = "windows")]
use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;
use tolk_sys::*;
use widestring::U16CString;

lazy_static! {
    static ref TOLK: Mutex<Option<Arc<Tolk>>> = Mutex::new(None);
}

/// Speaks SSML through the active screen-reader driver.
pub fn speak_ssml<S: Into<String>>(ssml: S) -> bool {
    let ssml = U16CString::from_str(ssml.into());
    match ssml {
        Ok(ssml) => unsafe { Tolk_SpeakSsml(ssml.as_ptr()) },
        Err(_) => false,
    }
}

#[derive(Clone, Debug)]
pub struct Tolk;

impl Tolk {
    pub fn new() -> Arc<Tolk> {
        let mut tolk = TOLK.lock().unwrap();
        if let Some(tolk) = tolk.as_mut() {
            tolk.clone()
        } else {
            unsafe {
                Tolk_Load();
            }
            let new_tolk = Arc::new(Tolk);
            *tolk = Some(new_tolk.clone());
            new_tolk
        }
    }

    pub fn try_sapi(&self, v: bool) {
        unsafe {
            Tolk_TrySAPI(v);
        }
    }

    pub fn prefer_sapi(&self, v: bool) {
        unsafe {
            Tolk_PreferSAPI(v);
        }
    }

    pub fn has_speech(&self) -> bool {
        unsafe { Tolk_HasSpeech() }
    }

    pub fn has_braille(&self) -> bool {
        unsafe { Tolk_HasBraille() }
    }

    pub fn output<S: Into<String>>(&self, s: S, interrupt: bool) {
        let s = s.into();
        let s = U16CString::from_str(s);
        if let Some(s) = s.ok() {
            unsafe {
                Tolk_Output(s.as_ptr(), interrupt);
            }
        }
    }

    pub fn speak<S: Into<String>>(&self, s: S, interrupt: bool) -> bool {
        let s = s.into();
        let s = U16CString::from_str(s);
        if let Some(s) = s.ok() {
            unsafe { Tolk_Speak(s.as_ptr(), interrupt) }
        } else {
            false
        }
    }

    pub fn braille<S: Into<String>>(&self, s: S) -> bool {
        let s = s.into();
        let s = U16CString::from_str(s);
        if let Some(s) = s.ok() {
            unsafe { Tolk_Braille(s.as_ptr()) }
        } else {
            false
        }
    }

    pub fn is_speaking(&self) -> bool {
        unsafe { Tolk_IsSpeaking() }
    }

    pub fn silence(&self) -> bool {
        unsafe { Tolk_Silence() }
    }

    pub fn detect_screen_reader(&self) -> Option<String> {
        let screen_reader = unsafe { Tolk_DetectScreenReader() };
        if screen_reader.is_null() {
            None
        } else {
            let screen_reader = unsafe { U16CString::from_ptr_with_nul(screen_reader, 64) };
            if let Some(screen_reader) = screen_reader.ok() {
                screen_reader.to_string().ok()
            } else {
                None
            }
        }
    }
}

impl Drop for Tolk {
    fn drop(&mut self) {
        let tolk = TOLK.lock().unwrap();
        let unload = if let Some(tolk) = &*tolk {
            Arc::strong_count(&tolk) == 0
        } else {
            false
        };
        if unload {
            unsafe {
                Tolk_Unload();
            }
        }
    }
}
