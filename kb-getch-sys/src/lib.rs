use std::os::raw::c_char;

#[link(name = "lib_kb_getch", kind = "static")]
extern {
    fn kb_getch() -> c_char;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let mut cnt = 0;
        loop {
            if let Some(tmp_i8) = kb_getch_ffi() {
                if tmp_i8 != 0 {
                    println!("Input is 0x{:x}", tmp_i8 as u8);
                    cnt += 1;
                }
                if cnt >= 10 {
                    break;
                }
            }
        }
    }
}

pub fn kb_getch_ffi() -> Option<i8> {
    unsafe {
        let ch = kb_getch();
        if ch == 0 {
            None
        }
        else {
            Some(ch)
        }
    }
}