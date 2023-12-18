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
     style_bold = "\x1B[1m";
     style_underline = "\x1B[4m";
     style_reset = "\x1B[0m";

     color_black = "\x1B[30m";
     color_red = "\x1B[31m";
     color_green = "\x1B[32m";
     color_yellow = "\x1B[33m";
     color_blue = "\x1B[34m";
     color_magenta = "\x1B[35m";
     color_cyan = "\x1B[36m";
     color_white = "\x1B[37m";
     color_bright_black = "\x1B[90m";
     color_bright_red = "\x1B[91m";
     color_bright_green = "\x1B[92m";
     color_bright_yellow = "\x1B[93m";
     color_bright_blue = "\x1B[94m";
     color_bright_magenta = "\x1B[95m";
     color_bright_cyan = "\x1B[96m";
     color_bright_white = "\x1B[97m";
     color_reset = "\x1B[39m";

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
