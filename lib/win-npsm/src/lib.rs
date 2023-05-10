use windows::{
    core::{IUnknown, IUnknown_Vtbl, GUID},
    Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CLSCTX_LOCAL_SERVER, CLSCTX_NO_CODE_DOWNLOAD,
        COINIT_MULTITHREADED,
    },
};

// Adapted from https://github.com/ADeltaX/NPSMLib
#[windows_interface::interface("3B6A7908-CE07-4BA9-878C-6E4A15DB5E5B")]
unsafe trait INowPlayingSessionManager: IUnknown {
    fn get_Count(&self, count: *mut usize) -> i32;
}

const CLSID_NowPlayingSessionManager: GUID = GUID::from_values(
    0xBCBB9860,
    0xC012,
    0x4AD7,
    [0xA9, 0x38, 0x6E, 0x33, 0x7A, 0xE6, 0xAB, 0xA5],
);

#[test]
fn basic() -> windows::core::Result<()> {
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED)?;

        let npsm: INowPlayingSessionManager = CoCreateInstance(
            &CLSID_NowPlayingSessionManager,
            None,
            CLSCTX_LOCAL_SERVER | CLSCTX_NO_CODE_DOWNLOAD,
        )?;
        let mut count = 0usize;
        let res = npsm.get_Count(&mut count);
        dbg!((res, count));
    };

    Ok(())
}
