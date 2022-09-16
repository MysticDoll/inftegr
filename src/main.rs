use windows::core::*;
use windows::ApplicationModel::Core::{
    CoreApplication, CoreApplicationView, IFrameworkView, IFrameworkViewSource,
};
use windows::Foundation::*;
use windows::Graphics::{Capture::*, DirectX::Direct3D11::*, DirectX::*, DisplayId, Imaging::SoftwareBitmap};
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::{Direct3D::*, Direct3D11::*, Dxgi::*};
use windows::Win32::System::ProcessStatus::*;
use windows::Win32::System::Threading::*;
use windows::Win32::System::WinRT::*;
use windows::Win32::System::{Com::*, WinRT::Direct3D11::*};
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::UI::Core::*;
use windows::Media::Ocr::OcrEngine;

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use windows as Windows;

use crate::directx::*;

mod directx;

static mut global_hwnd: HWND = HWND(0);

#[tokio::main]
async fn main() -> Result<()> {
    unsafe {
        CoInitializeEx(std::ptr::null_mut(), COINIT_MULTITHREADED)?;
        //RoInitialize(RO_INIT_SINGLETHREADED)?;
        EnumWindows(
            Some(enum_window_proc),
            LPARAM(0)
        );
    }

    let hwnd = unsafe { global_hwnd };

    if hwnd == HWND(0) {
        panic!("couldn't found target window handle");
    }

    //let hwnd = HWND(store
    //    .iter()
    //    .find(|(hwnd, window_text, exec_name)| {
    //        //println!(
    //        //    "HWND: {}, Window Title: {}, Executable: {}",
    //        //    hwnd, window_text, exec_name
    //        //);
    //        exec_name.contains("th185.exe")
    //    })
    //    .unwrap().0);
    //create_window()?;

    let d3device: IDirect3DDevice = create_device()?;

    // let gcitem = GraphicsCaptureItem::TryCreateFromDisplayId(DisplayId { Value: 0 })?;
    let interop = windows::core::factory::<
        GraphicsCaptureItem,
        windows::Win32::System::WinRT::Graphics::Capture::IGraphicsCaptureItemInterop,
    >()?;
    println!("create_item");
    let gcitem: GraphicsCaptureItem = unsafe { interop.CreateForWindow(*&hwnd)? };
    println!("create_item_done");

    /*
    let picker = GraphicsCapturePicker::new()?;

    let iaops: windows::Foundation::IAsyncOperation<GraphicsCaptureItem>= picker.PickSingleItemAsync()?;
    let gcitem: GraphicsCaptureItem = iaops.await;


    */

    println!("create_framepool");
    let frame_pool = Direct3D11CaptureFramePool::Create(
        &d3device,
        DirectXPixelFormat::B8G8R8A8UIntNormalized,
        2,
        (&gcitem).Size()?,
    )
    .unwrap();

    println!("create capture session");
    let session = frame_pool.CreateCaptureSession(&gcitem).unwrap();
    println!("start capture");
    let _r = session.StartCapture();
    println!("capture started");

    let ocrEngine = OcrEngine::TryCreateFromUserProfileLanguages().unwrap();

    loop {
        if let Ok(iaops_bitmap) = frame_pool.TryGetNextFrame()
            .and_then(|frame| frame.Surface())
            .and_then(|surface| SoftwareBitmap::CreateCopyFromSurfaceAsync(&surface)) {
                if let Ok(bitmap) = iaops_bitmap.await {
                    if let Ok(iaops_ocrResult) = ocrEngine.RecognizeAsync(&bitmap) {
                        let ocrResult = iaops_ocrResult.await;
                        if let Ok(lines) = ocrResult
                            .and_then(|r| r.Lines())
                            .map(|v| v.into_iter()) {
                                for line in lines {
                                    println!("{:?}", line.Text());
                                }
                            };
                    };
                };
            }

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    Ok(())
}

extern "system" fn enum_window_proc(
    hwnd: windows::Win32::Foundation::HWND,
    lp: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::BOOL {
    unsafe {
        // let mut store = &mut *(lp as *mut HashSet<(HWND, String, String)>);
        //let mut store: HWND = std::mem::transmute(lp);
        if global_hwnd == HWND(0) {
            let mut pid = 0;
            GetWindowThreadProcessId(hwnd, &mut pid);
            if let Ok(handle) = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid) {
                let mut module = HINSTANCE(0);
                K32EnumProcessModules(handle, &mut module, 1024, std::ptr::null_mut());

                let mut raw_exec_name = [0; 1024];
                let exec_name_len = K32GetModuleFileNameExA(handle, module, &mut raw_exec_name);
                let exec_name = std::str::from_utf8(&raw_exec_name);

                let mut raw_window_text = [0; 1024];
                let window_text_len = GetWindowTextA(hwnd, &mut raw_window_text);
                let window_text = std::str::from_utf8(&raw_window_text);
                //println!(
                //    "HWND: {:?}, Executable: {:?}, TEXT_LEN: {:?}",
                //    hwnd, exec_name, exec_name_len
                //);

                // store.insert((
                //    hwnd.0,
                //    window_text.unwrap_or("unknown").to_owned(),
                //    exec_name.unwrap_or("unknown").to_owned(),
                // ));

                if exec_name.unwrap_or("__unkonwn__").contains("th185.exe") {
                    println!("hwnd: {:?}, exec_name: {:?}", hwnd, exec_name.unwrap_or(""));
                    global_hwnd= hwnd;
                    println!("stored: {:?}", global_hwnd);
                };
            }
        }
    }
    true.into()
}
