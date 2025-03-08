#[link(name = "nvdaControllerClient")]
extern "stdcall" {
    fn nvdaController_speakText(message: *const u16) -> libc::c_int;
    fn nvdaController_cancelSpeech();
}

pub fn speak(s: &str) {
    let c: Vec<u16> = s.encode_utf16().chain([0]).collect();
    unsafe { nvdaController_speakText(c.as_ptr()) };
}

pub fn stop_speaking() {
    unsafe {
        nvdaController_cancelSpeech();
    };
}
