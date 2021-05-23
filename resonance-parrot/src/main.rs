#[allow(unused_imports)]
use bindings::Windows::Win32::System::SystemServices::*;
#[allow(unused_imports)]
use bindings::Windows::Win32::System::Com::*;
#[allow(unused_imports)]
use bindings::Windows::Win32::System::Diagnostics::Debug::*;
#[allow(unused_imports)]
use bindings::Windows::Foundation::Numerics::*;
#[allow(unused_imports)]
use bindings::Windows::Win32::UI::WindowsAndMessaging::*;
#[allow(unused_imports)]
use bindings::Windows::Win32::UI::MenusAndResources::*;
#[allow(unused_imports)]
use bindings::Windows::Win32::UI::Controls::*;
#[allow(unused_imports)]
use bindings::Windows::Win32::UI::KeyboardAndMouseInput::*;
#[allow(unused_imports)]
use bindings::Windows::Win32::UI::Shell::*;
#[allow(unused_imports)]
use bindings::Windows::Win32::UI::DisplayDevices::*;
#[allow(unused_imports)]
use bindings::Windows::Win32::UI::HiDpi::*;
#[allow(unused_imports)]
use bindings::Windows::Win32::Graphics::Gdi::*;
#[allow(unused_imports)]
use bindings::Windows::Win32::Graphics::Direct2D::*;
#[allow(unused_imports)]
use bindings::Windows::Win32::Graphics::Dxgi::*;
use windows::HRESULT;






use std::{mem::size_of, thread};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::convert::TryFrom;

extern crate wavfile;
use wavfile::*;

extern crate resonance;
use resonance::*;

mod error;
use error::*;

mod timeline;
use timeline::*;

mod terminal_diplay;
use terminal_diplay::*;

mod keyhit_input;
use keyhit_input::*;

#[cfg(test)]
mod tests {
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Track {
    pub file_path: std::path::PathBuf,
    pub format_id: usize,
    pub channel: usize,
    pub sampling_rate: usize,
    pub bits: usize,
    pub ch_vec: Vec<Vec<f64>>,
}

impl Track {
    fn new_from_file( wav_path: &std::path::Path)-> RpResult<Track> {
        let mut wav_file = WavFile::new();
        wav_file.open(wav_path)?;
        let base_wav_audio = wav_file.get_wav_audio()?;
        Ok(Track{
            file_path: wav_path.to_path_buf(),
            format_id : base_wav_audio.fmt.id,
            channel : base_wav_audio.fmt.channel,
            sampling_rate : base_wav_audio.fmt.sampling_rate,
            bits : base_wav_audio.fmt.bits,
            ch_vec : to_channel_vec(&base_wav_audio)?
        })
    }
    fn save(&self) -> RpResult<()> {
        self.saveas(&self.file_path)
    }

    fn saveas(&self, file_path: &std::path::Path) -> RpResult<()> {
        let wav_audio_fmt = Fmt {
            id:self.format_id,
            channel: self.channel,
            sampling_rate: self.sampling_rate,
            bits: self.bits,
        };
        let wav_audio = to_wav_audio(&self.ch_vec, &wav_audio_fmt)?;
        let mut new_file = WavFile::new();
        new_file.update_wav_audio(&wav_audio)?;
        new_file.save_as(file_path)?;
        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct DisplaySpectrum {
    pub span: usize,
    pub labels:&'static[&'static str],
    pub ch_vec: Vec<Vec<Vec<f64>>>, // channel<freq<data<energy>>>
}

#[derive(Clone)]
#[derive(PartialEq)]
pub enum ThreadID {
    TimeCounter,
    Display,
    KeyHit,
}

pub struct AppEvent {
    pub thread_id: ThreadID,
    pub event_id: usize,
}

#[allow(dead_code)]
fn set_gain(ref_data_vec:&mut Vec<i64>, db:&f64) {
    let gain = 10_f64.powf(db / 10_f64);
    for ref_data in ref_data_vec {
        *ref_data = ((*ref_data as f64) * gain) as i64;
    }
}


fn pwstr_to_string( pwstr: &PWSTR)->RpResult<String> {
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

unsafe fn get_client_size(hwnd: HWND)  -> RpResult<D2D_SIZE_U> {
    let mut rect: RECT = RECT::default();
    if GetClientRect(hwnd, &mut rect).0 == 0 {
        return Err(ResonanceParrotError::new("Error: GetClientRect is Failed"));
    }
    Ok(D2D_SIZE_U {
        width: u32::try_from(rect.right - rect.left)?,
        height: u32::try_from(rect.bottom - rect.top)?,
    })
}

//unsafe fn invalidate_client_rect(hwnd: HWND)  -> RpResult<()> {
//    let mut rect: RECT = RECT::default();
//    if GetClientRect(hwnd, &mut rect).0 == 0 {
//        return Err(ResonanceParrotError::new("Error: GetClientRect is Failed"));
//    }
//    if InvalidateRect(hwnd, &rect, FALSE).0 == 0 {
//        return Err(ResonanceParrotError::new("Error: InvalidateRect is Failed"));
//    }
//    Ok(())
//}

unsafe fn create_hwnd_render_target(hwnd: HWND, op_factory: &Option<ID2D1Factory>) -> RpResult<Option<ID2D1HwndRenderTarget>> {
    let mut op_render_target: Option<ID2D1HwndRenderTarget> = None;
    let target_properties = D2D1_RENDER_TARGET_PROPERTIES::default();
    if let Some(factory) = op_factory {
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

unsafe fn create_dc_render_target(hwnd: HWND, op_factory: &Option<ID2D1Factory>) -> RpResult<Option<ID2D1DCRenderTarget>> {
    let mut op_render_target: Option<ID2D1DCRenderTarget> = None;
    let mut target_properties = D2D1_RENDER_TARGET_PROPERTIES::default();
    target_properties.pixelFormat = D2D1_PIXEL_FORMAT{ format: DXGI_FORMAT_B8G8R8A8_UNORM, alphaMode: D2D1_ALPHA_MODE_IGNORE };
    if let Some(factory) = op_factory {
        factory.CreateDCRenderTarget(
            &target_properties,
            &mut op_render_target,
        ).ok()?;
    }
    Ok(op_render_target)
}


unsafe fn create_hwnd_solid_brush(op_render_target: &Option<ID2D1HwndRenderTarget>, color: &D2D1_COLOR_F) -> RpResult<ID2D1SolidColorBrush> {
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

unsafe fn create_dc_solid_brush(op_render_target: &Option<ID2D1DCRenderTarget>, color: &D2D1_COLOR_F) -> RpResult<ID2D1SolidColorBrush> {
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

fn rect_itof(rect: &RECT) -> D2D_RECT_F {
    D2D_RECT_F {
        left: f64::from(rect.left) as f32,
        right: f64::from(rect.right) as f32,
        top: f64::from(rect.top) as f32,
        bottom: f64::from(rect.bottom) as f32,
    }
}

#[derive(Clone, Copy)]
#[derive(PartialEq)]
#[repr(usize)]
enum AnchorConer {
    LeftTop,
    RightTop,
    LeftBottom,
    RightBottom,
    HcenterTop,
    HcenterBottom,
    LeftVcenter,
    RightVcenter,
}

#[derive(Clone, Copy)]
#[derive(PartialEq)]
#[repr(usize)]
enum MenuItemID {
    File,
    Fopen,
    Fclose,
    Save,
    SaveAs,
    Exit,
    Help,
    About,
    Separater,
}
struct MenuItemConstInfo {
    menu_id: MenuItemID,
    hierarchy: usize,
    initial_status: MENU_ITEM_FLAGS,
    name_str: &'static str,
    op_func: Option<fn(&mut MainWindow)->RpResult<()>>,
}


// KEEP THE ORDER.
// The order must be kept in MainWindow::cerate_menu().
// 1) Fist row is first item in menu bar.
// 2) If a item has a submenu, nest row is first item in the submenu.
// 3) If a item has not a submenu and has a next item, next row is the next item.
// 4) If item NEITHER has a submenu NOR a next item, next row is the next item of parent.
const MENU_ITEMS :[MenuItemConstInfo; 9] = [
    MenuItemConstInfo{   menu_id: MenuItemID::File,          hierarchy: 0, initial_status: MF_POPUP,     name_str: "File",            op_func:None,},
    MenuItemConstInfo{   menu_id: MenuItemID::Fopen,         hierarchy: 1, initial_status: MF_ENABLED,   name_str: "Open File",       op_func:Some(MainWindow::on_menu_fopen),},
    MenuItemConstInfo{   menu_id: MenuItemID::Fclose,        hierarchy: 1, initial_status: MF_DISABLED,  name_str: "Close File",      op_func:Some(MainWindow::on_menu_fclose),},
    MenuItemConstInfo{   menu_id: MenuItemID::Save,          hierarchy: 1, initial_status: MF_DISABLED,  name_str: "Save",            op_func:Some(MainWindow::on_menu_save),},
    MenuItemConstInfo{   menu_id: MenuItemID::SaveAs,        hierarchy: 1, initial_status: MF_DISABLED,  name_str: "Save As",         op_func:Some(MainWindow::on_menu_saveas),},
    MenuItemConstInfo{   menu_id: MenuItemID::Separater,     hierarchy: 1, initial_status: MF_SEPARATOR, name_str: "",                op_func:None,},
    MenuItemConstInfo{   menu_id: MenuItemID::Exit,          hierarchy: 1, initial_status: MF_ENABLED,   name_str: "Exit",            op_func:Some(MainWindow::on_menu_exit),},
    MenuItemConstInfo{   menu_id: MenuItemID::Help,          hierarchy: 0, initial_status: MF_POPUP,     name_str: "Help",            op_func:None,},
    MenuItemConstInfo{   menu_id: MenuItemID::About,         hierarchy: 1, initial_status: MF_ENABLED,   name_str: "About",           op_func:Some(MainWindow::on_menu_about),},
];
const MAX_MENU_HIERARYCHY :usize = 2;

#[derive(Clone, Copy)]
#[derive(PartialEq)]
#[repr(usize)]
enum ButtonID {
    PLAY,
    REC,
    STOP,
    FFORWORD,
    REWIND,
    H_ZOOM_IN,
    H_ZOOM_OUT,
    V_ZOOM_IN,
    V_ZOOM_OUT,
}

struct ButtonConstInfo {
    button_id: ButtonID,
    initial_status: BOOL,
    anchor: AnchorConer,
    x: i32, y: i32, width: i32, height: i32,
    push_func: fn(&mut MainWindow)->RpResult<()>,
    paint_func: fn(&ID2D1DCRenderTarget, &ID2D1SolidColorBrush, &RECT)->RpResult<()>,
}

const BUTTONS :[ButtonConstInfo; 9] = [
    ButtonConstInfo{   button_id: ButtonID::PLAY,        initial_status: FALSE, anchor: AnchorConer::LeftBottom,    x:    10, y:   -60, width:   50, height:   50, push_func:MainWindow::on_dummy, paint_func:ButtonInfo::on_dummy,},
    ButtonConstInfo{   button_id: ButtonID::REC,         initial_status: TRUE,  anchor: AnchorConer::LeftBottom,    x:    70, y:   -60, width:   50, height:   50, push_func:MainWindow::on_dummy, paint_func:ButtonInfo::on_paint_rec,},
    ButtonConstInfo{   button_id: ButtonID::STOP,        initial_status: FALSE, anchor: AnchorConer::LeftBottom,    x:   130, y:   -60, width:   50, height:   50, push_func:MainWindow::on_dummy, paint_func:ButtonInfo::on_dummy,},
    ButtonConstInfo{   button_id: ButtonID::FFORWORD,    initial_status: FALSE, anchor: AnchorConer::LeftBottom,    x:   190, y:   -60, width:   50, height:   50, push_func:MainWindow::on_dummy, paint_func:ButtonInfo::on_dummy,},
    ButtonConstInfo{   button_id: ButtonID::REWIND,      initial_status: FALSE, anchor: AnchorConer::LeftBottom,    x:   250, y:   -60, width:   50, height:   50, push_func:MainWindow::on_dummy, paint_func:ButtonInfo::on_dummy,},
    ButtonConstInfo{   button_id: ButtonID::H_ZOOM_IN,   initial_status: TRUE,  anchor: AnchorConer::HcenterTop,    x:   -60, y:    30, width:   50, height:   50, push_func:MainWindow::on_dummy, paint_func:ButtonInfo::on_dummy,},
    ButtonConstInfo{   button_id: ButtonID::H_ZOOM_OUT,  initial_status: TRUE,  anchor: AnchorConer::HcenterTop,    x:    10, y:    30, width:   50, height:   50, push_func:MainWindow::on_dummy, paint_func:ButtonInfo::on_dummy,},
    ButtonConstInfo{   button_id: ButtonID::V_ZOOM_IN,   initial_status: TRUE,  anchor: AnchorConer::RightVcenter,  x:   -60, y:   -60, width:   50, height:   50, push_func:MainWindow::on_dummy, paint_func:ButtonInfo::on_dummy,},
    ButtonConstInfo{   button_id: ButtonID::V_ZOOM_OUT,  initial_status: TRUE,  anchor: AnchorConer::RightVcenter,  x:   -60, y:    10, width:   50, height:   50, push_func:MainWindow::on_dummy, paint_func:ButtonInfo::on_dummy,},
];

const COLOR_DIM :D2D1_COLOR_F = D2D1_COLOR_F{r:0.0, g:0.0, b:0.15, a:1.0};
const COLOR_DIM_LIGHT :D2D1_COLOR_F = D2D1_COLOR_F{r:0.15, g:0.15, b:0.5, a:1.0};
const COLOR_RED :D2D1_COLOR_F = D2D1_COLOR_F{r:1.0, g:0.0, b:0.0, a:1.0};
const COLOR_BLUE :D2D1_COLOR_F = D2D1_COLOR_F{r:0.0, g:0.0, b:1.0, a:1.0};
const COLOR_GREY :D2D1_COLOR_F = D2D1_COLOR_F{r:0.5, g:0.5, b:0.5, a:1.0};

#[repr(C)]
#[derive(Clone)]
struct ButtonInfo{
    hwnd: HWND,
    const_info: &'static ButtonConstInfo,
    op_render_target: Option<ID2D1DCRenderTarget>,
    brush_vec: Vec<ID2D1SolidColorBrush>,
}

impl ButtonInfo{
    #[allow(dead_code)]
    fn on_dummy(render_target: &ID2D1DCRenderTarget, _brush: &ID2D1SolidColorBrush, _rect:&RECT) -> RpResult<()> {
        unsafe{ render_target.Clear(&COLOR_GREY); }
        Ok(())
    }

    fn on_paint_rec(render_target: &ID2D1DCRenderTarget, brush: &ID2D1SolidColorBrush, rect:&RECT) -> RpResult<()> {
        unsafe {
            let width: f32 = f64::from(rect.right - rect.left) as f32;
            let height: f32 = f64::from(rect.bottom - rect.top) as f32;
            render_target.FillEllipse(
                &D2D1_ELLIPSE{ point: D2D_POINT_2F{x: width/2.0, y: height/2.0}, radiusX: width/3.0, radiusY: width/3.0},
                brush
            );
        }
        Ok(())
    }

    fn on_draw( ref_self: &mut Self, op_factory:&Option<ID2D1Factory>, p_drawstruct:isize) -> RpResult<()> {
        unsafe {
            // Create Render_Target and Brush
            if ref_self.op_render_target.is_none() {
                ref_self.op_render_target = create_dc_render_target(ref_self.hwnd, op_factory)?;
                ref_self.brush_vec.push(create_dc_solid_brush(&ref_self.op_render_target, &COLOR_RED)?);  // 0 : Enable
                ref_self.brush_vec.push(create_dc_solid_brush(&ref_self.op_render_target, &COLOR_GREY)?); // 1 : Disable
                ref_self.brush_vec.push(create_dc_solid_brush(&ref_self.op_render_target, &COLOR_BLUE)?); // 2 : Focus
            }
            

            // Paint & Draw
            if let Some(render_target) = &ref_self.op_render_target {
                let hdc = (*(p_drawstruct as *const isize as *const DRAWITEMSTRUCT)).hDC;
                let rect= (*(p_drawstruct as *const isize as *const DRAWITEMSTRUCT)).rcItem;
                render_target.BindDC(hdc, &rect).ok()?;

               
                println!("itemState {}", (*(p_drawstruct as *const isize as *const DRAWITEMSTRUCT)).itemState);

                let item_state = (*(p_drawstruct as *const isize as *const DRAWITEMSTRUCT)).itemState;
                let brush_idx: usize;
                let bg_color: D2D1_COLOR_F;
                // Set Brush
                if item_state & ODS_DISABLED != 0 { brush_idx = 1;}
                else { brush_idx = 0;}
                // Set Back-Ground Color
                if item_state & ODS_SELECTED != 0 { bg_color = COLOR_DIM_LIGHT;}
                else { bg_color = COLOR_DIM;}


                render_target.BeginDraw();
                render_target.Clear(&bg_color);
                // Draw Foucus Andle
                if item_state & ODS_FOCUS != 0 {
                    render_target.DrawRectangle(
                        &rect_itof(&rect),
                        &ref_self.brush_vec[2],
                        3.0,
                        None,
                    )
                }

                let func = ref_self.const_info.paint_func;
                func(render_target, &ref_self.brush_vec[brush_idx], &rect)?;

                
                let mut tag: (u64, u64) = (0,0);
                let h_result = render_target.EndDraw(&mut tag.0, &mut tag.1);
                if h_result.is_err() {
                    if h_result == D2DERR_RECREATE_TARGET {
                        ref_self.op_render_target = None;
                    }
                }
            }
            Ok(())
        }
    }


}


fn calc_button_pos(rect: &RECT, anchor: AnchorConer, rel_x_pos: i32, rel_y_pos: i32) -> (i32,i32) {
    match anchor {
        AnchorConer::LeftTop => { (rect.left + rel_x_pos, rect.top + rel_y_pos) }
        AnchorConer::RightTop => { (rect.right + rel_x_pos, rect.top + rel_y_pos) }
        AnchorConer::LeftBottom => { (rect.left + rel_x_pos, rect.bottom + rel_y_pos) }
        AnchorConer::RightBottom => { (rect.right + rel_x_pos, rect.bottom + rel_y_pos) }
        AnchorConer::HcenterTop => { (rect.left + (rect.right - rect.left)/2 + rel_x_pos, rect.top + rel_y_pos) }
        AnchorConer::HcenterBottom => { (rect.left + (rect.right - rect.left)/2  + rel_x_pos, rect.bottom + rel_y_pos) }
        AnchorConer::LeftVcenter => { (rect.left + rel_x_pos, rect.top + (rect.bottom - rect.top)/2 + rel_y_pos) }
        AnchorConer::RightVcenter => { (rect.right + rel_x_pos, rect.top + (rect.bottom - rect.top)/2 + rel_y_pos) }
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct Sound {
    dummy: u16,
}


#[repr(C)]
#[derive(Clone)]
pub struct MainWindow {
    class_name_vec: Arc<Vec<u16>>,
    class_name: PWSTR,
    window_name: String,
    hwnd: HWND,
    op_sound: Option<Sound>,
    op_track: Option<Track>,
    op_factory: Option<ID2D1Factory>,
    op_render_target: Option<ID2D1HwndRenderTarget>,
    op_brush: Option<ID2D1SolidColorBrush>,
    op_buttons: Option<Vec<ButtonInfo>>,
    op_subwindows: Option<Vec<HWND>>,
}
const MAINWINDOW_WIDTH :i32 = 1200;
const MAINWINDOW_HEIGHT :i32 = 800;


impl MainWindow {
    fn new() -> MainWindow {
        let mut class_name_vec: Vec<u16> = "resonance".encode_utf16().collect::<Vec<u16>>();
        class_name_vec.push(0);
        let class_name:PWSTR = PWSTR(class_name_vec.as_ptr() as *mut u16);

        let window_name: String = String::from("Resonance Parrot");

        MainWindow {
            class_name_vec: Arc::new(class_name_vec),
            class_name: class_name,
            window_name: window_name,
            hwnd: HWND(0),
            op_sound: None,
            op_track: None,
            op_factory: None,
            op_render_target: None,
            op_brush: None,
            op_buttons: None,
            op_subwindows: None,
        }
    }

    fn create(&mut self)-> RpResult<()> {
        unsafe {
            let hinstance = GetModuleHandleW(None);
            let wnd_class = WNDCLASSEXW {
                cbSize: u32::try_from(size_of::<WNDCLASSEXW>())?,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(Self::wndproc),
                hInstance: hinstance,
                lpszClassName: self.class_name,
                ..Default::default()
            };

            let register_class = RegisterClassExW(&wnd_class);
            if register_class == 0 {
                return Err(ResonanceParrotError::new("Error: RegisterClass is failed"));
            }

            let hwnd = CreateWindowExW(
                Default::default(), 
                self.class_name, 
                self.window_name.clone(), 
                WS_VISIBLE | WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN,
                CW_USEDEFAULT, CW_USEDEFAULT,
                MAINWINDOW_WIDTH, MAINWINDOW_HEIGHT,
                None,
                None,
                hinstance,
                self as *mut Self as *mut std::ffi::c_void                
            );
            println!("h_wnd {} self.h_wnd {}", hwnd.0, self.hwnd.0);
            // At this point WM_NCCREATE & WM_CREATE processes are finished.
            if self.hwnd.0 == 0 || self.hwnd.0 != hwnd.0 {
                return Err(ResonanceParrotError::new("Error: Create HWND is failed"));
            }
        }
        Ok(())
    }

    fn show(&self) {
        unsafe {
            ShowWindow(self.hwnd, SW_SHOW);
        }
    }

    fn create_factory(&mut self) -> RpResult<()> {
        unsafe {
            let mut op_factory: Option<ID2D1Factory> = None;
            D2D1CreateFactory(
                D2D1_FACTORY_TYPE_SINGLE_THREADED,
                &windows::Guid::from_values(0x06152247, 0x6f50, 0x465a, [0x92, 0x45, 0x11, 0x8b, 0xfd, 0x3b, 0x60, 0x07]),
                &D2D1_FACTORY_OPTIONS::default(),
                &mut op_factory as *mut Option<ID2D1Factory> as *mut *mut std::ffi::c_void
            ).ok()?;

            if op_factory.is_none() {
                return Err(ResonanceParrotError::new("Error: ID2D1Factory is None"));
            }
            self.op_factory = op_factory;
            Ok(())
        }
    }

    fn create_buttons(&mut self) -> RpResult<()> {
        self.op_buttons = Some(Vec::new());
        let mut rect: RECT = RECT::default();
        unsafe {
            if GetClientRect(self.hwnd, &mut rect).0 == 0 {
                return Err(ResonanceParrotError::new("Error: GetClientRect is Failed"));
            }
        }
        for button_info in BUTTONS.iter() {
            let pos :(i32,i32) = calc_button_pos(&rect, button_info.anchor, button_info.x, button_info.y);
            let button_hwnd:HWND;
            unsafe {
                button_hwnd = CreateWindowExW(
                    Default::default(), 
                    "BUTTON", 
                    "", 
                    WS_VISIBLE | WS_CHILD | WS_TABSTOP | WS_CLIPSIBLINGS | WINDOW_STYLE(BS_OWNERDRAW.0),
                    pos.0, pos.1,
                    button_info.width, button_info.height,
                    self.hwnd,
                    None,
                    HINSTANCE(GetWindowLongPtrW(self.hwnd, GWLP_HINSTANCE)),
                    self as *mut Self as *mut std::ffi::c_void                
                );
            }
            if let Some(bottons) = &mut self.op_buttons {
                bottons.push(ButtonInfo{ hwnd: button_hwnd, const_info: button_info, op_render_target: None, brush_vec: Vec::new()})
            }
            unsafe {
                EnableWindow(button_hwnd, button_info.initial_status);
            }
        }
        Ok(())
    }

    fn create_subwindows(&mut self) -> RpResult<()> {
        self.create_buttons()?;
        Ok(())
    }

    fn create_menu(&mut self) -> RpResult<()> {
        unsafe {
            let mut h_menu_vec: Vec<HMENU> = vec!(HMENU::default(); MAX_MENU_HIERARYCHY);
            let mut is_set_menu_vec: Vec<bool> = vec!(true; MAX_MENU_HIERARYCHY);
            let mut n_pos_vec: Vec<i32> = vec!(0; MAX_MENU_HIERARYCHY);

            h_menu_vec[0] = CreateMenu();
            SetMenu(self.hwnd, h_menu_vec[0]);
            for menu_item in MENU_ITEMS.iter() {
                if menu_item.hierarchy > MAX_MENU_HIERARYCHY {
                    return Err(ResonanceParrotError::new("Error: Create Menu is failed(hierarchy is too large)"));
                }
                is_set_menu_vec[menu_item.hierarchy] = true;
                if menu_item.initial_status == MF_POPUP {
                    if menu_item.hierarchy < MAX_MENU_HIERARYCHY {
                        if !is_set_menu_vec[menu_item.hierarchy+1] {
                            DestroyMenu(h_menu_vec[menu_item.hierarchy+1]);
                        }
                        h_menu_vec[menu_item.hierarchy+1] = CreatePopupMenu();
                        if AppendMenuW(h_menu_vec[menu_item.hierarchy], menu_item.initial_status, usize::try_from(h_menu_vec[menu_item.hierarchy+1].0)?, menu_item.name_str) != FALSE {
                            n_pos_vec[menu_item.hierarchy] += 1;
                            n_pos_vec[menu_item.hierarchy+1] = 0;
                        }
                    }
                    else {
                        return Err(ResonanceParrotError::new("Error: Create Menu is failed"));
                    }
                }
                else {
                    if AppendMenuW(h_menu_vec[menu_item.hierarchy], menu_item.initial_status, menu_item.menu_id as usize, menu_item.name_str) != FALSE {
                        n_pos_vec[menu_item.hierarchy] += 1;
                    }
                }
            }
            for (i,h_menu) in h_menu_vec.iter().enumerate() {
                if !is_set_menu_vec[i] {
                    DestroyMenu(h_menu).ok()?;
                }
            }
        }
        Ok(())
    }


    fn file_open_dialog(&mut self) -> RpResult<()> {
        let fopen_dialog :IFileOpenDialog = windows::create_instance(&FileOpenDialog)?;
        unsafe {
            let mut wav_pwstr_vec = "wav".encode_utf16().collect::<Vec<u16>>();
            wav_pwstr_vec.push(0);
            let mut wav_ext_pwstr_vec = "*.wav".encode_utf16().collect::<Vec<u16>>();
            wav_ext_pwstr_vec.push(0);
            let ext_filter: [COMDLG_FILTERSPEC; 1] = [
                COMDLG_FILTERSPEC{ pszName:PWSTR(wav_pwstr_vec.as_ptr() as *mut u16), pszSpec:PWSTR(wav_ext_pwstr_vec.as_ptr() as *mut u16)},
            ];
            fopen_dialog.SetFileTypes(1, &ext_filter[0]).ok()?;
            let h_result = fopen_dialog.Show(self.hwnd);
            if h_result.is_ok() {
                    let mut op_item: Option<IShellItem> = None;
                    fopen_dialog.GetResult(&mut op_item).ok()?;
                    if let Some(item) = op_item {
                        let mut file_path: PWSTR = PWSTR::default();
                        item.GetDisplayName(SIGDN_FILESYSPATH, &mut file_path).ok()?;

                        let file_path_string = pwstr_to_string(&file_path)?;
                        let track = Track::new_from_file(std::path::Path::new(&file_path_string))?;
                        self.op_track = Some(track);
                        self.set_menu_status(MenuItemID::Fclose, MF_ENABLED)?;
                        self.set_menu_status(MenuItemID::Save, MF_ENABLED)?;
                        self.set_menu_status(MenuItemID::SaveAs, MF_ENABLED)?;


                        CoTaskMemFree(file_path.0 as *mut std::ffi::c_void);
                    }
            }
            else if h_result.0 & 0xffff == ERROR_CANCELLED.0 {
                // Cancelled 
            }
            else {
                h_result.ok()?;
            }
        }
        Ok(())
    }

    fn file_save_dialog(&mut self) -> RpResult<()> {
        let fsave_dialog :IFileSaveDialog = windows::create_instance(&FileSaveDialog)?;
        unsafe {
            let mut wav_pwstr_vec = "wav".encode_utf16().collect::<Vec<u16>>();
            wav_pwstr_vec.push(0);
            let mut wav_ext_pwstr_vec = "*.wav".encode_utf16().collect::<Vec<u16>>();
            wav_ext_pwstr_vec.push(0);
            let ext_filter: [COMDLG_FILTERSPEC; 1] = [
                COMDLG_FILTERSPEC{ pszName:PWSTR(wav_pwstr_vec.as_ptr() as *mut u16), pszSpec:PWSTR(wav_ext_pwstr_vec.as_ptr() as *mut u16)},
            ];
            fsave_dialog.SetFileTypes(1, &ext_filter[0]).ok()?;
            fsave_dialog.SetDefaultExtension(PWSTR(wav_pwstr_vec.as_ptr() as *mut u16)).ok()?;
            let h_result = fsave_dialog.Show(self.hwnd);
            if h_result.is_ok() {
                    let mut op_item: Option<IShellItem> = None;
                    fsave_dialog.GetResult(&mut op_item).ok()?;
                    if let Some(item) = op_item {
                        let mut file_path: PWSTR = PWSTR::default();
                        item.GetDisplayName(SIGDN_FILESYSPATH, &mut file_path).ok()?;
                        let file_path_string = pwstr_to_string(&file_path)?;
                        if let Some(track) = &self.op_track {
                            track.saveas(std::path::Path::new(&file_path_string))?;
                        }
                        else {
                            println!("no file is opened");
                        }
                        CoTaskMemFree(file_path.0 as *mut std::ffi::c_void);
                    }
            }
            else if h_result.0 & 0xffff == ERROR_CANCELLED.0 {
                // Cancelled 
            }
            else {
                h_result.ok()?;
            }
        }
        Ok(())
    }

    fn set_menu_status(&mut self, menu_id: MenuItemID, menu_status: MENU_ITEM_FLAGS) -> RpResult<()> {
        unsafe {
            let h_menu = GetMenu(self.hwnd);
            if menu_status.0 & MF_BYPOSITION.0 != 0 {
                return Err(ResonanceParrotError::new("Error: Do not set MF_BYPOSITION in menu_status"));
            }
            if EnableMenuItem(h_menu, u32::try_from(menu_id as usize)?, menu_status).0 == -1 {
                return Err(ResonanceParrotError::new("Error: menu_id does not exist"));
            }
        }
        Ok(())
    }


    #[allow(dead_code)]
    fn on_dummy( _ref_self: &mut Self) -> RpResult<()> {
        Ok(())
    }

    fn on_menu_fopen( ref_self: &mut Self) -> RpResult<()> {
        ref_self.file_open_dialog()?;
        Ok(())
    }

    fn on_menu_fclose( ref_self: &mut Self) -> RpResult<()> {
        ref_self.op_track = None;
        ref_self.set_menu_status(MenuItemID::Fclose, MF_DISABLED)?;
        ref_self.set_menu_status(MenuItemID::Save, MF_DISABLED)?;
        ref_self.set_menu_status(MenuItemID::SaveAs, MF_DISABLED)?;
        Ok(())
    }

    fn on_menu_save( ref_self: &mut Self) -> RpResult<()> {
        if let Some(track) = &ref_self.op_track {
            track.save()?
        }
        Ok(())
    }

    fn on_menu_saveas( ref_self: &mut Self) -> RpResult<()> {
        ref_self.file_save_dialog()?;
        Ok(())
    }

    fn on_menu_exit( ref_self: &mut Self) -> RpResult<()> {
        unsafe {
            DestroyWindow(ref_self.hwnd);
        }
        Ok(())
    }

    fn on_menu_about( ref_self: &mut Self) -> RpResult<()> {
        unsafe {
            MessageBoxW(ref_self.hwnd, "Version 0.1.0\n©Yoshiyuki Koyama", "Resonance Parrot", MB_OK);
        }
        Ok(())
    }

    fn on_menu(&mut self, menu_id: usize) -> RpResult<()> {
        for menu_item in MENU_ITEMS.iter() {
            if menu_item.menu_id as usize == menu_id {
                if let Some(func) = menu_item.op_func {
                    return func(self)
                }
            }
        }
        Ok(())
    }

    fn on_paint(&mut self) -> RpResult<()> {
        unsafe {
            // Create Render_Target and Brush
            if self.op_render_target.is_none() {
                self.op_render_target = create_hwnd_render_target(self.hwnd, &self.op_factory)?;

            }

            // Paint & Draw
            // let mut ps : PAINTSTRUCT = PAINTSTRUCT::default();
            // BeginPaint(self.hwnd, &mut ps);
            if let Some(render_target) = &self.op_render_target {
                render_target.BeginDraw();
                render_target.Clear(&COLOR_DIM);
                let mut tag: (u64, u64) = (0,0);
                let h_result = render_target.EndDraw(&mut tag.0, &mut tag.1);
                if h_result.is_err() {
                    if h_result == D2DERR_RECREATE_TARGET {
                        self.op_render_target = None;
                    }
                }
            }
            //EndPaint(self.hwnd, &ps);
        }
        Ok(())
    }

    fn on_draw_item(&mut self, p_drawstruct: isize) -> RpResult<()> {
        let hwnd: HWND;
        let hdc: HDC;
        unsafe{
            hwnd = (*(p_drawstruct as *const isize as *const DRAWITEMSTRUCT)).hwndItem;
        }
        if let Some(buttons) = &mut self.op_buttons {
            for button in buttons {
                if hwnd == button.hwnd {
                    ButtonInfo::on_draw(button, &self.op_factory, p_drawstruct)?;
                    print!("button {}, ",button.const_info.button_id as usize);
                    break;
                }
            }
        }
        Ok(())
    }

    fn on_size(&mut self) -> RpResult<()> {
        let mut rect: RECT = RECT::default();
        unsafe {
            if GetClientRect(self.hwnd, &mut rect).0 == 0 {
                return Err(ResonanceParrotError::new("Error: GetClientRect is Failed"));
            }
        }
                    // Adjust position & size of subwindows
        if let Some(buttons) = &self.op_buttons {
            for button in buttons{
                let pos :(i32,i32) = calc_button_pos(&rect, button.const_info.anchor, button.const_info.x, button.const_info.y);
                unsafe {
                    if SetWindowPos(
                        button.hwnd,
                        HWND(0), // Specify Z order
                        pos.0,
                        pos.1,
                        0,
                        0,
                        SWP_NOSIZE | SWP_NOZORDER | SWP_SHOWWINDOW | SWP_NOSENDCHANGING,
                    ).0 == 0 {
                        return Err(ResonanceParrotError::new("Error: SetWindowPos(for) is Failed"));
                    };
                }
            }
        }


        // Resize render_target
        if let Some(render_target) = &self.op_render_target {
            unsafe {
                if render_target.Resize(&get_client_size(self.hwnd)?).is_err() {
                    self.op_render_target = None;
                }
            }
        }
        else{
            return Ok(());
        }
        
        //invalidate_client_rect(self.hwnd)?;
        Ok(())
    }

    extern "system" fn wndproc(hwnd: HWND, message: u32,  wparam: WPARAM, lparam: LPARAM)-> LRESULT {
        let p_self: *mut Self;
        if message == WM_NCCREATE {
            unsafe {
                p_self = (*(lparam.0 as * const CREATESTRUCTW)).lpCreateParams as *mut Self;
                (*p_self).hwnd = hwnd;
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, p_self as isize);
            }
        }
        else {
            unsafe {
                p_self = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Self;
            }
        }
        let ref_self: &mut Self;
        unsafe {
            ref_self = &mut (*p_self);
        }

        match message {
            WM_NCCREATE =>{
                if ref_self.create_menu().is_err() {
                    unsafe {
                        PostQuitMessage(-1);
                    }
                    return LRESULT(0);
                }
                if ref_self.create_subwindows().is_err() {
                    unsafe {
                        PostQuitMessage(-1);
                    }
                    return LRESULT(0);
                }
            }
            WM_CREATE => {
                if ref_self.create_factory().is_err() {
                    unsafe {
                        PostQuitMessage(-1);
                    }
                    return LRESULT(0);
                }
            }
            WM_CLOSE => {
                // 
            }
            WM_DESTROY => {
                unsafe {
                    PostQuitMessage(0);
                }
                return LRESULT(0);
            }
            WM_COMMAND => {
                if wparam.0 & 0xffff0000 == 0 && lparam.0 == 0 { // Command is Menu.
                    match ref_self.on_menu(wparam.0 & 0x0000ffff) {
                        Ok(_) => {},
                        Err(err) => {
                            println!("Error!!! {}",err);
                            unsafe {
                                DestroyWindow(ref_self.hwnd);
                            }
                        }
                    }
                }
            }
            WM_PAINT =>{
                if ref_self.on_paint().is_err() {
                    unsafe {
                        DestroyWindow(ref_self.hwnd);
                    }
                }
            }
            WM_DRAWITEM =>{
                if ref_self.on_draw_item(lparam.0).is_err() {
                    unsafe {
                        DestroyWindow(ref_self.hwnd);
                    }
                }
                println!("WM_DRAWITEM");
            }
            WM_SIZE =>{
                if ref_self.on_size().is_err() {
                    unsafe {
                        DestroyWindow(ref_self.hwnd);
                    }
                }
                println!("WM_SIZE");
            }
            WM_DPICHANGED =>{
                // include WM_SIZE
                println!("WM_DPICHANGED");
            }
            WM_DISPLAYCHANGE => {
                if ref_self.on_size().is_err() {
                    unsafe {
                        DestroyWindow(ref_self.hwnd);
                    }
                }
                println!("WM_DISPLAYCHANGE");
            }
            _ => {

            }
        }


        unsafe {
            DefWindowProcW( hwnd, message, wparam, lparam)
        }
    }
}

fn resonance_parrot() -> RpResult<()> {
    unsafe {
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT(-4));
        CoInitializeEx(std::ptr::null_mut(), COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE).ok()?;
    }

    let mut main_window = MainWindow::new();
    main_window.create()?;
    main_window.show();

    let mut message = MSG::default();
    unsafe {
        while GetMessageW(&mut message, None, 0, 0) != FALSE {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
        CoUninitialize();
    }
    return Ok(());





    let base_track = Track::new_from_file(std::path::Path::new(r"./test.wav"))?;
    
    let (event_sender, event_receiver) = channel::<AppEvent>();

    let time_event = event_sender.clone();
    let (to_timeline_sender, to_timeline_receiver) = channel::<TimelineRequest>();
    let (from_timeline_sender, from_timeline_receiver) = channel::<TimelineReport>();
    let timeline_thread_instanse = thread::spawn(move || 
        timeline_thread(time_event, from_timeline_sender, to_timeline_receiver)
    );

    let (to_display_sender, to_display_receiver) = channel::<DisplayRequest>();
    let display_thread_instanse = thread::spawn(move || 
        display_thread(to_display_receiver)
    );

    let key_event = event_sender.clone();
    let (to_key_sender, to_key_receiver) = channel::<KeyHitRequest>();
    let (from_key_sender, from_key_receiver) = channel::<char>();
    let keyhit_thread_instanse = thread::spawn(move || 
        keyhit_thread(key_event, from_key_sender, to_key_receiver)
    );

    to_display_sender.send(DisplayRequest::open(base_track.file_path.to_string_lossy().to_string(),base_track.sampling_rate,base_track.bits,base_track.ch_vec.len())?)?;
    to_timeline_sender.send(TimelineRequest::open(base_track.ch_vec[0].len(), base_track.sampling_rate, base_track.sampling_rate/100))?;
    let mut resonance = Resonance::new(440.0, base_track.sampling_rate, base_track.ch_vec.len(), 3)?;

    loop {
        let event = event_receiver.recv()?;
        match event.thread_id {
            ThreadID::TimeCounter => {
                let timeline_report = from_timeline_receiver.recv()?;
                
                // Temporary Process
                let mut sound_vec:Vec<Vec<f64>> = Vec::with_capacity(base_track.ch_vec.len());
                for (_ch_idx, ch) in base_track.ch_vec.iter().enumerate() {
                    let data_stt = timeline_report.timeline.time_counter;
                    let data_end;
                    if timeline_report.timeline.time_counter + timeline_report.timeline.base.event_divisor < ch.len() {
                        data_end = timeline_report.timeline.time_counter + timeline_report.timeline.base.event_divisor;
                    }
                    else{
                        data_end = ch.len()
                    }
                    sound_vec.push(ch[data_stt..data_end].to_vec());
                }
                let sound_arc = Arc::new(sound_vec);
                let resonance_vec = resonance.resonance(sound_arc.clone())?;

                let spectrum_arc = Arc::new(resonance_vec);
                to_display_sender.send(DisplayRequest::update_value(timeline_report.timeline.time_counter, sound_arc, spectrum_arc))?;
            },
            ThreadID::KeyHit => {
                let input_char = from_key_receiver.recv()?;
                if input_char == '\x1B' || input_char == 'q' || input_char == 'Q' {
                    break;
                }
                if input_char == 'w' || input_char == 'W' {
                    to_timeline_sender.send(TimelineRequest::play_or_pause())?;
                }
                if input_char == 's' || input_char == 'S' {
                    to_timeline_sender.send(TimelineRequest::stop())?;
                }
                if input_char == 'd' || input_char == 'D' {
                    // Fast Forword
                }
                if input_char == 'a' || input_char == 'A' {
                    // Rewind
                }
                if input_char == 'o' || input_char == 'O' {
                    // File Open
                }
                if input_char == 'm' || input_char == 'M' {
                    // Menu
                }
                if input_char == 'e' || input_char == 'E' {
                    // Shift Range High
                    to_display_sender.send(DisplayRequest::change_rel_range(12))?;
                }
                if input_char == 'c' || input_char == 'C' {
                    // Shift Range Low
                    to_display_sender.send(DisplayRequest::change_rel_range(-12))?;
                }
                to_key_sender.send(KeyHitRequest::Continue)?;
            },
            _ => { /*None*/ }
        }
    }
    print!("\n\n\n");
    print!("Display Thread Close....");
    to_display_sender.send(DisplayRequest::exit())?;
    match  display_thread_instanse.join() {
        Ok(_ret) => {
            println!("Ok!");
        }
        Err(_err) => {
            println!("Error!");
        }
    }
    print!("Timeline Thread Close....");
    to_timeline_sender.send(TimelineRequest::close())?;
    match  timeline_thread_instanse.join() {
        Ok(_ret) => {
            println!("Ok!");
        }
        Err(_err) => {
            println!("Error!");
        }
    }
    print!("Keyhit Thread Close....");
    to_key_sender.send(KeyHitRequest::Exit)?;
    match  keyhit_thread_instanse.join() {
        Ok(_ret) => {
            println!("Ok!");
        }
        Err(_err) => {
            println!("Error!");
        }
    }
    resonance.exit()?;

    let wav_audio_fmt = Fmt {
        id:base_track.format_id,
        channel: base_track.channel,
        sampling_rate: base_track.sampling_rate,
        bits: base_track.bits,
    };
    let wav_audio = to_wav_audio(&base_track.ch_vec, &wav_audio_fmt)?;
    let mut new_file = WavFile::new();
    new_file.update_wav_audio(&wav_audio)?;
    new_file.save_as(std::path::Path::new(r"./new.wav"))?;
    unsafe {
        CoUninitialize();
    }
    Ok(())
}

fn main() {
    match resonance_parrot() {
        Ok(_) => {},
        Err(err) => println!("Error!!! {}",err)
    }
}
