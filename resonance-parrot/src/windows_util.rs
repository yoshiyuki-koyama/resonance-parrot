use bindings::Windows::Win32::System::SystemServices::*;
use bindings::Windows::Foundation::Numerics::*;
use bindings::Windows::Win32::UI::WindowsAndMessaging::*;
use bindings::Windows::Win32::UI::DisplayDevices::*;
use bindings::Windows::Win32::Graphics::Direct2D::*;
use bindings::Windows::Win32::Graphics::Dxgi::*;
#[allow(unused_imports)]
use windows::HRESULT;

use std::rc::Rc;
use std::convert::TryFrom;

use super::error::*;


#[allow(dead_code)]
#[derive(Clone, Copy)]
#[derive(PartialEq)]
#[repr(usize)]
pub enum AnchorConer {
    LeftTop,
    RightTop,
    LeftBottom,
    RightBottom,
    HcenterTop,
    HcenterBottom,
    LeftVcenter,
    RightVcenter,
}



pub const COLOR_DAWN:D2D1_COLOR_F = D2D1_COLOR_F{r:0.85, g:0.8, b:1.0, a:1.0};
pub const COLOR_DIM :D2D1_COLOR_F = D2D1_COLOR_F{r:0.0, g:0.0, b:0.15, a:1.0};
pub const COLOR_DIM_LIGHT :D2D1_COLOR_F = D2D1_COLOR_F{r:0.15, g:0.15, b:0.5, a:1.0};
pub const COLOR_RED :D2D1_COLOR_F = D2D1_COLOR_F{r:1.0, g:0.0, b:0.0, a:1.0};
pub const COLOR_BLUE :D2D1_COLOR_F = D2D1_COLOR_F{r:0.0, g:0.0, b:1.0, a:1.0};
pub const COLOR_GREY :D2D1_COLOR_F = D2D1_COLOR_F{r:0.7, g:0.7, b:0.7, a:1.0};

pub fn pwstr_to_string( pwstr: &PWSTR)->RpResult<String> {
    if pwstr.0.is_null() {
        return Err(ResonanceParrotError::new("Error: PWSTR is NULL"));
    }
    let mut pwstr_end: *mut u16 = pwstr.0;
    let mut pwstr_size: usize = 0;
    unsafe {
        loop {
            if *pwstr_end == 0 {
                break;
            }
            else {
                pwstr_end = pwstr_end.add(1);
                pwstr_size+=1;
            }
        }
        let u16_str: &[u16] = std::slice::from_raw_parts(pwstr.0, pwstr_size);
        Ok(String::from_utf16_lossy(u16_str))
    }
}


pub unsafe fn get_client_size(hwnd: HWND)  -> RpResult<D2D_SIZE_U> {
    let mut rect: RECT = RECT::default();
    if GetClientRect(hwnd, &mut rect).0 == 0 {
        return Err(ResonanceParrotError::new("Error: GetClientRect is Failed"));
    }
    Ok(D2D_SIZE_U {
        width: u32::try_from(rect.right - rect.left)?,
        height: u32::try_from(rect.bottom - rect.top)?,
    })
}

pub fn rect_itof(rect: &RECT) -> D2D_RECT_F {
    D2D_RECT_F {
        left: f64::from(rect.left) as f32,
        right: f64::from(rect.right) as f32,
        top: f64::from(rect.top) as f32,
        bottom: f64::from(rect.bottom) as f32,
    }
}

pub fn rect_to_fwidth_fheight(rect: &RECT) -> RpResult<(f32, f32)> {
    if rect.left != 0 || rect.top != 0 {
        return Err(ResonanceParrotError::new("Error: Rect is not start (0,0)"));
    }
    Ok((f64::from(rect.right) as f32, f64::from(rect.bottom) as f32))
}

pub fn eq_rect(rect1: &RECT, rect2: &RECT) -> bool {
    if rect1.left == rect2.left && rect1.right == rect2.right && rect1.top == rect2.top && rect1.bottom == rect2.bottom {
        true
    }
    else{
        false
    }
}

pub unsafe fn create_hwnd_render_target(hwnd: HWND, rc_op_factory: Rc<Option<ID2D1Factory>>) -> RpResult<Option<ID2D1HwndRenderTarget>> {
    let mut op_render_target: Option<ID2D1HwndRenderTarget> = None;
    let target_properties = D2D1_RENDER_TARGET_PROPERTIES::default();
    if let Some(factory) = &*rc_op_factory {
        factory.CreateHwndRenderTarget(
            &target_properties,
            &D2D1_HWND_RENDER_TARGET_PROPERTIES {
                hwnd: hwnd, pixelSize: get_client_size(hwnd)?, presentOptions: D2D1_PRESENT_OPTIONS_IMMEDIATELY,
            },
            &mut op_render_target,
        ).ok()?;
    }
    Ok(op_render_target)
}

pub unsafe fn create_dc_render_target(rc_op_factory: Rc<Option<ID2D1Factory>>) -> RpResult<Option<ID2D1DCRenderTarget>> {
    let mut op_render_target: Option<ID2D1DCRenderTarget> = None;
    let mut target_properties = D2D1_RENDER_TARGET_PROPERTIES::default();
    target_properties.pixelFormat = D2D1_PIXEL_FORMAT{ format: DXGI_FORMAT_B8G8R8A8_UNORM, alphaMode: D2D1_ALPHA_MODE_IGNORE };
    if let Some(factory) = &*rc_op_factory {
        factory.CreateDCRenderTarget(
            &target_properties,
            &mut op_render_target,
        ).ok()?;
    }
    Ok(op_render_target)
}


pub unsafe fn create_hwnd_solid_brush(op_render_target: &Option<ID2D1HwndRenderTarget>, color: &D2D1_COLOR_F) -> RpResult<ID2D1SolidColorBrush> {
    let mut op_brush: Option<ID2D1SolidColorBrush> = None;
    if let Some(render_target) = op_render_target {
        render_target.CreateSolidColorBrush(
            color,
            &D2D1_BRUSH_PROPERTIES {
                opacity: 1.0,
                transform: Matrix3x2::default(),
            },
            &mut op_brush,
        ).ok()?;
    }
    if let Some(brush) = op_brush {
        return Ok(brush);
    }
    else {
        return Err(ResonanceParrotError::new("Error: CreateSolidColorBrush is failed"));
    }
}

pub unsafe fn create_dc_solid_brush(op_render_target: &Option<ID2D1DCRenderTarget>, color: &D2D1_COLOR_F) -> RpResult<ID2D1SolidColorBrush> {
    let mut op_brush: Option<ID2D1SolidColorBrush> = None;
    if let Some(render_target) = op_render_target {
        render_target.CreateSolidColorBrush(
            color,
            &D2D1_BRUSH_PROPERTIES {
                opacity: 1.0,
                transform: Matrix3x2::default(),
            },
            &mut op_brush,
        ).ok()?;
    }
    if let Some(brush) = op_brush {
        return Ok(brush);
    }
    else {
        return Err(ResonanceParrotError::new("Error: CreateSolidColorBrush is failed"));
    }
}
