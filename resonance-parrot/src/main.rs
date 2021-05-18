#[allow(unused_imports)]
use bindings::Windows::Win32::System::SystemServices::*;
#[allow(unused_imports)]
use bindings::Windows::Win32::System::Com::*;
#[allow(unused_imports)]
use bindings::Windows::Win32::System::Diagnostics::Debug::*;
#[allow(unused_imports)]
use bindings::Windows::Win32::UI::WindowsAndMessaging::*;
#[allow(unused_imports)]
use bindings::Windows::Win32::UI::MenusAndResources::*;
#[allow(unused_imports)]
use bindings::Windows::Win32::UI::Shell::*;
#[allow(unused_imports)]
use bindings::Windows::Win32::UI::HiDpi::*;





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

#[derive(Clone, Copy)]
#[derive(PartialEq)]
#[repr(usize)]
enum MenuItemID {
    FILE,
    FOPEN,
    FCLOSE,
    SAVE,
    SAVEAS,
    EXIT,
    HELP,
    ABOUT,
    SEPARATOR,
}
struct MenuItemInfo {
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
const MENU_ITEMS :[MenuItemInfo; 9] = [
    MenuItemInfo{   menu_id: MenuItemID::FILE,          hierarchy: 0, initial_status: MF_POPUP,     name_str: "File",            op_func:None,},
    MenuItemInfo{   menu_id: MenuItemID::FOPEN,         hierarchy: 1, initial_status: MF_ENABLED,   name_str: "Open File",       op_func:Some(MainWindow::on_menu_fopen),},
    MenuItemInfo{   menu_id: MenuItemID::FCLOSE,        hierarchy: 1, initial_status: MF_DISABLED,  name_str: "Close File",      op_func:Some(MainWindow::on_menu_fclose),},
    MenuItemInfo{   menu_id: MenuItemID::SAVE,          hierarchy: 1, initial_status: MF_DISABLED,  name_str: "Save",            op_func:Some(MainWindow::on_menu_save),},
    MenuItemInfo{   menu_id: MenuItemID::SAVEAS,        hierarchy: 1, initial_status: MF_DISABLED,  name_str: "Save As",         op_func:Some(MainWindow::on_menu_saveas),},
    MenuItemInfo{   menu_id: MenuItemID::SEPARATOR,     hierarchy: 1, initial_status: MF_SEPARATOR, name_str: "",                op_func:None,},
    MenuItemInfo{   menu_id: MenuItemID::EXIT,          hierarchy: 1, initial_status: MF_ENABLED,   name_str: "Exit",            op_func:Some(MainWindow::on_menu_exit),},
    MenuItemInfo{   menu_id: MenuItemID::HELP,          hierarchy: 0, initial_status: MF_POPUP,     name_str: "Help",            op_func:None,},
    MenuItemInfo{   menu_id: MenuItemID::ABOUT,         hierarchy: 1, initial_status: MF_ENABLED,   name_str: "About",           op_func:Some(MainWindow::on_menu_about),},
];
const MAX_MENU_HIERARYCHY :usize = 2;

#[repr(C)]
#[derive(Clone)]
pub struct Graphic {
    dummy: u16,
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
    op_subwindows: Option<Graphic>,
    op_sound: Option<Sound>,
    op_track: Option<Track>,
}

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
            op_subwindows: None,
            op_sound: None,
            op_track: None,
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
                WS_VISIBLE | WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT,
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
                        self.set_menu_status(MenuItemID::FCLOSE, MF_ENABLED)?;
                        self.set_menu_status(MenuItemID::SAVE, MF_ENABLED)?;
                        self.set_menu_status(MenuItemID::SAVEAS, MF_ENABLED)?;


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
    fn on_menu_dummy( _ref_self: &mut Self) -> RpResult<()> {
        Ok(())
    }

    fn on_menu_fopen( ref_self: &mut Self) -> RpResult<()> {
        ref_self.file_open_dialog()?;
        Ok(())
    }

    fn on_menu_fclose( ref_self: &mut Self) -> RpResult<()> {
        ref_self.op_track = None;
        ref_self.set_menu_status(MenuItemID::FCLOSE, MF_DISABLED)?;
        ref_self.set_menu_status(MenuItemID::SAVE, MF_DISABLED)?;
        ref_self.set_menu_status(MenuItemID::SAVEAS, MF_DISABLED)?;
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

    fn on_menu( ref_self: &mut Self, menu_id: usize) -> RpResult<()> {
        for menu_item in MENU_ITEMS.iter() {
            if menu_item.menu_id as usize == menu_id {
                if let Some(func) = menu_item.op_func {
                    return func(ref_self)
                }
            }
        }
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
            }
            WM_CREATE => {

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
                    match Self::on_menu(ref_self, wparam.0 & 0x0000ffff) {
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
