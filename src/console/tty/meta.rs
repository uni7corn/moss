use crate::{memory::uaccess::UserCopyable, process::thread_group::Pgid};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WinSize {
    pub ws_row: u16,
    pub ws_col: u16,
    pub ws_xpixel: u16,
    pub ws_ypixel: u16,
}

unsafe impl UserCopyable for WinSize {}

impl Default for WinSize {
    fn default() -> Self {
        Self {
            ws_row: 33,
            ws_col: 136,
            ws_xpixel: 0,
            ws_ypixel: 0,
        }
    }
}

use bitflags::bitflags;

pub type Tcflag = u32;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct TermiosInputFlags: Tcflag {
        const IGNBRK = 0x001;  // Ignore break condition
        const BRKINT = 0x002;  // Signal interrupt on break
        const IGNPAR = 0x004;  // Ignore characters with parity errors
        const PARMRK = 0x008;  // Mark parity and framing errors
        const INPCK  = 0x010;  // Enable input parity check
        const ISTRIP = 0x020;  // Strip 8th bit off characters
        const INLCR  = 0x040;  // Map NL to CR on input
        const IGNCR  = 0x080;  // Ignore CR
        const ICRNL  = 0x100;  // Map CR to NL on input
        const IXANY  = 0x800;  // Any character will restart after stop
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct TermiosControlFlags: u32 {
        const CS8     = 0x00000030; // 8 bits per byte
        const CSTOPB  = 0x00000040; // Send 2 stop bits
        const CREAD   = 0x00000080; // Enable receiver
        const PARENB  = 0x00000100; // Enable parity generation on output and parity checking for input
        const PARODD  = 0x00000200; // Odd parity instead of even
        const HUPCL   = 0x00000400; // Hang up on last close
        const CLOCAL  = 0x00000800; // Ignore modem control lines
        const CBAUDEX = 0x00001000; // Extra baud rates (platform-specific)
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct TermiosOutputFlags: Tcflag {
        const OPOST  = 0x01;  // Perform output processing
        const ONLCR  = 0x04;
        const OCRNL  = 0x08;
        const ONOCR  = 0x10;
        const ONLRET = 0x20;
        const OFILL  = 0x40;
        const OFDEL  = 0x80;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct TermiosLocalFlags: Tcflag {
        const ISIG    = 0x00001;
        const ICANON  = 0x00002;
        const XCASE   = 0x00004;
        const ECHO    = 0x00008;
        const ECHOE   = 0x00010;
        const ECHOK   = 0x00020;
        const ECHONL  = 0x00040;
        const NOFLSH  = 0x00080;
        const TOSTOP  = 0x00100;
        const ECHOCTL = 0x00200;
        const ECHOPRT = 0x00400;
        const ECHOKE  = 0x00800;
        const FLUSHO  = 0x01000;
        const PENDIN  = 0x04000;
        const IEXTEN  = 0x08000;
        const EXTPROC = 0x10000;
    }
}

pub const TCGETS: usize = 0x5401;
pub const TCSETS: usize = 0x5402;
pub const TCSETSW: usize = 0x5403;
pub const TIOCGWINSZ: usize = 0x5413;
pub const TIOCSWINSZ: usize = 0x5414;
pub const TIOCGPGRP: usize = 0x540F;
pub const TIOCSPGRP: usize = 0x5410;
pub const TCGETS2: usize = 0x802c542a;
pub const TCSETS2: usize = 0x402c542b;
pub const TCSETSW2: usize = 0x402c542c;

pub type Cc = u8;

pub const VINTR: usize = 0;
pub const VQUIT: usize = 1;
pub const VERASE: usize = 2;
pub const VKILL: usize = 3;
pub const VEOF: usize = 4;
pub const VTIME: usize = 5;
pub const VMIN: usize = 6;
//pub const VSWTC: usize = 7;
//pub const VSTART: usize = 8;
//pub const VSTOP: usize = 9;
pub const VSUSP: usize = 10;
//pub const VEOL: usize = 11;
//pub const VREPRINT: usize = 12;
//pub const VDISCARD: usize = 13;
//pub const VWERASE: usize = 14;
//pub const VLNEXT: usize = 15;
//pub const VEOL2: usize = 16;

pub const NCCS: usize = 19;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Termios {
    pub c_iflag: TermiosInputFlags,   // input mode flags
    pub c_oflag: TermiosOutputFlags,  // output mode flags
    pub c_cflag: TermiosControlFlags, // control mode flags
    pub c_lflag: TermiosLocalFlags,   // local mode flags
    pub c_line: Cc,                   // line discipline
    pub c_cc: [Cc; NCCS],             // control characters
}

unsafe impl UserCopyable for Termios {}

impl From<Termios2> for Termios {
    fn from(value: Termios2) -> Self {
        Self {
            c_iflag: value.c_iflag,
            c_oflag: value.c_oflag,
            c_cflag: value.c_cflag,
            c_lflag: value.c_lflag,
            c_line: value.c_line,
            c_cc: value.c_cc,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Termios2 {
    pub c_iflag: TermiosInputFlags,   // input mode flags
    pub c_oflag: TermiosOutputFlags,  // output mode flags
    pub c_cflag: TermiosControlFlags, // control mode flags
    pub c_lflag: TermiosLocalFlags,   // local mode flags
    pub c_line: Cc,                   // line discipline
    pub c_cc: [Cc; NCCS],             // control characters
    pub i_speed: u32,                 // input speed
    pub o_speed: u32,                 // output speed
}

unsafe impl UserCopyable for Termios2 {}

impl Termios2 {
    pub(super) fn update_from_termios(&mut self, other: &Termios) {
        self.c_iflag = other.c_iflag;
        self.c_oflag = other.c_oflag;
        self.c_cflag = other.c_cflag;
        self.c_lflag = other.c_lflag;
        self.c_line = other.c_line;
        self.c_cc = other.c_cc;
    }
}

impl Default for Termios2 {
    fn default() -> Self {
        let mut c_cc = [0; NCCS];
        c_cc[VINTR] = 3; // ^C
        c_cc[VQUIT] = 28; // ^\
        c_cc[VERASE] = 127; // DEL
        c_cc[VKILL] = 21; // ^U
        c_cc[VEOF] = 4; // ^D
        c_cc[VMIN] = 1;
        c_cc[VTIME] = 0;
        c_cc[VSUSP] = 26; // ^Z

        Self {
            c_iflag: TermiosInputFlags::ICRNL,
            c_oflag: TermiosOutputFlags::OPOST | TermiosOutputFlags::ONLCR,
            c_cflag: TermiosControlFlags::CREAD | TermiosControlFlags::CS8,
            c_lflag: TermiosLocalFlags::ISIG
                | TermiosLocalFlags::ICANON
                | TermiosLocalFlags::ECHO
                | TermiosLocalFlags::ECHOE
                | TermiosLocalFlags::ECHOK
                | TermiosLocalFlags::ECHONL,
            c_line: 0,
            c_cc,
            i_speed: 38400,
            o_speed: 38400,
        }
    }
}

#[derive(Debug, Default)]
pub struct TtyMetadata {
    pub winsz: WinSize,
    pub termios: Termios2,
    /// foreground process group.
    pub fg_pg: Option<Pgid>,
}
