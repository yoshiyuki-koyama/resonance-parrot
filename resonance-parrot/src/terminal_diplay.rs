use std::io::{Write, BufWriter, stderr};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::convert::TryFrom;

use super::error::*;
extern crate resonance;
use resonance::{SPN_LABEL, SPN_NUM, SpnIdx};

fn f64_to_u32(f64_val:f64) -> u32 {
    if f64_val < 0.0 {
        u32::MIN
    }
    else if f64::from(u32::MAX) < f64_val {
        u32::MAX
    }
    else {
        f64_val.round() as u32
    }
}


#[derive(Clone)]
#[derive(PartialEq)]
enum DisplayEventType {
    Open,
    ChangeRange,
    UpdateValue,
    Close,
    Exit,
}

struct NoteRange {
    stt_idx: usize,
    end_idx: usize,
}

struct InputInfo {
    name: String,
    sampling_rate: usize,
    bits: usize,
    ch_num: usize,
}

pub struct DisplayEvent {
    event: DisplayEventType,
    time_idx: Option<usize>,
    sound_vec_asrc: Option<Arc<Vec<Vec<f64>>>>,
    spectrum_vec_arc: Option<Arc<Vec<Vec<Vec<f64>>>>>,
    abs_range: Option<NoteRange>,
    rel_range: Option<isize>,
    input_info: Option<InputInfo>,
}

impl DisplayEvent {
    pub fn open( name: String, sampling_rate: usize, bits: usize, ch_num: usize) -> Result<DisplayEvent> {
        Ok(DisplayEvent {
            event: DisplayEventType::Open,
            time_idx: Some(0),
            sound_vec_asrc: None,
            spectrum_vec_arc: None,
            abs_range: Some(NoteRange{
                stt_idx:usize::try_from(SpnIdx::A2 as i32)?,
                end_idx:usize::try_from(SpnIdx::A5 as i32)?+1
            }),
            rel_range: None,
            input_info: Some(InputInfo{name: name, sampling_rate: sampling_rate, bits: bits, ch_num: ch_num}),
        })
    }
    pub fn change_abs_range(lowest_note: SpnIdx, highest_note: SpnIdx) -> Result<DisplayEvent> {
        Ok(DisplayEvent {
            event: DisplayEventType::ChangeRange,
            time_idx: None,
            sound_vec_asrc: None,
            spectrum_vec_arc: None,
            abs_range: Some(NoteRange{
                stt_idx:usize::try_from(lowest_note as i32)?,
                end_idx:usize::try_from(highest_note as i32)?+1
            }),
            rel_range: None,
            input_info: None,
        })
    }
    pub fn change_rel_range(rel_range: isize) -> DisplayEvent {
        DisplayEvent {
            event: DisplayEventType::ChangeRange,
            time_idx: None,
            sound_vec_asrc: None,
            spectrum_vec_arc: None,
            abs_range: None,
            rel_range: Some(rel_range),
            input_info: None,
        }
    }
    pub fn update_value(time_idx: usize, sound_vec_asrc: Arc<Vec<Vec<f64>>>, spectrum_vec_arc: Arc<Vec<Vec<Vec<f64>>>>) -> DisplayEvent {
        DisplayEvent {
            event: DisplayEventType::UpdateValue,
            time_idx: Some(time_idx),
            sound_vec_asrc: Some(sound_vec_asrc),
            spectrum_vec_arc: Some(spectrum_vec_arc),
            abs_range: None,
            rel_range: None,
            input_info: None,
        }
    }
    pub fn close() -> DisplayEvent {
        DisplayEvent {
            event: DisplayEventType::Close,
            time_idx: None,
            sound_vec_asrc: None,
            spectrum_vec_arc: None,
            abs_range: None,
            rel_range: None,
            input_info: None,
        }
    }
    pub fn exit() -> DisplayEvent {
        DisplayEvent {
            event: DisplayEventType::Close,
            time_idx: None,
            sound_vec_asrc: None,
            spectrum_vec_arc: None,
            abs_range: None,
            rel_range: None,
            input_info: None,
        }
    }
}



#[derive(Clone)]
#[derive(PartialEq)]
enum TerminalStatus {
    Opened,
    Closed,
}

#[allow(dead_code)]
pub struct ContentsStatus {
    input_info: InputInfo,
    time_idx: usize,
    vbar_meter_sound: Vec<VbarMeter>,
    vbar_meter_spectrum: Vec<VbarMeter>,
    range: NoteRange,
}

#[allow(dead_code)]
pub struct TerminalDisplay {
    vertical_pos: usize,
    horizontal_pos: usize,
    vertical_home_pos: usize,
    horizontal_home_pos: usize,
    string: String,
    status: TerminalStatus,
    contents: ContentsStatus
}




#[allow(dead_code)]
impl TerminalDisplay {
    pub fn new() -> Result<TerminalDisplay> {
        Ok(TerminalDisplay {
            vertical_pos:0,
            horizontal_pos:0,
            vertical_home_pos:0,
            horizontal_home_pos:0,
            string: String::with_capacity(2000),
            status: TerminalStatus::Closed,
            contents: ContentsStatus {
                input_info: InputInfo{name: String::new(), sampling_rate: 0, bits: 0, ch_num: 0},
                time_idx: 0,
                vbar_meter_sound: Vec::new(),
                vbar_meter_spectrum: Vec::new(),
                range: NoteRange {
                    stt_idx:usize::try_from(SpnIdx::A3 as i32)?,
                    end_idx:usize::try_from(SpnIdx::A4 as i32)?+1
                }
            }   
        })
    }

    pub fn push_one_line(&mut self, string: String) {
        self.string.push_str(&string);
        self.string.push('\n');
        self.vertical_pos += 1;
    }

    pub fn print_and_flush(&mut self)  -> Result<()> {
        let stderr = stderr();
        let stderr_lock = stderr.lock();
        let mut buf_writer = BufWriter::new(stderr_lock);
        buf_writer.write_all(self.string.as_bytes())?;
        std::io::stderr().flush()?;
        self.string.clear();
        Ok(())
    }

    pub fn display_one_line (&mut self, string: String) -> Result<()> {
        println!("{}", string);
        std::io::stderr().flush()?;
        self.vertical_pos += 1;
        Ok(())
    }

    pub fn display_lines (&mut self, string: String) -> Result<()> {
        print!("{}", string);
        std::io::stderr().flush()?;
        self.vertical_pos += string.lines().count();
        Ok(())
    }

    pub fn set_current_pos_as_home(&mut self) {
        self.vertical_home_pos = self.vertical_pos;
        self.horizontal_home_pos = self.horizontal_pos;
    }

    pub fn set_home_pos(&mut self, vertical_pos: usize, horizontal_pos: usize) {
        self.vertical_home_pos = vertical_pos;
        self.horizontal_home_pos = horizontal_pos;
    }


    pub fn back_to_home_line(&mut self) -> Result<()> {
        self.back_to_the_line(self.vertical_home_pos)
    }

    pub fn back_to_the_line(&mut self, vertical_pos: usize) -> Result<()> {
        while self.vertical_pos > vertical_pos {
            self.string.push_str("\u{001B}[1A");
            self.vertical_pos -= 1;
        }
        self.print_and_flush()
    }

    pub fn back_to_initial_line (&mut self) -> Result<()> {
        self.back_to_the_line(0)
    }

    pub fn erase_display_from_cusor_to_end (&mut self) -> Result<()> {
        self.string.push_str("\u{001B}[0J");
        self.print_and_flush()
    }

    pub fn erase_display (&mut self) -> Result<()> {
        self.string.push_str("\u{001B}[2J");
        self.print_and_flush()
    }

}

#[allow(dead_code)]
pub struct VbarMeter {
    label: String,
    min: f64,
    max: f64,
    divsor: u32,
    interval: f64,
    yellow_level:u32,
    red_level: u32,
}

impl VbarMeter {
    fn new( label: String, label_len: usize, min: f64, max: f64, divsor: u32,  
        op_yellow_value: Option<f64>, op_red_value: Option<f64>)
        -> Result<VbarMeter> {
        let f64_divsor : f64 =  f64::from(divsor);
        if min >= max {
            return Err(ResonanceParrotError::new("Set Value Error in VbarMeter !"));
        }
        
        let mut yellow_level = divsor;
        let mut red_level = divsor;
        let interval = (max - min) / f64_divsor;
        if let Some(value) = op_yellow_value {
            if value >= min {
                let f64_level = (value - min) / interval;
                yellow_level =  f64_to_u32(f64_level);
                if f64_level.floor() != f64_level.ceil() {
                    yellow_level +=  1;
                }
            }
            else {
                return Err(ResonanceParrotError::new("Set Value Error in VbarMeter !"));
            }
        }
        if let Some(value) = op_red_value {
            if value >= min {
                let f64_level = (value - min) / interval;
                red_level = f64_to_u32(f64_level);
                if f64_level.floor() != f64_level.ceil() {
                    red_level +=  1;
                }
            }
            else {
                return Err(ResonanceParrotError::new("Set Value Error in VbarMeter !"));
            }
        }
        let mut adjusted_len_label = label.clone();
        if adjusted_len_label.chars().count() > label_len {
            while adjusted_len_label.chars().count() < label_len {
                adjusted_len_label.pop();
            }
        }
        else {
            while adjusted_len_label.chars().count() < label_len {
                adjusted_len_label.push(' ');
            }
        }
        
        Ok(VbarMeter {
            label: adjusted_len_label,
            min: min,
            max: max,
            divsor: divsor,
            interval: interval,
            yellow_level:yellow_level,
            red_level: red_level,
        })
    }

    fn update_label(&mut self, label: String, label_len: usize) {
        let mut adjusted_len_label = label.clone();
        if adjusted_len_label.len() > label_len {
            adjusted_len_label.truncate(label_len);
        }
        else {
            while adjusted_len_label.len() < label_len {
                adjusted_len_label.push(' ');
            }
        }
        self.label = adjusted_len_label;
    }
    
    fn set_value(&self, value: f64) -> String {
        let mut vbar_string = String::new();
        let values_level = (value - self.min) / self.interval;
        vbar_string.push_str("\u{001B}[37m"); // white
        vbar_string.push_str(&self.label);
        vbar_string.push('[');
        vbar_string.push_str("\u{001B}[32m"); // green
        for level in 0..self.divsor {
            if level == self.yellow_level {
                vbar_string.push_str("\u{001B}[33m"); // yellow
            }
            if level == self.red_level {
                vbar_string.push_str("\u{001B}[31m"); // red
            }
            if level <= f64_to_u32(values_level) {
                vbar_string.push('|');
            }
            else {
                vbar_string.push(' ');
            }
        }
        vbar_string.push_str("\u{001B}[37m"); // white
        vbar_string.push(']');
        vbar_string.push_str("\u{001B}[0m"); // reset
        vbar_string
    }
}

fn set_vbar_meter_ch_label(ch_idx: usize, ch_len: usize) -> String {
    match ch_len {
        1 => { r"  Mono:".to_string() },
        2 => {
            match ch_idx {
                0 => r"/ L_ch :".to_string(),
                _ => r"\ R_ch :".to_string()
            }
        }
        _ =>  {
            let mut vbar_label = match ch_idx {
                0 => r"/ ".to_string(),
                n if (n == ch_len-1) => r"\ ".to_string(),
                _ =>  r"| ".to_string(),
            };
            vbar_label.push_str(&(ch_idx+1).to_string());
            vbar_label.push_str("_ch :");
            vbar_label
        }
    }
}

fn set_vbar_meter_spectrum_label(idx:usize, freq_len:usize, spn_label:&str) -> String {
    let bar_label = match idx {
        0 => r"\".to_string(),
        n if (n == freq_len-1) => r"/".to_string(),
        _ =>  r"|".to_string(),
    };
    format!("{} {}:", bar_label, spn_label)
}

fn reset_vbar(terminal :&mut TerminalDisplay) -> Result<()>{
    // Sound
    terminal.contents.vbar_meter_sound = Vec::new();
    for ch_idx in 0..terminal.contents.input_info.ch_num {
        let vbar_label = set_vbar_meter_ch_label(ch_idx, terminal.contents.input_info.ch_num);
        let vbar_meter = VbarMeter::new(vbar_label, 10, 0.0, 1.0, 40, Some(0.6), Some(0.8))?;
        terminal.contents.vbar_meter_sound.push(vbar_meter);
    }
    // Spectrum
    let range = &terminal.contents.range;
    terminal.contents.vbar_meter_spectrum = Vec::new();
    for _ in 0..terminal.contents.input_info.ch_num {
        for label_idx in (0..range.end_idx-range.stt_idx).rev() { // Freq Bar is Reversed
            let vbar_label = set_vbar_meter_spectrum_label(label_idx, range.end_idx-range.stt_idx, SPN_LABEL[range.stt_idx+label_idx]);
            let vbar_meter = VbarMeter::new(vbar_label, 10, 0.0, 0.005, 40, Some(0.6), Some(0.8))?;
            terminal.contents.vbar_meter_spectrum.push(vbar_meter);
        }
    }
    Ok(())
}

fn push_time_display(terminal :&mut TerminalDisplay) -> Result<()> {
    if terminal.contents.input_info.sampling_rate == 0 {
        return Err(ResonanceParrotError::new("Display SamplingRate is 0!"));
    }
    let time_idx = terminal.contents.time_idx;
    let sampling = terminal.contents.input_info.sampling_rate;
    terminal.push_one_line(format!("  Time {:02}:{:02}.{:02}",time_idx/60/sampling, time_idx/sampling%60, time_idx*100/sampling%100));
    Ok(())
}

fn print_blank_vbar(terminal :&mut TerminalDisplay) -> Result<()>{
    terminal.back_to_home_line()?;
    push_time_display(terminal)?;
    for idx in 0..terminal.contents.vbar_meter_sound.len() {
        terminal.push_one_line(terminal.contents.vbar_meter_sound[idx].set_value(0.0));
    }
    terminal.push_one_line("".to_string());
    for idx in 0..terminal.contents.vbar_meter_spectrum.len() {
        terminal.push_one_line(terminal.contents.vbar_meter_spectrum[idx].set_value(0.0));
    }
    terminal.print_and_flush()?;
    terminal.back_to_home_line()?;
    Ok(())
}

fn display_main(to_display_receiver: Receiver<DisplayEvent>) ->  Result<bool> {
    let mut terminal = TerminalDisplay::new()?;

    loop {
        // Set block start & end index
        let display_event = to_display_receiver.recv()?;
        match display_event.event {
            DisplayEventType::Open => {
                if terminal.status != TerminalStatus::Closed {
                    return Err(ResonanceParrotError::new("Display Open when Status is Opened!"));
                }
                
                // Check & Copy Event to Contents
                if display_event.input_info.is_none() {
                    return Err(ResonanceParrotError::new("Display Open with No Infomation!"));
                }
                terminal.contents.input_info = display_event.input_info.unwrap();
                if display_event.abs_range.is_none() {
                    return Err(ResonanceParrotError::new("Display Open with No Range!"));
                }
                terminal.contents.range = display_event.abs_range.unwrap();
                if display_event.time_idx.is_none() {
                    return Err(ResonanceParrotError::new("Display Open with No Time Idx!"));
                }
                terminal.contents.time_idx = display_event.time_idx.unwrap();

                terminal.erase_display()?;
                terminal.push_one_line(terminal.contents.input_info.name.clone());
                terminal.push_one_line(format!("  Sampling Rate:{}  Bits/Sample:{}", terminal.contents.input_info.sampling_rate, terminal.contents.input_info.bits));
                terminal.print_and_flush()?;
                terminal.set_current_pos_as_home();

                reset_vbar(&mut terminal)?;
                print_blank_vbar(&mut terminal)?;
                terminal.status = TerminalStatus::Opened;
            }
            DisplayEventType::ChangeRange => {
                if terminal.status == TerminalStatus::Closed {
                    return Err(ResonanceParrotError::new("Display ChangeRange when Status is Closed!"));
                }
                // Check & Copy Event to Contents
                if let Some(abs_range) = display_event.abs_range {
                    terminal.contents.range = abs_range;
                }
                else {
                    if let Some(rel_range) = display_event.rel_range {
                        if rel_range > 0 {
                            if terminal.contents.range.end_idx + usize::try_from(rel_range.abs())? <= SPN_NUM {
                                terminal.contents.range.stt_idx += usize::try_from(rel_range.abs())?;
                                terminal.contents.range.end_idx += usize::try_from(rel_range.abs())?;
                            }
                        }
                        else {
                            if terminal.contents.range.stt_idx >= usize::try_from(rel_range.abs())? {
                                terminal.contents.range.stt_idx -= usize::try_from(rel_range.abs())?;
                                terminal.contents.range.end_idx -= usize::try_from(rel_range.abs())?;
                            }
                        }
                    }
                    else{
                        return Err(ResonanceParrotError::new("Display ChangeRange with No Range!"));
                    }
                }
                
                reset_vbar(&mut terminal)?;
                print_blank_vbar(&mut terminal)?;
            }
            DisplayEventType::UpdateValue => {
                if terminal.status == TerminalStatus::Closed {
                    return Err(ResonanceParrotError::new("Display UpdateValue when Status is Closed!"));
                }

                // Check & Copy Event to Contents
                if display_event.time_idx.is_none() {
                    return Err(ResonanceParrotError::new("Display Open with No Time Idx!"));
                }
                terminal.contents.time_idx = display_event.time_idx.unwrap();
                if display_event.sound_vec_asrc.is_none() {
                    return Err(ResonanceParrotError::new("Display Open with No Sound Data!"));
                }
                if display_event.spectrum_vec_arc.is_none() {
                    return Err(ResonanceParrotError::new("Display Open with No Spectrum Data!"));
                }
                
                // tmp value
                let sound_vec_asrc = display_event.sound_vec_asrc.unwrap();
                let spectrum_vec_arc = display_event.spectrum_vec_arc.unwrap();

                terminal.back_to_home_line()?;
                // Time Display
                push_time_display(&mut terminal)?;
                // Extract Max Value in Data Block
                for (ch_idx, ch) in sound_vec_asrc.iter().enumerate() {
                    let mut max = 0.0;
                    for data_idx in 0..ch.len() {
                        let tmp = ch[data_idx].abs();
                        if tmp > max {
                            max = tmp;
                        }
                    }
                    terminal.push_one_line(terminal.contents.vbar_meter_sound[ch_idx].set_value(max));
                }

                terminal.push_one_line("".to_string());

                // channel<freq<data<energy>>>
                for (ch_idx, ch) in spectrum_vec_arc.iter().enumerate() {
                    // Freq Bar is Reversed
                    for (freq_idx, freq) in ch[terminal.contents.range.stt_idx..terminal.contents.range.end_idx].iter().rev().enumerate() {
                        let mut max = 0.0;
                        for data_idx in 0..freq.len() {
                            let tmp = freq[data_idx].abs();
                            if tmp > max {
                                max = tmp;
                            }
                        }
                        terminal.push_one_line(terminal.contents.vbar_meter_spectrum[freq_idx + ch_idx*(terminal.contents.range.end_idx-terminal.contents.range.stt_idx)].set_value(max));
                    }
                }
                terminal.print_and_flush()?;
            },
            DisplayEventType::Close => {
                //todo!() contents reset??
                terminal.status = TerminalStatus::Closed;
                break;
            },
            DisplayEventType::Exit => {
                break;
            },
            //_ => {},
        }
    }
    terminal.back_to_initial_line()?;
    terminal.erase_display_from_cusor_to_end()?;

    Ok(true)
}

pub fn display_thread(to_display_receiver: Receiver<DisplayEvent>) -> AtomicBool {  
    match display_main(to_display_receiver) {
         Ok(bool_ret) => {
             if bool_ret {
                 AtomicBool::new(true)
             }
             else {
                 AtomicBool::new(false)
             }
         }
         Err(err) => {
             println!("Error! : {}", err);
             AtomicBool::new(false)
         }
     }
}