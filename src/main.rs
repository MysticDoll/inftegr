use windows::core::*;
use windows::ApplicationModel::Core::{
    CoreApplication, CoreApplicationView, IFrameworkView, IFrameworkViewSource,
};
use windows::Foundation::*;
use windows::Graphics::{Capture::*, DirectX::Direct3D11::*, DirectX::*, DisplayId};
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::{Direct3D::*, Direct3D11::*, Dxgi::*};
use windows::Win32::System::ProcessStatus::*;
use windows::Win32::System::Threading::*;
use windows::Win32::System::WinRT::*;
use windows::Win32::System::{Com::*, WinRT::Direct3D11::*};
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::UI::Core::*;

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use windows as Windows;

use crate::directx::*;

mod directx;

fn main() -> Result<()> {
    let mut store: HashSet<(HWND, String, String)> = HashSet::new();
    unsafe {
        CoInitializeEx(std::ptr::null_mut(), COINIT_MULTITHREADED)?;
        //RoInitialize(RO_INIT_SINGLETHREADED)?;
        EnumWindows(
            Some(enum_window_proc),
            std::ptr::addr_of_mut!(store) as isize,
        );
    }

    let hwnd = store
        .iter()
        .find(|(hwnd, window_text, exec_name)| {
            //println!(
            //    "HWND: {}, Window Title: {}, Executable: {}",
            //    hwnd, window_text, exec_name
            //);
            exec_name.contains("th185.exe")
        })
        .unwrap()
        .0;
    //create_window()?;

    let d3device: IDirect3DDevice = create_device()?;

    // let gcitem = GraphicsCaptureItem::TryCreateFromDisplayId(DisplayId { Value: 0 })?;
    let interop = windows::core::factory::<
        GraphicsCaptureItem,
        windows::Win32::System::WinRT::Graphics::Capture::IGraphicsCaptureItemInterop,
    >()?;
    println!("create_item");
    let gcitem: GraphicsCaptureItem = unsafe { interop.CreateForWindow(&hwnd)? };
    println!("create_item_done");

    /*
    let picker = GraphicsCapturePicker::new()?;

    let iaops: windows::Foundation::IAsyncOperation<GraphicsCaptureItem>= picker.PickSingleItemAsync()?;
    let gcitem: GraphicsCaptureItem = iaops.await;


    */

    println!("create_framepool");
    let frame_pool = Direct3D11CaptureFramePool::Create(
        d3device,
        DirectXPixelFormat::B8G8R8A8UIntNormalized,
        2,
        (&gcitem).Size()?,
    )
    .unwrap();

    println!("create capture session");
    let session = frame_pool.CreateCaptureSession(gcitem).unwrap();
    println!("start capture");
    let _r = session.StartCapture();
    println!("capture started");

    loop {}

    Ok(())
}

extern "system" fn enum_window_proc(
    hwnd: windows::Win32::Foundation::HWND,
    lp: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::BOOL {
    unsafe {
        let mut store = &mut *(lp as *mut HashSet<(HWND, String, String)>);
        let mut pid = 0;
        GetWindowThreadProcessId(&hwnd, &mut pid);
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid);
        let mut module = [0; 1024];
        K32EnumProcessModules(&handle, module.as_mut_ptr(), 1024, std::ptr::null_mut());
        let mut raw_exec_name = windows::Win32::Foundation::PSTR([0; 1024].as_mut_ptr());
        let exec_name_len = K32GetModuleFileNameExA(&handle, module[0], raw_exec_name, 1024);
        let exec_name = std::str::from_utf8(std::slice::from_raw_parts(
            raw_exec_name.0,
            exec_name_len.try_into().unwrap(),
        ));

        let mut raw_window_text = windows::Win32::Foundation::PSTR([0; 1024].as_mut_ptr());
        let window_text_len = GetWindowTextA(hwnd, raw_window_text, 1024);
        let window_text = std::str::from_utf8(std::slice::from_raw_parts(
            raw_window_text.0,
            window_text_len.try_into().unwrap(),
        ));
        //println!(
        //    "HWND: {:?}, Executable: {:?}, TEXT_LEN: {:?}",
        //    hwnd, exec_name, file_name_len
        //);

        store.insert((
            hwnd,
            window_text.unwrap_or("unknown").to_owned(),
            exec_name.unwrap_or("unknown").to_owned(),
        ));
    }
    true.into()
}
