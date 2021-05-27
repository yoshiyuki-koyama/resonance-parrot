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
#[allow(unused_imports)]
use windows::HRESULT;

use std::rc::Rc;
use std::convert::TryFrom;

use super::error::*;
use super::windows_util::*;
use super::{AnchorConer, MainWindow};


#[derive(Clone, Copy)]
#[derive(PartialEq)]
#[repr(usize)]
pub enum ButtonID {
    Play,
    Rec,
    Stop,
    FForword,
    Rewind,
    HZoomIn,
    HZoomOut,
    VZoomIn,
    VZoomOut,
}


pub struct ButtonConstInfo {
    pub id: ButtonID,
    pub  initial_status: BOOL,
    pub anchor: AnchorConer,
    pub x: i32, pub y: i32,
    pub color: D2D1_COLOR_F,
}

pub const BUTTONS :[ButtonConstInfo; 9] = [
    ButtonConstInfo{   id: ButtonID::Play,     initial_status: TRUE,  anchor: AnchorConer::LeftBottom,    x:    10, y:   -60, color: COLOR_DAWN},
    ButtonConstInfo{   id: ButtonID::Rec,      initial_status: TRUE,  anchor: AnchorConer::LeftBottom,    x:    70, y:   -60, color: COLOR_RED,},
    ButtonConstInfo{   id: ButtonID::Stop,     initial_status: TRUE,  anchor: AnchorConer::LeftBottom,    x:   130, y:   -60, color: COLOR_DAWN},
    ButtonConstInfo{   id: ButtonID::Rewind,   initial_status: TRUE,  anchor: AnchorConer::LeftBottom,    x:   190, y:   -60, color: COLOR_DAWN},
    ButtonConstInfo{   id: ButtonID::FForword, initial_status: TRUE,  anchor: AnchorConer::LeftBottom,    x:   250, y:   -60, color: COLOR_DAWN},
    ButtonConstInfo{   id: ButtonID::HZoomIn,  initial_status: TRUE,  anchor: AnchorConer::HcenterTop,    x:   -60, y:    30, color: COLOR_DAWN},
    ButtonConstInfo{   id: ButtonID::HZoomOut, initial_status: TRUE,  anchor: AnchorConer::HcenterTop,    x:    10, y:    30, color: COLOR_DAWN},
    ButtonConstInfo{   id: ButtonID::VZoomIn,  initial_status: TRUE,  anchor: AnchorConer::RightVcenter,  x:   -60, y:   -60, color: COLOR_DAWN},
    ButtonConstInfo{   id: ButtonID::VZoomOut, initial_status: TRUE,  anchor: AnchorConer::RightVcenter,  x:   -60, y:    10, color: COLOR_DAWN},
];

pub const BUTTON_WIDTH: i32 = 50;
pub const BUTTON_HEIGHT: i32 = 50;

#[derive(Clone, Copy)]
#[derive(PartialEq)]
#[repr(usize)]
#[allow(dead_code)]
enum ButtonRecMode {
    Rec = 0,
    Pause = 1,
}

#[derive(Clone, Copy)]
#[derive(PartialEq)]
#[repr(usize)]
#[allow(dead_code)]
enum ButtonPlayMode {
    Play = 0,
    Pause = 1,
}

#[repr(C)]
#[derive(Clone)]
pub struct ButtonInfo{
    pub hwnd: HWND,
    pub const_info: &'static ButtonConstInfo,
    pub op_render_target: Option<ID2D1DCRenderTarget>,
    pub brush_vec: Vec<ID2D1SolidColorBrush>,
    pub mode: usize,
}

#[repr(C)]
#[derive(Clone)]
pub struct ButtonGeometry {
    rc_op_factory:Rc<Option<ID2D1Factory>>,
    rect: RECT,
    play: ID2D1PathGeometry,
    rec: ID2D1EllipseGeometry,
    pause: (ID2D1RectangleGeometry,ID2D1RectangleGeometry),
    stop: ID2D1RectangleGeometry,
    rewind: ID2D1PathGeometry,
    fforword: ID2D1PathGeometry,
    zoomin: (ID2D1EllipseGeometry,ID2D1PathGeometry, ID2D1PathGeometry, ID2D1PathGeometry),
    zoomout: (ID2D1EllipseGeometry,ID2D1PathGeometry, ID2D1PathGeometry),
}

impl  ButtonGeometry {
    pub fn new(rc_op_factory:Rc<Option<ID2D1Factory>>) -> RpResult<ButtonGeometry> {
        let rect = RECT{ top: 0, bottom: BUTTON_HEIGHT, left: 0, right: BUTTON_WIDTH};
        Ok(ButtonGeometry {
            rc_op_factory: Rc::clone(&rc_op_factory),
            rect : rect,
            play    : ButtonGeometry::create_play_geometry(Rc::clone(&rc_op_factory), &rect)?,
            rec     : ButtonGeometry::create_rec_geometry(Rc::clone(&rc_op_factory), &rect)?,
            pause   : ButtonGeometry::create_pause_geometry(Rc::clone(&rc_op_factory), &rect)?,
            stop    : ButtonGeometry::create_stop_geometry(Rc::clone(&rc_op_factory), &rect)?,
            rewind  : ButtonGeometry::create_rewind_geometry(Rc::clone(&rc_op_factory), &rect)?,
            fforword: ButtonGeometry::create_fforword_geometry(Rc::clone(&rc_op_factory), &rect)?,
            zoomin  : ButtonGeometry::create_zoomin_geometry(Rc::clone(&rc_op_factory), &rect)?,
            zoomout : ButtonGeometry::create_zoomout_geometry(Rc::clone(&rc_op_factory), &rect)?,
        })
    }

    fn create_play_geometry(rc_op_factory:Rc<Option<ID2D1Factory>>, rect:&RECT) -> RpResult<ID2D1PathGeometry> {
        unsafe { 
            if let Some(factory) = &*rc_op_factory {
                let mut op_path_geometry: Option<ID2D1PathGeometry> = None;
                factory.CreatePathGeometry(&mut op_path_geometry).ok()?;
                
                if let Some(path_geometry) = op_path_geometry {
                    let mut op_sink:Option<ID2D1GeometrySink> = None;
                    path_geometry.Open(&mut op_sink).ok()?;
                    if let Some(sink) = op_sink {
                        let (width, height) = rect_to_fwidth_fheight(&rect)?;
                        sink.BeginFigure(D2D_POINT_2F{x:width * 0.2, y:height * 0.2} , D2D1_FIGURE_BEGIN_FILLED);
                        sink.AddLine(D2D_POINT_2F{x:width * 0.2, y:height * 0.8});
                        sink.AddLine(D2D_POINT_2F{x:width * 0.8, y:height * 0.5});
                        sink.EndFigure(D2D1_FIGURE_END_CLOSED);
                        sink.Close().ok()?;
                        return Ok(path_geometry);
                    }
                }
            }
        }
        return Err(ResonanceParrotError::new("Error: Create Play Geometry is failed"));
    }

    fn create_rec_geometry(rc_op_factory:Rc<Option<ID2D1Factory>>, rect:&RECT) -> RpResult<ID2D1EllipseGeometry> {
        unsafe { 
            if let Some(factory) = &*rc_op_factory {
                let (width, height) = rect_to_fwidth_fheight(&rect)?;
                let mut op_ellipse_geometry: Option<ID2D1EllipseGeometry> = None;
                factory.CreateEllipseGeometry(
                    &D2D1_ELLIPSE{ point: D2D_POINT_2F{x: width*0.5, y: height*0.5}, radiusX: width*0.3, radiusY: width*0.3},
                    &mut op_ellipse_geometry
                ).ok()?;
                
                if let Some(ellipse_geometry) = op_ellipse_geometry {
                    return Ok(ellipse_geometry);
                }
            }
        }
        return Err(ResonanceParrotError::new("Error: Create Rec Geometry is failed"));
    }

    fn create_pause_geometry(rc_op_factory:Rc<Option<ID2D1Factory>>, rect:&RECT) -> RpResult<(ID2D1RectangleGeometry,ID2D1RectangleGeometry)> {
        unsafe { 
            if let Some(factory) = &*rc_op_factory {
                
                let (width, height) = rect_to_fwidth_fheight(&rect)?;
                // Create First Rect
                let rect_f = D2D_RECT_F{left :width * 0.2, top: height * 0.2, right: width * 0.4, bottom: height * 0.8};
                let mut op_rectangle_geometry1: Option<ID2D1RectangleGeometry> = None;
                factory.CreateRectangleGeometry(&rect_f,&mut op_rectangle_geometry1).ok()?;
                // Create Second Rect
                let rect_f = D2D_RECT_F{left :width * 0.6, top: height * 0.2, right: width * 0.8, bottom: height * 0.8};
                let mut op_rectangle_geometry2: Option<ID2D1RectangleGeometry> = None;
                factory.CreateRectangleGeometry(&rect_f,&mut op_rectangle_geometry2).ok()?;

                
                if let Some(rectandle_geometry1) = op_rectangle_geometry1 {
                    if let Some(rectandle_geometry2) = op_rectangle_geometry2 {
                        return Ok((rectandle_geometry1,rectandle_geometry2));
                    }
                }
            }
        }
        return Err(ResonanceParrotError::new("Error: Create Pause Geometry is failed"));
    }

    fn create_stop_geometry(rc_op_factory:Rc<Option<ID2D1Factory>>, rect:&RECT) -> RpResult<ID2D1RectangleGeometry> {
        unsafe { 
            if let Some(factory) = &*rc_op_factory {
                let (width, height) = rect_to_fwidth_fheight(&rect)?;
                let rect_f = D2D_RECT_F{left :width * 0.2, top: height * 0.2, right: width * 0.8, bottom: height * 0.8};
                let mut op_rectangle_geometry: Option<ID2D1RectangleGeometry> = None;
                factory.CreateRectangleGeometry(&rect_f,&mut op_rectangle_geometry).ok()?;

                
                if let Some(rectandle_geometry) = op_rectangle_geometry {
                    return Ok(rectandle_geometry);
                }
            }
        }
        return Err(ResonanceParrotError::new("Error: Create Stop Geometry is failed"));
    }

    fn create_rewind_geometry(rc_op_factory:Rc<Option<ID2D1Factory>>, rect:&RECT) -> RpResult<ID2D1PathGeometry> {
        unsafe { 
            if let Some(factory) = &*rc_op_factory {
                let mut op_path_geometry: Option<ID2D1PathGeometry> = None;
                factory.CreatePathGeometry(&mut op_path_geometry).ok()?;
                
                if let Some(path_geometry) = op_path_geometry {
                    let mut op_sink:Option<ID2D1GeometrySink> = None;
                    path_geometry.Open(&mut op_sink).ok()?;
                    if let Some(sink) = op_sink {
                        let (width, height) = rect_to_fwidth_fheight(&rect)?;
                        sink.BeginFigure(D2D_POINT_2F{x:width * 0.5, y:height * 0.5} , D2D1_FIGURE_BEGIN_FILLED);
                        sink.AddLine(D2D_POINT_2F{x:width * 0.8, y:height * 0.2});
                        sink.AddLine(D2D_POINT_2F{x:width * 0.8, y:height * 0.8});
                        sink.AddLine(D2D_POINT_2F{x:width * 0.5, y:height * 0.5});
                        sink.AddLine(D2D_POINT_2F{x:width * 0.5, y:height * 0.8});
                        sink.AddLine(D2D_POINT_2F{x:width * 0.2, y:height * 0.5});
                        sink.AddLine(D2D_POINT_2F{x:width * 0.2, y:height * 0.8});
                        sink.AddLine(D2D_POINT_2F{x:width * 0.1, y:height * 0.8});
                        sink.AddLine(D2D_POINT_2F{x:width * 0.1, y:height * 0.2});
                        sink.AddLine(D2D_POINT_2F{x:width * 0.2, y:height * 0.2});
                        sink.AddLine(D2D_POINT_2F{x:width * 0.2, y:height * 0.5});
                        sink.AddLine(D2D_POINT_2F{x:width * 0.5, y:height * 0.2});
                        sink.AddLine(D2D_POINT_2F{x:width * 0.5, y:height * 0.5});
                        sink.EndFigure(D2D1_FIGURE_END_CLOSED);
                        sink.Close().ok()?;
                        return Ok(path_geometry);
                    }
                }
            }
        }
        return Err(ResonanceParrotError::new("Error: Create Play Geometry is failed"));
    }

    fn create_fforword_geometry(rc_op_factory:Rc<Option<ID2D1Factory>>, rect:&RECT) -> RpResult<ID2D1PathGeometry> {
        unsafe { 
            if let Some(factory) = &*rc_op_factory {
                let mut op_path_geometry: Option<ID2D1PathGeometry> = None;
                factory.CreatePathGeometry(&mut op_path_geometry).ok()?;
                
                if let Some(path_geometry) = op_path_geometry {
                    let mut op_sink:Option<ID2D1GeometrySink> = None;
                    path_geometry.Open(&mut op_sink).ok()?;
                    if let Some(sink) = op_sink {
                        let (width, height) = rect_to_fwidth_fheight(&rect)?;
                        sink.BeginFigure(D2D_POINT_2F{x:width * 0.5, y:height * 0.5} , D2D1_FIGURE_BEGIN_FILLED);
                        sink.AddLine(D2D_POINT_2F{x:width * 0.2, y:height * 0.2});
                        sink.AddLine(D2D_POINT_2F{x:width * 0.2, y:height * 0.8});
                        sink.AddLine(D2D_POINT_2F{x:width * 0.5, y:height * 0.5});
                        sink.AddLine(D2D_POINT_2F{x:width * 0.5, y:height * 0.8});
                        sink.AddLine(D2D_POINT_2F{x:width * 0.8, y:height * 0.5});
                        sink.AddLine(D2D_POINT_2F{x:width * 0.8, y:height * 0.8});
                        sink.AddLine(D2D_POINT_2F{x:width * 0.9, y:height * 0.8});
                        sink.AddLine(D2D_POINT_2F{x:width * 0.9, y:height * 0.2});
                        sink.AddLine(D2D_POINT_2F{x:width * 0.8, y:height * 0.2});
                        sink.AddLine(D2D_POINT_2F{x:width * 0.8, y:height * 0.5});
                        sink.AddLine(D2D_POINT_2F{x:width * 0.5, y:height * 0.2});
                        sink.AddLine(D2D_POINT_2F{x:width * 0.5, y:height * 0.5});
                        sink.EndFigure(D2D1_FIGURE_END_CLOSED);
                        sink.Close().ok()?;
                        return Ok(path_geometry);
                    }
                }
            }
        }
        return Err(ResonanceParrotError::new("Error: Create Play Geometry is failed"));
    }

    fn create_zoomout_geometry(rc_op_factory:Rc<Option<ID2D1Factory>>, rect:&RECT) -> RpResult<(ID2D1EllipseGeometry,ID2D1PathGeometry, ID2D1PathGeometry)> {
        unsafe { 
            if let Some(factory) = &*rc_op_factory {
                let (width, height) = rect_to_fwidth_fheight(&rect)?;
                // Create magnifying glass
                let glass_center = D2D_POINT_2F{x: width*0.6, y: height*0.4};


                let mut op_ellipse_geometry: Option<ID2D1EllipseGeometry> = None;
                factory.CreateEllipseGeometry(
                    &D2D1_ELLIPSE{ point: glass_center, radiusX: width*0.3, radiusY: width*0.3},
                    &mut op_ellipse_geometry
                ).ok()?;

                // Create magnifying glass arm
                let mut op_path_geometry_arm: Option<ID2D1PathGeometry> = None;
                factory.CreatePathGeometry(&mut op_path_geometry_arm).ok()?;
                
                if let Some(path_geometry_arm) = &op_path_geometry_arm {
                    let mut op_sink:Option<ID2D1GeometrySink> = None;
                    path_geometry_arm.Open(&mut op_sink).ok()?;
                    if let Some(sink) = op_sink {
                        sink.BeginFigure(D2D_POINT_2F{x: glass_center.x - width*0.3*2.0_f32.powf(-0.5), y: glass_center.y + height*0.3*2.0_f32.powf(-0.5)} , D2D1_FIGURE_BEGIN_HOLLOW);
                        sink.AddLine(D2D_POINT_2F{x:width*0.1, y:height*0.9});
                        sink.EndFigure(D2D1_FIGURE_END_OPEN);
                        sink.Close().ok()?;
                    }
                    else {
                        return Err(ResonanceParrotError::new("Error: Create Zoom Geometry is failed"));
                    }
                }
                else{
                    return Err(ResonanceParrotError::new("Error: Create Zoom Geometry is failed"));
                }

                // Create Hbar of Plus
                let mut op_path_geometry_hbar: Option<ID2D1PathGeometry> = None;
                factory.CreatePathGeometry(&mut op_path_geometry_hbar).ok()?;

                if let Some(path_geometry_hbar) = &op_path_geometry_hbar {
                    let mut op_sink:Option<ID2D1GeometrySink> = None;
                    path_geometry_hbar.Open(&mut op_sink).ok()?;
                    if let Some(sink) = op_sink {
                        sink.BeginFigure(D2D_POINT_2F{x: glass_center.x - width*0.2, y: glass_center.y} , D2D1_FIGURE_BEGIN_HOLLOW);
                        sink.AddLine(D2D_POINT_2F{x:glass_center.x + width*0.2, y: glass_center.y});
                        sink.EndFigure(D2D1_FIGURE_END_OPEN);
                        sink.Close().ok()?;
                    }
                    else {
                        return Err(ResonanceParrotError::new("Error: Create Zoom Geometry is failed"));
                    }
                }
                else{
                    return Err(ResonanceParrotError::new("Error: Create Zoom Geometry is failed"));
                }
                
                if let Some(ellipse_geometry) = op_ellipse_geometry {
                    if let Some(path_geometry_arm) = op_path_geometry_arm {
                        if let Some(path_geometry_hbar) = op_path_geometry_hbar {
                            return Ok((ellipse_geometry, path_geometry_arm, path_geometry_hbar));
                        }
                    }
                }
            }
        }
        return Err(ResonanceParrotError::new("Error: Create Zoom Geometry is failed"));
    }

    fn create_zoomin_geometry(rc_op_factory:Rc<Option<ID2D1Factory>>, rect:&RECT) -> RpResult<(ID2D1EllipseGeometry,ID2D1PathGeometry, ID2D1PathGeometry, ID2D1PathGeometry)> {
        let zuumout_tuple = Self::create_zoomout_geometry(Rc::clone(&rc_op_factory), rect)?;
        if let Some(factory) = &*rc_op_factory {
            let (width, height) = rect_to_fwidth_fheight(&rect)?;
            // Create magnifying glass
            let glass_center = D2D_POINT_2F{x: width*0.6, y: height*0.4};
            unsafe { 
                // Create Vbar of Plus
                let mut op_path_geometry_vbar: Option<ID2D1PathGeometry> = None;
                factory.CreatePathGeometry(&mut op_path_geometry_vbar).ok()?;

                if let Some(path_geometry_vbar) = &op_path_geometry_vbar {
                    let mut op_sink:Option<ID2D1GeometrySink> = None;
                    path_geometry_vbar.Open(&mut op_sink).ok()?;
                    if let Some(sink) = op_sink {
                        sink.BeginFigure(D2D_POINT_2F{x: glass_center.x, y: glass_center.y - height * 0.2} , D2D1_FIGURE_BEGIN_HOLLOW);
                        sink.AddLine(D2D_POINT_2F{x:glass_center.x, y: glass_center.y + height * 0.2});
                        sink.EndFigure(D2D1_FIGURE_END_OPEN);
                        sink.Close().ok()?;
                    }
                    else {
                        return Err(ResonanceParrotError::new("Error: Create Zoom Geometry is failed"));
                    }   
                }
                else{
                    return Err(ResonanceParrotError::new("Error: Create Zoom Geometry is failed"));
                }
                if let Some(path_geometry_vbar) = op_path_geometry_vbar {
                    return Ok((zuumout_tuple.0,zuumout_tuple.1,zuumout_tuple.2, path_geometry_vbar));
                }
            }
        }
        return Err(ResonanceParrotError::new("Error: Create Zoom Geometry is failed"));
    }

    fn update_geometry(&mut self, rect:&RECT) -> RpResult<()> {
        if eq_rect(&self.rect, rect) {
            return Ok(());
        }
        else {
            self.rect = rect.clone();
            self.play  = Self::create_play_geometry(Rc::clone(&self.rc_op_factory), &self.rect)?;
            self.rec   = Self::create_rec_geometry(Rc::clone(&self.rc_op_factory), &self.rect)?;
            self.pause = Self::create_pause_geometry(Rc::clone(&self.rc_op_factory), &self.rect)?;
            self.stop = Self::create_stop_geometry(Rc::clone(&self.rc_op_factory), &self.rect)?;
            self.rewind = Self::create_rewind_geometry(Rc::clone(&self.rc_op_factory), &self.rect)?;
            self.fforword = Self::create_fforword_geometry(Rc::clone(&self.rc_op_factory), &self.rect)?;
            self.zoomin = Self::create_zoomin_geometry(Rc::clone(&self.rc_op_factory), &self.rect)?;
            self.zoomout = Self::create_zoomout_geometry(Rc::clone(&self.rc_op_factory), &self.rect)?;
        }
        return Err(ResonanceParrotError::new("Error: RegisterClass is failed"));
    }
}

pub fn on_draw_button(button: &mut ButtonInfo, op_button_geometry: &mut Option<ButtonGeometry>, p_drawstruct:isize) -> RpResult<()> {
    unsafe {
        // Create Render_Target and Brush
        if let Some(button_geomtry) = op_button_geometry {
            if button.op_render_target.is_none() {
                button.op_render_target = create_dc_render_target(Rc::clone(&button_geomtry.rc_op_factory))?;
                button.brush_vec.push(create_dc_solid_brush(&button.op_render_target, &button.const_info.color)?);  // 0 : Enable
                button.brush_vec.push(create_dc_solid_brush(&button.op_render_target, &COLOR_GREY)?); // 1 : Disable
                button.brush_vec.push(create_dc_solid_brush(&button.op_render_target, &COLOR_BLUE)?); // 2 : Focus
            }
    
            // Paint & Draw
            if let Some(render_target) = &button.op_render_target {
                let hdc = (*(p_drawstruct as *const isize as *const DRAWITEMSTRUCT)).hDC;
                let rect= (*(p_drawstruct as *const isize as *const DRAWITEMSTRUCT)).rcItem;
                render_target.BindDC(hdc, &rect).ok()?;
                button_geomtry.update_geometry(&rect)?;

                println!("itemState {}", (*(p_drawstruct as *const isize as *const DRAWITEMSTRUCT)).itemState);

                let item_state = (*(p_drawstruct as *const isize as *const DRAWITEMSTRUCT)).itemState;
                let brush_idx: usize;
                let bg_color: D2D1_COLOR_F;
                // Set Brush
                if item_state & ODS_DISABLED != 0 { brush_idx = 1;}
                else { brush_idx = 0;}
                // Set Back-Ground Color
                if item_state & ODS_SELECTED != 0 { bg_color = COLOR_DIM_LIGHT; button.mode =1;}
                else { bg_color = COLOR_DIM; button.mode =0;}


                render_target.BeginDraw();
                render_target.Clear(&bg_color);
                // Draw Foucus Andle
                if item_state & ODS_FOCUS != 0 {
                    render_target.DrawRectangle(
                        &rect_itof(&rect),
                        &button.brush_vec[2],
                        3.0,
                        None,
                    )
                }
            
                match button.const_info.id {
                    ButtonID::Play =>{
                        if button.mode == ButtonPlayMode::Play as usize {
                            render_target.FillGeometry(&button_geomtry.play, &button.brush_vec[brush_idx], None);
                        }
                        else {
                            render_target.FillGeometry(&button_geomtry.pause.0, &button.brush_vec[brush_idx], None);
                            render_target.FillGeometry(&button_geomtry.pause.1, &button.brush_vec[brush_idx], None);
                        }
                    }   
                    ButtonID::Rec =>{
                        if button.mode == ButtonRecMode::Rec as usize {
                            render_target.FillGeometry(&button_geomtry.rec, &button.brush_vec[brush_idx], None);
                        }
                        else {
                            render_target.FillGeometry(&button_geomtry.pause.0, &button.brush_vec[brush_idx], None);
                            render_target.FillGeometry(&button_geomtry.pause.1, &button.brush_vec[brush_idx], None);
                        }
                    }    
                    ButtonID::Stop =>{
                        render_target.FillGeometry(&button_geomtry.stop, &button.brush_vec[brush_idx], None);
                    }
                    ButtonID::Rewind=>{
                        render_target.FillGeometry(&button_geomtry.rewind, &button.brush_vec[brush_idx], None);
                    } 
                    ButtonID::FForword =>{
                        render_target.FillGeometry(&button_geomtry.fforword, &button.brush_vec[brush_idx], None);
                    }
                    id if id == ButtonID::HZoomIn || id == ButtonID::VZoomIn =>{
                        render_target.DrawGeometry(&button_geomtry.zoomin.0, &button.brush_vec[brush_idx], 3.0, None);
                        render_target.DrawGeometry(&button_geomtry.zoomin.1, &button.brush_vec[brush_idx], 3.0, None);
                        render_target.DrawGeometry(&button_geomtry.zoomin.2, &button.brush_vec[brush_idx], 3.0, None);
                        render_target.DrawGeometry(&button_geomtry.zoomin.3, &button.brush_vec[brush_idx], 3.0, None);
                    }
                    id if id == ButtonID::HZoomOut || id == ButtonID::VZoomOut =>{
                        render_target.DrawGeometry(&button_geomtry.zoomout.0, &button.brush_vec[brush_idx], 3.0, None);
                        render_target.DrawGeometry(&button_geomtry.zoomout.1, &button.brush_vec[brush_idx], 3.0, None);
                        render_target.DrawGeometry(&button_geomtry.zoomout.2, &button.brush_vec[brush_idx], 3.0, None);
                    }
                    _ => {}
                }
                
                let mut tag: (u64, u64) = (0,0);
                let h_result = render_target.EndDraw(&mut tag.0, &mut tag.1);
                if h_result.is_err() {
                    if h_result == D2DERR_RECREATE_TARGET {
                        button.op_render_target = None;
                    }
                }
            }
        }
        Ok(())
    }
}

