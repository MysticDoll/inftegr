use windows::core::*;
use windows::Graphics::{Capture::*, DirectX::Direct3D11::*, DirectX::*, Imaging::{
    SoftwareBitmap,
    BitmapEncoder,
    BitmapDecoder,
    BitmapBounds
}};
use windows::Win32::Foundation::*;
use windows::Win32::System::Com::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Media::Ocr::OcrEngine;
use windows::Storage::Streams::InMemoryRandomAccessStream;

use crate::directx::*;

mod directx;

#[tokio::main]
async fn main() -> Result<()> {
    unsafe {
        CoInitializeEx(std::ptr::null_mut(), COINIT_MULTITHREADED)?;
    }

    println!("foreground window will be captured");
    let _ = tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    let hwnd = unsafe { GetForegroundWindow() };

    if hwnd == HWND(0) {
        panic!("couldn't found target window handle");
    }

    let d3device: IDirect3DDevice = create_device()?;

    let interop = windows::core::factory::<
        GraphicsCaptureItem,
        windows::Win32::System::WinRT::Graphics::Capture::IGraphicsCaptureItemInterop,
    >()?;
    println!("create_item");
    let gcitem: GraphicsCaptureItem = unsafe { interop.CreateForWindow(*&hwnd)? };
    println!("create_item_done");

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

    let ocr_engine = OcrEngine::TryCreateFromUserProfileLanguages().unwrap();

    loop {
        if let Ok(iaops_bitmap) = frame_pool.TryGetNextFrame()
            .and_then(|frame| frame.Surface())
            .and_then(|surface| SoftwareBitmap::CreateCopyFromSurfaceAsync(&surface)) {
                if let Ok(bitmap) = iaops_bitmap.await {
                    let trimed_bitmap = trim_software_bitmap(
                        bitmap,
                        0,
                        0,
                        640,
                        720
                    ).await;
                    if let Ok(iaops_ocr_result) = ocr_engine.RecognizeAsync(&trimed_bitmap) {
                        let ocr_result = iaops_ocr_result.await;
                        if let Ok(lines) = ocr_result
                            .and_then(|r| r.Lines())
                            .map(|v| v.into_iter()) {
                                for line in lines {
                                    println!(
                                        "{:?}",
                                        line.Text()
                                            .map(|t| t.to_string())
                                            .unwrap_or("UNKNOWN STRING".to_string())
                                    );
                                }
                            };
                    };
                };
            }

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    Ok(())
}

async fn trim_software_bitmap(bitmap: SoftwareBitmap, x: u32, y: u32, width: u32, height: u32) -> SoftwareBitmap {
    let mut stream = InMemoryRandomAccessStream::new().unwrap();
    let mut encoder = BitmapEncoder::CreateAsync(
        BitmapEncoder::BmpEncoderId().unwrap(),
        &stream
    ).unwrap().await.unwrap();

    encoder.SetSoftwareBitmap(&bitmap).unwrap();
    let transform = encoder.BitmapTransform().unwrap();
    transform.SetBounds(
        BitmapBounds {
            X: x,
            Y: y,
            Width: width,
            Height: height,
        }
    ).unwrap();

    encoder.FlushAsync().unwrap().await;

    let decoder = BitmapDecoder::CreateWithIdAsync(
        BitmapDecoder::BmpDecoderId().unwrap(),
        &stream
    )
        .unwrap()
        .await
        .unwrap();

    decoder.GetSoftwareBitmapAsync().unwrap().await.unwrap()
}
