use std::sync::atomic::{AtomicBool, Ordering};

static mut COLOR_SUPPORTED: AtomicBool = AtomicBool::new(false);

pub fn load_supported_colors() {
    if let Some(support) = supports_color::on(supports_color::Stream::Stdout) {
        if support.has_basic {
            unsafe { COLOR_SUPPORTED.store(true, Ordering::Relaxed) }
        }
    }
}

fn supports() -> bool {
    unsafe { COLOR_SUPPORTED.load(Ordering::Relaxed) }
}

macro_rules! color {
    ($name:ident, $value:expr) => {
        #[allow(non_camel_case_types)]
        pub struct $name;
        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                if supports() {
                    f.write_str($value)
                } else {
                    Ok(())
                }
            }
        }
    };
}

macro_rules! colors {
        ($($name:ident = $value:expr);*) => {
            $(
            color!($name, $value);
            )*
        };
    }

colors! {
     s_bold = "\x1B[1m";
     s_underline = "\x1B[4m";
     s_reset = "\x1B[0m";

     c_black = "\x1B[30m";
     c_red = "\x1B[31m";
     c_green = "\x1B[32m";
     c_yellow = "\x1B[33m";
     c_blue = "\x1B[34m";
     c_magenta = "\x1B[35m";
     c_cyan = "\x1B[36m";
     c_white = "\x1B[37m";
     c_bright_black = "\x1B[90m";
     c_bright_red = "\x1B[91m";
     c_bright_green = "\x1B[92m";
     c_bright_yellow = "\x1B[93m";
     c_bright_blue = "\x1B[94m";
     c_bright_magenta = "\x1B[95m";
     c_bright_cyan = "\x1B[96m";
     c_bright_white = "\x1B[97m";
     c_reset = "\x1B[39m";

     bg_black = "\x1B[40m";
     bg_red = "\x1B[41m";
     bg_green = "\x1B[42m";
     bg_yellow = "\x1B[43m";
     bg_blue = "\x1B[44m";
     bg_magenta = "\x1B[45m";
     bg_cyan = "\x1B[46m";
     bg_white = "\x1B[47m";
     bg_bright_black = "\x1B[100m";
     bg_bright_red = "\x1B[101m";
     bg_bright_green = "\x1B[102m";
     bg_bright_yellow = "\x1B[103m";
     bg_bright_blue = "\x1B[104m";
     bg_bright_magenta = "\x1B[105m";
     bg_bright_cyan = "\x1B[106m";
     bg_bright_white = "\x1B[107m";
     bg_reset = "\x1B[49m"
}
