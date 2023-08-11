use windows::{
    core::{ComInterface, IUnknown, IUnknown_Vtbl, GUID, HRESULT, PCWSTR, PWSTR},
    Foundation::EventRegistrationToken,
    Win32::{
        Foundation::{HANDLE, HWND},
        System::Com::{
            CoCreateInstance, CoInitializeEx, CLSCTX_LOCAL_SERVER, CLSCTX_NO_CODE_DOWNLOAD,
            COINIT_MULTITHREADED,
        },
    },
};

#[repr(transparent)]
struct NowPlayingSessionType(pub i32);

impl NowPlayingSessionType {
    pub const Unknown: Self = Self(0);
    pub const PlayTo: Self = Self(1);
    pub const Local: Self = Self(2);
}

// >= 19041
// Adapted from https://github.com/ADeltaX/NPSMLib
#[windows_interface::interface("3B6A7908-CE07-4BA9-878C-6E4A15DB5E5B")]
unsafe trait INowPlayingSessionManager: IUnknown {
    fn get_Count(&self, count: *mut usize) -> HRESULT;

    fn get_CurrentSession(&self, ppSession: *mut *mut IUnknown) -> HRESULT;

    fn AddSession(
        &self,
        ty: &NowPlayingSessionType,
        hwnd: HWND,
        dwPID: u32,
        appId: PWSTR,
        szSourceDeviceId: PWSTR,
        szRenderDeviceId: PWSTR,
        source: PWSTR,
        pMediaControl: *mut IUnknown,
        pConnection: *mut IUnknown,
        fMarkAsCurrentSession: bool,
        processHandle: HANDLE,
    ) -> HRESULT;

    fn RemoveSession(&self, pInfo: *mut IUnknown) -> HRESULT;

    fn GetSessions(&self, pdwCount: *mut u32, pppSessions: *mut *mut IUnknown) -> HRESULT;

    fn FindSession(&self, pInfo: *mut IUnknown, ppSession: *mut *mut IUnknown) -> HRESULT;

    fn SetCurrentSession(&self, pInfo: *mut IUnknown) -> HRESULT;

    fn SetCurrentNextSession(&self) -> HRESULT;

    fn Refresh(&self, hwnd: HWND) -> HRESULT;

    fn Update(
        &self,
        fEnabled: bool,
        hwnd: HWND,
        dwPID: u32,
        unk: u64,
        pSource: IUnknown,
    ) -> HRESULT;

    fn RegisterEventHandler(
        &self,
        pEventHandler: *mut IUnknown, // TODO
        pToken: *mut EventRegistrationToken,
    ) -> HRESULT;

    fn UnregisterEventHandler(&self, token: EventRegistrationToken) -> HRESULT;
}

// >= 14393
#[windows_interface::interface("431268CF-7477-4285-950B-6F892A944712")]
unsafe trait INowPlayingSession: IUnknown {
    fn get_SessionType(&self, pType: *mut NowPlayingSessionType) -> HRESULT;

    fn get_SourceAppId(&self, pszSrcAppId: *mut PCWSTR) -> HRESULT;

    fn get_SourceDeviceId(&self, pszSourceDeviceId: *mut PCWSTR) -> HRESULT;

    fn get_RenderDeviceId(&self, pszRenderId: *mut PCWSTR) -> HRESULT;

    fn get_HWND(&self, pHwnd: *mut HWND) -> HRESULT;

    fn get_PID(&self, pdwPID: *mut u32) -> HRESULT;

    fn get_Info(&self, ppInfo: *mut *mut IUnknown) -> HRESULT;

    fn get_Connection(&self, ppUnknown: *mut *mut IUnknown) -> HRESULT;

    fn ActivateMediaPlaybackDataSource(&self, ppMediaControl: *mut IUnknown) -> HRESULT;

    fn BeginInteractionWithSession(&self, ppSessionInteractionToken: *mut IUnknown) -> HRESULT;
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
        let mut ptr = std::ptr::null_mut();
        let res = npsm.get_CurrentSession(&mut ptr);
        dbg!((res, ptr));

        let mut count = 0;
        let mut sessions = std::ptr::null_mut();
        let res = npsm.GetSessions(&mut count, &mut sessions);
        dbg!((count, sessions));

        let session: INowPlayingSession = (*sessions).cast()?;
        let mut st = PCWSTR::null();
        dbg!(session.get_SourceAppId(&mut st));
        println!("{}", st.display());
    };

    Ok(())
}
