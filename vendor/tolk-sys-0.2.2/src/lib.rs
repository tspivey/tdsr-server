extern crate libc;
use libc::wchar_t;

#[link(name = "tolk")]
extern "C" {
    pub fn Tolk_Load();
    pub fn Tolk_IsLoaded() -> bool;
    pub fn Tolk_Unload();

    pub fn Tolk_TrySAPI(trySAPI: bool);
    pub fn Tolk_PreferSAPI(preferSAPI: bool);

    pub fn Tolk_DetectScreenReader() -> *const wchar_t;

    pub fn Tolk_HasSpeech() -> bool;
    pub fn Tolk_HasBraille() -> bool;

    pub fn Tolk_Output(str: *const wchar_t, interrupt: bool) -> bool;

    pub fn Tolk_Speak(str: *const wchar_t, interrupt: bool) -> bool;
    pub fn Tolk_Braille(str: *const wchar_t) -> bool;

    pub fn Tolk_IsSpeaking() -> bool;
    pub fn Tolk_Silence() -> bool;
}

#[test]
fn load_unload() {
    unsafe {
        Tolk_Load();
        assert_eq!(Tolk_IsLoaded(), true);
        Tolk_Unload();
        assert_eq!(Tolk_IsLoaded(), false);
    }
}
