use windows::core::*;
use windows::ApplicationModel::Core::{
    CoreApplication, CoreApplicationView, IFrameworkView, IFrameworkViewSource,
};
use windows::Foundation::AsyncStatus;
use windows::Graphics::{Capture::*, DirectX::Direct3D11::*, DirectX::*, DisplayId};
use windows::Win32::Graphics::{Direct3D::*, Direct3D11::*, Dxgi::*};
use windows::Win32::System::{Com::*, WinRT::Direct3D11::*};
use windows::UI::Core::*;

use windows as Windows;

pub fn create_device() -> Result<IDirect3DDevice> {
    let mut ppdevice: Option<ID3D11Device> = None;
    let feature_levels = [
        D3D_FEATURE_LEVEL_11_0,
        D3D_FEATURE_LEVEL_11_1,
        D3D_FEATURE_LEVEL_12_0,
        D3D_FEATURE_LEVEL_12_1,
        D3D_FEATURE_LEVEL_12_2,
    ];

    let r = unsafe {
        D3D11CreateDevice(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            None,
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            feature_levels.as_ptr(),
            5,
            D3D11_SDK_VERSION,
            &mut ppdevice,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
        .unwrap()
    };

    let mut device: ID3D11Device = ppdevice.unwrap();
    let mut dxgi_device_ptr: *mut IDXGIDevice = std::ptr::null_mut();
    dxgi_device_ptr = std::ptr::addr_of_mut!(device) as *mut IDXGIDevice;
    let dxgi_device = unsafe { &*dxgi_device_ptr };

    //let dxgi_device: IDXGIDevice = unsafe { core::mem::transmute(&ppdevice.unwrap()) };
    unsafe { CreateDirect3D11DeviceFromDXGIDevice(dxgi_device).map(|d| core::mem::transmute(d)) }
}
