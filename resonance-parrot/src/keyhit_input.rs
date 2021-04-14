extern crate kb_getch_sys;
use  kb_getch_sys::*;

//use std::io::{Write, BufWriter, stderr};
use super::error::*;

use std::thread;
use std::time;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Sender, Receiver};

use super::{ThreadID, AppEvent};

#[allow(dead_code)]
pub struct KeyHitInput {
    input_vec: Vec<char>
}

#[derive(Clone)]
#[derive(PartialEq)]
pub enum KeyHitRequest {
    Continue,
    Exit,
}

#[allow(dead_code)]
impl KeyHitInput {
    fn new() -> KeyHitInput {
        KeyHitInput {
            input_vec: Vec::new()
        }
    }

    fn get_following_input(&self, ref_u8_array:&mut [u8; 4], char_size:usize) -> Result<char>{
        for i in 1..char_size {
            if let Some(tmp_i8) = kb_getch_ffi() {
                let tmp_ch = tmp_i8 as u8;
                if tmp_ch & 0xC0 == 0x80 {
                    ref_u8_array[i] = tmp_ch;
                }
                else if char_size == 3 && i == 1 {
                    let ret_char = match tmp_ch {
                        0x48 => '↑',
                        0x50 => '↓',
                        0x4b => '←',
                        0x4d => '→',
                        _ => {
                            return Err(ResonanceParrotError::new(&format!("Irregel Input:0x{:x},0x{:x}", ref_u8_array[0],ref_u8_array[1])));
                        },
                    };
                    return Ok(ret_char);
                }
            }
        }
        if let Some(ret_char) =String::from_utf8_lossy(ref_u8_array).chars().next(){
            Ok(ret_char)
        }
        else {
            return Err(ResonanceParrotError::new(&format!("Irregel Input:0x{:x},0x{:x},0x{:x},0x{:x}", ref_u8_array[0],ref_u8_array[1],ref_u8_array[2],ref_u8_array[3])));
        }
        
    }

    fn cli_key_input(&self) -> Result<Option<char>> {
        let char_size;
        let mut u8_array:[u8; 4] = [0,0,0,0];
        if let Some(tmp_i8) = kb_getch_ffi() {
            u8_array[0] =  tmp_i8 as u8; 
            if u8_array[0] <= 0x7F {  // ASCII 
                char_size = 1;
            }
            else if u8_array[0] & 0xE0 == 0xC0 {
                char_size = 2;
            }
            else if u8_array[0] & 0xF0 == 0xE0 {
                char_size = 3;
            }
            else if u8_array[0] & 0xF8 == 0xF0 {
                char_size = 4;
            }
            else {
                char_size = 0;
                println!("Unregistered Key");
            }
            
            //println!("Input:{}   0x{:x},0x{:x},0x{:x},0x{:x}", char_size, u8_array[0],u8_array[1],u8_array[2],u8_array[3]);
            return Ok(Some(self.get_following_input(&mut u8_array, char_size)?));
        }
        Ok(None)
    }
}

fn keyhit_main(event_sender: Sender<AppEvent>, from_key_sender: Sender<char>, to_keyhit_receiver: Receiver<KeyHitRequest>) -> Result<()> {
    let sleep_duration = time::Duration::new(0, 1000000); // 1ms
    let keyhit_input = KeyHitInput::new();
    let mut event_id = 0;
    loop {
        if let Some(input_char) = keyhit_input.cli_key_input()? {
            event_sender.send(AppEvent{thread_id:ThreadID::KeyHit, event_id:event_id})?;
            event_id += 1;
            from_key_sender.send(input_char)?;
            if to_keyhit_receiver.recv()? == KeyHitRequest::Exit {
                break;
            }
        }
        else {
            thread::sleep(sleep_duration);
        }
    }
    Ok(())
}

pub fn keyhit_thread(event_sender: Sender<AppEvent>, from_key_sender: Sender<char>, to_keyhit_receiver: Receiver<KeyHitRequest>) -> AtomicBool {
    match keyhit_main(event_sender, from_key_sender, to_keyhit_receiver){
        Ok(_) => {
            AtomicBool::new(true)
        }
        Err(err) => {
            println!("Error! : {}", err);
            AtomicBool::new(false)
        }
    }
}