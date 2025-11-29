use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::fs::File;
use std::io::{Read, Write};

use windows::core::{PCWSTR, w};

// Macro to create wide strings - wraps the w function
macro_rules! w {
    ($expr:expr) => {
        windows::core::w!($expr)
    };
    ($expr:expr, $($rest:expr),+) => {
        to_wstring(&format!($expr, $($rest),+))
    };
}
use windows::Win32::Media::{timeBeginPeriod, timeEndPeriod};
use windows::Win32::Foundation::{
    COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM, 
    FALSE, TRUE,
};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, CreateSolidBrush, DeleteObject, EndPaint, FillRect, 
    FrameRect, GetStockObject, InvalidateRect, PAINTSTRUCT, SetBkMode, DrawTextW,
    BLACK_BRUSH, HBRUSH, HDC, HFONT, 
    TRANSPARENT, WHITE_BRUSH, 
    FW_NORMAL, ANSI_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, 
    DEFAULT_QUALITY, DEFAULT_PITCH, FF_SWISS,
    DT_CENTER, DT_VCENTER, DT_SINGLELINE,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::{
    InitCommonControlsEx, ICC_STANDARD_CLASSES, ICC_WIN95_CLASSES, INITCOMMONCONTROLSEX, 
    WC_BUTTON, WC_COMBOBOX, WC_EDIT, WC_STATIC,
    BST_CHECKED, BST_UNCHECKED
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, SendInput, UnregisterHotKey, INPUT, INPUT_MOUSE, MOUSEEVENTF_LEFTDOWN, 
    MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, 
    MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_ABSOLUTE, VIRTUAL_KEY, 
    VK_F1, VK_F2, VK_F24, VK_MENU, VK_SHIFT, VK_CONTROL,
    HOT_KEY_MODIFIERS, EnableWindow,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, 
    GetClientRect, GetCursorPos, GetDlgItem, GetMessageW, GetSystemMetrics, GetWindowRect, GetWindowTextLengthW, 
    GetWindowTextW, LoadCursorW, LoadIconW, LoadImageW, PostQuitMessage, RegisterClassExW, 
    SendMessageW, SetCursorPos, SetForegroundWindow, SetWindowPos, SetWindowTextW, 
    ShowWindow, TranslateMessage, SW_RESTORE,
    SetWindowsHookExW, UnhookWindowsHookEx,
    BS_AUTOCHECKBOX, BS_AUTORADIOBUTTON, BS_GROUPBOX, BS_PUSHBUTTON, 
    CB_ADDSTRING, CB_GETCURSEL, CB_SETCURSEL, CBS_DROPDOWNLIST, CS_HREDRAW, CS_VREDRAW, 
    CW_USEDEFAULT, ES_NUMBER, ES_RIGHT, HHOOK, IDC_ARROW, 
    MSG, MSLLHOOKSTRUCT, KBDLLHOOKSTRUCT, SW_MINIMIZE, SWP_NOSIZE, SWP_NOZORDER, SWP_NOACTIVATE, WINDOW_EX_STYLE, WINDOW_STYLE, WM_COMMAND, WM_CREATE, 
    WM_CTLCOLORBTN, WM_CTLCOLORSTATIC, WM_DESTROY, WM_HOTKEY, WM_KEYDOWN, WM_LBUTTONDOWN, 
    WM_PAINT, WS_BORDER, WS_CAPTION, WS_CHILD, WS_DISABLED, WS_EX_DLGMODALFRAME, WS_EX_TOOLWINDOW, 
    WS_EX_TOPMOST, WS_GROUP, WS_MINIMIZEBOX, WS_OVERLAPPED, WS_POPUP, WS_SYSMENU, 
    WS_VISIBLE, WS_VSCROLL,
    WINDOWS_HOOK_ID, WH_MOUSE_LL, WH_KEYBOARD_LL,
    BM_GETCHECK, BM_SETCHECK, 
    SM_CXSCREEN, SM_CYSCREEN,
    MessageBoxW, MB_OK, MB_ICONWARNING, MB_TOPMOST,
    IDI_APPLICATION, IMAGE_ICON, LR_LOADFROMFILE,
};
use windows::Win32::System::Console::FreeConsole;
use windows::Win32::System::Threading::{SetThreadPriority, SetPriorityClass, 
    THREAD_PRIORITY_HIGHEST, HIGH_PRIORITY_CLASS};
use windows::Win32::System::{Threading::GetCurrentProcess, Threading::GetCurrentThread, Threading::OpenProcessToken};
use windows::Win32::Security::{
    GetTokenInformation, TokenElevation,
    TOKEN_QUERY, TOKEN_ELEVATION
};
use windows::Win32::Foundation::HANDLE;
use std::time::{SystemTime, UNIX_EPOCH};
use std::env;

// --- Constants ---
const ID_HOTKEY_GLOBAL: i32 = 1;

const VERSION: &str = "2.1.1";

// Control IDs
const IDC_EDIT_HOURS: i32 = 101;
const IDC_EDIT_MINS: i32 = 102;
const IDC_EDIT_SECS: i32 = 103;
const IDC_EDIT_MILLIS: i32 = 104;
const IDC_CHECK_RANDOM: i32 = 105;
const IDC_EDIT_RANDOM: i32 = 106;
const IDC_COMBO_MOUSE_BTN: i32 = 107;
const IDC_COMBO_CLICK_TYPE: i32 = 108;
const IDC_RADIO_REPEAT_TIMES: i32 = 109;
const IDC_RADIO_REPEAT_UNTIL: i32 = 110;
const IDC_EDIT_REPEAT_COUNT: i32 = 111;
const IDC_RADIO_POS_CURRENT: i32 = 112;
const IDC_RADIO_POS_PICK: i32 = 113;
const IDC_BTN_PICK_LOC: i32 = 114;
const IDC_EDIT_X: i32 = 115;
const IDC_EDIT_Y: i32 = 116;
const IDC_BTN_START: i32 = 117;
const IDC_BTN_STOP: i32 = 118;
const IDC_BTN_HOTKEY: i32 = 119;
const IDC_BTN_SETTINGS: i32 = 120;

const IDC_RADIO_REPEAT_TIME: i32 = 121;
const IDC_EDIT_REPEAT_TIME: i32 = 122;

const IDC_RADIO_REPEAT_HOLD: i32 = 123;

const IDC_HK_BTN_SET: i32 = 201;
const IDC_HK_BTN_DISPLAY: i32 = 202;
const IDC_HK_BTN_OK: i32 = 203;
const IDC_HK_BTN_CANCEL: i32 = 204;
const IDC_HK_CHECK_SHIFT: i32 = 205;
const IDC_HK_CHECK_CTRL: i32 = 206;
const IDC_HK_CHECK_ALT: i32 = 207;

const IDC_SETTINGS_CHECK_SUPPRESS: i32 = 301;
const IDC_SETTINGS_CHECK_SAFETY_DISABLE: i32 = 304;
const IDC_SETTINGS_BTN_OK: i32 = 302;
const IDC_SETTINGS_BTN_CANCEL: i32 = 303;

const CONFIG_FILE: &str = "autoclicker_settings.dat";

// Win32 API Constants often missing or named differently
const COLOR_BTNFACE: u32 = 15;
const SS_LEFT: u32 = 0x00000000;

// --- Global State ---
static mut USE_DEFAULT_SETTINGS: bool = false;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HotkeyCombo {
    vk: VIRTUAL_KEY,
    modifiers: HOT_KEY_MODIFIERS,
}

impl HotkeyCombo {
    fn new(vk: VIRTUAL_KEY, modifiers: HOT_KEY_MODIFIERS) -> Self {
        Self { vk, modifiers }
    }
    
    const fn none() -> Self {
        Self { vk: VK_F2, modifiers: HOT_KEY_MODIFIERS(0) }
    }
}

struct AppState {
    h_main_wnd: HWND,
    h_hotkey_wnd: HWND,
    h_tooltip_wnd: HWND,
    h_settings_wnd: HWND,
    h_font: HFONT,
    h_mouse_hook: HHOOK,
    h_keyboard_hook: HHOOK,
    h_hotkey_tracker_hook: HHOOK,
    
    current_hotkey: HotkeyCombo,
    temp_hotkey: HotkeyCombo,
    
    is_listening_for_key: bool,
    is_picking_location: bool,
    suppress_admin_popup: bool,
    safety_disable_enabled: bool,
}

static mut STATE: AppState = AppState {
    h_main_wnd: HWND(0),
    h_hotkey_wnd: HWND(0),
    h_tooltip_wnd: HWND(0),
    h_settings_wnd: HWND(0),
    h_font: HFONT(0),
    h_mouse_hook: HHOOK(0),
    h_keyboard_hook: HHOOK(0),
    h_hotkey_tracker_hook: HHOOK(0),
    current_hotkey: HotkeyCombo::none(),
    temp_hotkey: HotkeyCombo::none(),
    is_listening_for_key: false,
    is_picking_location: false,
    suppress_admin_popup: false,
    safety_disable_enabled: true,
};

// Thread coordination with atomic operations
static IS_RUNNING: AtomicBool = AtomicBool::new(false);
static CLICKER_THREAD_HANDLE: Mutex<Option<thread::JoinHandle<()>>> = Mutex::new(None);
static HOTKEY_HELD: AtomicBool = AtomicBool::new(false);

// Performance counters for monitoring and optimization
static CLICK_COUNT: AtomicU64 = AtomicU64::new(0);
static LAST_PERFORMANCE_CHECK: AtomicU64 = AtomicU64::new(0);

// --- High-Performance Timer ---
use std::arch::x86_64::_rdtsc;

#[inline(always)]
fn get_high_precision_time() -> u64 {
    unsafe { _rdtsc() }
}

#[inline(always)]
fn get_nanoseconds_since(previous_tsc: u64) -> u64 {
    // Convert TSC to nanoseconds - assuming 3.2GHz CPU
    // This is approximate but provides sub-nanosecond precision
    let current_tsc = get_high_precision_time();
    let cycles = current_tsc.saturating_sub(previous_tsc);
    // 3.2 GHz = 3.2 cycles per nanosecond
    cycles / 3
}

// --- CPU-Specific Optimizations ---
#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn cpu_relax() {
    unsafe {
        std::arch::x86_64::_mm_pause();
    }
}

pub struct FastRng {
    s: [u64; 4],
}

impl FastRng {
    pub fn new() -> Self {
        // Seed from system time
        let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;
        // Use SplitMix64 to initialize the state from a single u64 seed
        let mut sm = start;
        let mut next_u64 = || {
            sm = sm.wrapping_add(0x9e3779b97f4a7c15);
            let mut z = sm;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            z ^ (z >> 31)
        };

        Self {
            s: [next_u64(), next_u64(), next_u64(), next_u64()],
        }
    }

    #[inline(always)]
    fn next_u64(&mut self) -> u64 {
        let result = self.s[0].wrapping_add(self.s[3]).rotate_left(23).wrapping_add(self.s[0]);
        let t = self.s[1] << 17;

        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];

        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);

        result
    }

    #[inline(always)]
    pub fn gen_range(&mut self, min: i64, max: i64) -> i64 {
        // OPTIMIZATION 1: Lemire's Multiplication Method
        // Replaces "abs() % range" (Division) with Multiplication + Shift.
        // This turns a ~50 cycle operation into a ~3 cycle operation.
        let range = max.wrapping_sub(min).wrapping_add(1) as u64;
        let r = self.next_u64();
        
        // ((r * range) >> 64) maps r to [0, range) efficiently
        let offset = ((r as u128 * range as u128) >> 64) as i64;
        
        min.wrapping_add(offset)
    }
}

// Check if running as administrator and show warning if needed
unsafe fn check_admin_and_warn() {
    if STATE.suppress_admin_popup {
        return;
    }

    let process = GetCurrentProcess();
    let mut token_handle: HANDLE = HANDLE(0);
    
    if OpenProcessToken(process, TOKEN_QUERY, &mut token_handle).is_ok() {
        let mut elevation = TOKEN_ELEVATION::default();
        let mut return_length = 0u32;
        
        let result = GetTokenInformation(
            token_handle,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut std::ffi::c_void),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut return_length
        );
        
        if result.is_ok() && elevation.TokenIsElevated == 0 {
            // Not running as administrator, show warning
            let title = to_wstring("AutoClicker - Administrator Rights Recommended");
            let message = to_wstring(
                "This AutoClicker may not work properly in some applications (e.g. games) without administrator rights.\n\n\
                Please run the program as administrator if clicking doesn't work in your target application.\n\n\
                You can disable this warning in Settings."
            );
            
            MessageBoxW(
                STATE.h_main_wnd,
                PCWSTR(message.as_ptr()),
                PCWSTR(title.as_ptr()),
                MB_OK | MB_ICONWARNING | MB_TOPMOST
            );
        }
    }
}

fn to_wstring(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn rgb(r: u8, g: u8, b: u8) -> COLORREF {
    COLORREF((r as u32) | ((g as u32) << 8) | ((b as u32) << 16))
}

// Pre-allocated static strings to avoid repeated allocations
static mut START_TEXT_CACHE: Option<Vec<u16>> = None;
static mut STOP_TEXT_CACHE: Option<Vec<u16>> = None;

// Dialog Helpers (Reimplemented using SendMessageW)
unsafe fn is_dlg_button_checked(h_wnd: HWND, id: i32) -> bool {
    let state = SendMessageW(GetDlgItem(h_wnd, id), BM_GETCHECK, WPARAM(0), LPARAM(0));
    state.0 as u32 == BST_CHECKED.0
}

unsafe fn check_dlg_button(h_wnd: HWND, id: i32, checked: bool) {
    let state = if checked { BST_CHECKED } else { BST_UNCHECKED };
    SendMessageW(GetDlgItem(h_wnd, id), BM_SETCHECK, WPARAM(state.0 as usize), LPARAM(0));
}

unsafe fn check_radio_button(h_wnd: HWND, first: i32, last: i32, check: i32) {
    for id in first..=last {
        let state = if id == check { BST_CHECKED } else { BST_UNCHECKED };
        SendMessageW(GetDlgItem(h_wnd, id), BM_SETCHECK, WPARAM(state.0 as usize), LPARAM(0));
    }
}

fn get_key_name(vk: VIRTUAL_KEY) -> &'static str {
    let code = vk.0;
    if code >= 0x30 && code <= 0x39 { 
        return match code {
            0x30 => "0", 0x31 => "1", 0x32 => "2", 0x33 => "3", 0x34 => "4",
            0x35 => "5", 0x36 => "6", 0x37 => "7", 0x38 => "8", 0x39 => "9",
            _ => "?"
        };
    }
    if code >= 0x41 && code <= 0x5A { 
        return match code {
            0x41 => "A", 0x42 => "B", 0x43 => "C", 0x44 => "D", 0x45 => "E",
            0x46 => "F", 0x47 => "G", 0x48 => "H", 0x49 => "I", 0x4A => "J",
            0x4B => "K", 0x4C => "L", 0x4D => "M", 0x4E => "N", 0x4F => "O",
            0x50 => "P", 0x51 => "Q", 0x52 => "R", 0x53 => "S", 0x54 => "T",
            0x55 => "U", 0x56 => "V", 0x57 => "W", 0x58 => "X", 0x59 => "Y",
            0x5A => "Z",
            _ => "?"
        };
    }

    match vk {
        v if v.0 >= VK_F1.0 && v.0 <= VK_F24.0 => {
            const F_NAMES: [&str; 24] = ["F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12", "F13", "F14", "F15", "F16", "F17", "F18", "F19", "F20", "F21", "F22", "F23", "F24"];
            F_NAMES.get((v.0 - VK_F1.0) as usize).unwrap_or(&"Key ?")
        },
        windows::Win32::UI::Input::KeyboardAndMouse::VK_SPACE => "Space",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_RETURN => "Enter",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_ESCAPE => "Esc",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_BACK => "Backspace",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_TAB => "Tab",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_SHIFT => "Shift",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_CONTROL => "Ctrl",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_MENU => "Alt",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_INSERT => "Insert",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_DELETE => "Delete",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_HOME => "Home",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_END => "End",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_PRIOR => "Page Up",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_NEXT => "Page Down",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_LEFT => "Left",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_RIGHT => "Right",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_UP => "Up",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_DOWN => "Down",
        // Special characters
        windows::Win32::UI::Input::KeyboardAndMouse::VK_OEM_1 => ";",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_OEM_PLUS => "+",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_OEM_COMMA => ",",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_OEM_MINUS => "-",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_OEM_PERIOD => ".",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_OEM_2 => "/",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_OEM_3 => "`",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_OEM_4 => "[",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_OEM_5 => "\\",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_OEM_6 => "]",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_OEM_7 => "'",
        // Additional OEM keys for international keyboards
        windows::Win32::UI::Input::KeyboardAndMouse::VK_OEM_102 => "<", // Additional key for non-US keyboards
        windows::Win32::UI::Input::KeyboardAndMouse::VK_OEM_8 => "¨", // International key
        // Note: Characters like ?, !, :, ", {, }, |, _, +, =, etc. are Shift+combinations 
        // of the base keys above, so they map to the same virtual key codes
        
        // Additional special keys
        windows::Win32::UI::Input::KeyboardAndMouse::VK_CAPITAL => "Caps Lock",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_NUMLOCK => "Num Lock",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_SCROLL => "Scroll Lock",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_APPS => "Menu",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_SLEEP => "Sleep",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_LWIN => "Win Left",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_RWIN => "Win Right",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_BROWSER_BACK => "Browser Back",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_BROWSER_FORWARD => "Browser Forward",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_BROWSER_REFRESH => "Browser Refresh",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_BROWSER_STOP => "Browser Stop",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_BROWSER_SEARCH => "Browser Search",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_BROWSER_FAVORITES => "Browser Favorites",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_BROWSER_HOME => "Browser Home",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_VOLUME_MUTE => "Volume Mute",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_VOLUME_DOWN => "Volume Down",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_VOLUME_UP => "Volume Up",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_MEDIA_NEXT_TRACK => "Next Track",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_MEDIA_PREV_TRACK => "Prev Track",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_MEDIA_STOP => "Media Stop",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_MEDIA_PLAY_PAUSE => "Play/Pause",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_LAUNCH_MAIL => "Launch Mail",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_LAUNCH_MEDIA_SELECT => "Launch Media",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_LAUNCH_APP1 => "Launch App 1",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_LAUNCH_APP2 => "Launch App 2",
        // Numpad
        windows::Win32::UI::Input::KeyboardAndMouse::VK_NUMPAD0 => "Num 0",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_NUMPAD1 => "Num 1",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_NUMPAD2 => "Num 2",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_NUMPAD3 => "Num 3",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_NUMPAD4 => "Num 4",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_NUMPAD5 => "Num 5",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_NUMPAD6 => "Num 6",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_NUMPAD7 => "Num 7",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_NUMPAD8 => "Num 8",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_NUMPAD9 => "Num 9",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_MULTIPLY => "Num *",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_ADD => "Num +",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_SUBTRACT => "Num -",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_DECIMAL => "Num .",
        windows::Win32::UI::Input::KeyboardAndMouse::VK_DIVIDE => "Num /",
        _ => "Key ?",
    }
}

unsafe fn get_int_from_edit(hwnd: HWND, ctrl_id: i32) -> i32 {
    let h_ctrl = GetDlgItem(hwnd, ctrl_id);
    let len = GetWindowTextLengthW(h_ctrl);
    if len == 0 { return 0; }
    
    let mut buf = vec![0u16; (len + 1) as usize];
    GetWindowTextW(h_ctrl, &mut buf);
    
    let str_val = String::from_utf16_lossy(&buf[..len as usize]);
    str_val.trim().parse::<i32>().unwrap_or(0)
}

unsafe fn set_int_to_edit(hwnd: HWND, ctrl_id: i32, value: i32) {
    let h_ctrl = GetDlgItem(hwnd, ctrl_id);
    let s = to_wstring(&value.to_string());
    let _ = SetWindowTextW(h_ctrl, PCWSTR(s.as_ptr()));
}

unsafe fn get_float_from_edit(hwnd: HWND, ctrl_id: i32) -> f64 {
    let h_ctrl = GetDlgItem(hwnd, ctrl_id);
    let len = GetWindowTextLengthW(h_ctrl);
    if len == 0 { return 0.0; }
    
    let mut buf = vec![0u16; (len + 1) as usize];
    GetWindowTextW(h_ctrl, &mut buf);
    
    let str_val = String::from_utf16_lossy(&buf[..len as usize]);
    str_val.trim().parse::<f64>().unwrap_or(0.0)
}

unsafe fn set_float_to_edit(hwnd: HWND, ctrl_id: i32, value: f64) {
    let h_ctrl = GetDlgItem(hwnd, ctrl_id);
    let s = to_wstring(&format!("{:.2}", value));
    let _ = SetWindowTextW(h_ctrl, PCWSTR(s.as_ptr()));
}

fn get_hotkey_display_text(combo: HotkeyCombo) -> String {
    let mut parts = Vec::new();
    
    if combo.modifiers.0 & windows::Win32::UI::Input::KeyboardAndMouse::MOD_SHIFT.0 != 0 {
        parts.push("Shift".to_string());
    }
    if combo.modifiers.0 & windows::Win32::UI::Input::KeyboardAndMouse::MOD_CONTROL.0 != 0 {
        parts.push("Ctrl".to_string());
    }
    if combo.modifiers.0 & windows::Win32::UI::Input::KeyboardAndMouse::MOD_ALT.0 != 0 {
        parts.push("Alt".to_string());
    }
    
    parts.push(get_key_name(combo.vk).to_string());
    parts.join("+")
}

unsafe fn update_start_button_text() {
    let display_text = get_hotkey_display_text(STATE.current_hotkey);
    let start_text = format!("Start ({})", display_text);
    let stop_text = format!("Stop ({})", display_text);
    
    // Cache the strings to avoid repeated allocations
    START_TEXT_CACHE = Some(to_wstring(&start_text));
    STOP_TEXT_CACHE = Some(to_wstring(&stop_text));
    
    if let Some(ref text) = START_TEXT_CACHE {
        let _ = SetWindowTextW(GetDlgItem(STATE.h_main_wnd, IDC_BTN_START), PCWSTR(text.as_ptr()));
    }
    if let Some(ref text) = STOP_TEXT_CACHE {
        let _ = SetWindowTextW(GetDlgItem(STATE.h_main_wnd, IDC_BTN_STOP), PCWSTR(text.as_ptr()));
    }
}

// --- Persistence ---
unsafe fn save_settings() {
    // Don't save settings if using default mode
    if USE_DEFAULT_SETTINGS {
        return;
    }
    
    let h_wnd = STATE.h_main_wnd;
    let mut output = String::with_capacity(256); // Pre-allocate capacity

    output.push_str(&format!("IntervalH={}\n", get_int_from_edit(h_wnd, IDC_EDIT_HOURS)));
    output.push_str(&format!("IntervalM={}\n", get_int_from_edit(h_wnd, IDC_EDIT_MINS)));
    output.push_str(&format!("IntervalS={}\n", get_int_from_edit(h_wnd, IDC_EDIT_SECS)));
    output.push_str(&format!("IntervalMs={}\n", get_int_from_edit(h_wnd, IDC_EDIT_MILLIS)));

    let rnd_checked = is_dlg_button_checked(h_wnd, IDC_CHECK_RANDOM);
    output.push_str(&format!("RandomCheck={}\n", u8::from(rnd_checked)));
    output.push_str(&format!("RandomVal={}\n", get_int_from_edit(h_wnd, IDC_EDIT_RANDOM)));

    let mouse_btn = SendMessageW(GetDlgItem(h_wnd, IDC_COMBO_MOUSE_BTN), CB_GETCURSEL, WPARAM(0), LPARAM(0));
    output.push_str(&format!("MouseBtn={}\n", mouse_btn.0));
    
    let click_type = SendMessageW(GetDlgItem(h_wnd, IDC_COMBO_CLICK_TYPE), CB_GETCURSEL, WPARAM(0), LPARAM(0));
    output.push_str(&format!("ClickType={}\n", click_type.0));

    let repeat_check = is_dlg_button_checked(h_wnd, IDC_RADIO_REPEAT_TIMES);
    output.push_str(&format!("RepeatCheck={}\n", u8::from(repeat_check)));
    output.push_str(&format!("RepeatCount={}\n", get_int_from_edit(h_wnd, IDC_EDIT_REPEAT_COUNT)));
    
    let repeat_time_check = is_dlg_button_checked(h_wnd, IDC_RADIO_REPEAT_TIME);
    output.push_str(&format!("RepeatTimeCheck={}\n", u8::from(repeat_time_check)));
    output.push_str(&format!("RepeatTime={}\n", get_float_from_edit(h_wnd, IDC_EDIT_REPEAT_TIME)));

    let repeat_hold_check = is_dlg_button_checked(h_wnd, IDC_RADIO_REPEAT_HOLD);
    output.push_str(&format!("RepeatHoldCheck={}\n", u8::from(repeat_hold_check)));

    let pos_fixed = is_dlg_button_checked(h_wnd, IDC_RADIO_POS_PICK);
    output.push_str(&format!("PosFixed={}\n", u8::from(pos_fixed)));
    output.push_str(&format!("PosX={}\n", get_int_from_edit(h_wnd, IDC_EDIT_X)));
    output.push_str(&format!("PosY={}\n", get_int_from_edit(h_wnd, IDC_EDIT_Y)));

    output.push_str(&format!("HotkeyVk={}\n", { let vk = STATE.current_hotkey.vk.0; vk }));
    output.push_str(&format!("HotkeyMods={}\n", { let mods = STATE.current_hotkey.modifiers.0; mods }));
    output.push_str(&format!("SuppressAdminPopup={}\n", u8::from(STATE.suppress_admin_popup)));
    output.push_str(&format!("SafetyDisableEnabled={}\n", u8::from(STATE.safety_disable_enabled)));

    if let Ok(mut file) = File::create(CONFIG_FILE) {
        let _ = file.write_all(output.as_bytes());
    }
}

unsafe fn load_settings() {
    // Don't load settings if using default mode
    if USE_DEFAULT_SETTINGS {
        return;
    }
    
    let mut file = match File::open(CONFIG_FILE) {
        Ok(f) => f,
        Err(_) => return,
    };
    let mut contents = String::new();
    if file.read_to_string(&mut contents).is_err() { return; }

    let h_wnd = STATE.h_main_wnd;

    for line in contents.lines() {
        if let Some((key, val_str)) = line.split_once('=') {
            let val = val_str.parse::<i32>().unwrap_or(0);
            match key {
                "IntervalH" => set_int_to_edit(h_wnd, IDC_EDIT_HOURS, val),
                "IntervalM" => set_int_to_edit(h_wnd, IDC_EDIT_MINS, val),
                "IntervalS" => set_int_to_edit(h_wnd, IDC_EDIT_SECS, val),
                "IntervalMs" => set_int_to_edit(h_wnd, IDC_EDIT_MILLIS, val),
                "RandomCheck" => check_dlg_button(h_wnd, IDC_CHECK_RANDOM, val != 0),
                "RandomVal" => set_int_to_edit(h_wnd, IDC_EDIT_RANDOM, val),
                "MouseBtn" => { SendMessageW(GetDlgItem(h_wnd, IDC_COMBO_MOUSE_BTN), CB_SETCURSEL, WPARAM(val as usize), LPARAM(0)); },
                "ClickType" => { SendMessageW(GetDlgItem(h_wnd, IDC_COMBO_CLICK_TYPE), CB_SETCURSEL, WPARAM(val as usize), LPARAM(0)); },
                "RepeatCheck" => {
                    let id = if val != 0 { IDC_RADIO_REPEAT_TIMES } else { IDC_RADIO_REPEAT_UNTIL };
                    check_radio_button(h_wnd, IDC_RADIO_REPEAT_TIMES, IDC_RADIO_REPEAT_UNTIL, id);
                },
                "RepeatCount" => set_int_to_edit(h_wnd, IDC_EDIT_REPEAT_COUNT, val),
                "RepeatTimeCheck" => {
                    let id = if val != 0 { IDC_RADIO_REPEAT_TIME } else { IDC_RADIO_REPEAT_UNTIL };
                    check_radio_button(h_wnd, IDC_RADIO_REPEAT_TIME, IDC_RADIO_REPEAT_UNTIL, id);
                },
                "RepeatTime" => set_float_to_edit(h_wnd, IDC_EDIT_REPEAT_TIME, val as f64),
                "RepeatHoldCheck" => {
                    let id = if val != 0 { IDC_RADIO_REPEAT_HOLD } else { IDC_RADIO_REPEAT_UNTIL };
                    check_radio_button(h_wnd, IDC_RADIO_REPEAT_HOLD, IDC_RADIO_REPEAT_UNTIL, id);
                },
                "PosFixed" => {
                    let id = if val != 0 { IDC_RADIO_POS_PICK } else { IDC_RADIO_POS_CURRENT };
                    check_radio_button(h_wnd, IDC_RADIO_POS_CURRENT, IDC_RADIO_POS_PICK, id);
                    EnableWindow(GetDlgItem(h_wnd, IDC_BTN_PICK_LOC), if val != 0 { TRUE } else { FALSE });
                },
                "PosX" => set_int_to_edit(h_wnd, IDC_EDIT_X, val),
                "PosY" => set_int_to_edit(h_wnd, IDC_EDIT_Y, val),
                "HotkeyVk" => STATE.current_hotkey.vk = VIRTUAL_KEY(val as u16),
                "HotkeyMods" => STATE.current_hotkey.modifiers = HOT_KEY_MODIFIERS(val as u32),
                "SuppressAdminPopup" => STATE.suppress_admin_popup = val != 0,
                "SafetyDisableEnabled" => STATE.safety_disable_enabled = val != 0,
                _ => {}
            }
        }
    }
    update_start_button_text();
}

// --- Ultra-Performance Clicker Logic ---
unsafe fn clicker_loop(
    hwnd: isize, 
    h: i32, m: i32, s: i32, ms: i32, 
    use_random: bool, random_offset: i32,
    repeat_finite: bool, repeat_count: i32,
    repeat_time: bool, repeat_duration_seconds: f64,
    repeat_while_held: bool,
    btn_idx: i32, type_idx: i32,
    use_fixed_pos: bool, fixed_x: i32, fixed_y: i32
) {
    let hwnd = HWND(hwnd);
    
    let total_millis = (h as i64 * 3600000) + (m as i64 * 60000) + (s as i64 * 1000) + ms as i64;
    let total_millis = if total_millis < 0 { 0 } else { total_millis };

    // This fixes thread::sleep being inaccurate for values like 2ms, 5ms, etc.
    timeBeginPeriod(1);

    let mut current_clicks = 0;
    let start_time = std::time::Instant::now();
    let repeat_duration_nanos = if repeat_time { 
        (repeat_duration_seconds * 1_000_000_000.0) as u128 
    } else { 
        u128::MAX 
    };

    let mut simd_rng = if use_random { Some(FastRng::new()) } else { None };
    
    let (btn_down, btn_up) = match btn_idx {
        0 => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
        1 => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
        _ => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
    };
    
    let (screen_width, screen_height) = if use_fixed_pos {
        (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN))
    } else { (0, 0) };
    
    let (norm_x, norm_y) = if use_fixed_pos && screen_width > 0 && screen_height > 0 {
        ((fixed_x * 65535) / screen_width, (fixed_y * 65535) / screen_height)
    } else { (0, 0) };
    
    let input_size = std::mem::size_of::<INPUT>() as i32;
    
    // Adjust thresholds for precision modes
    let use_ultra_precision = total_millis < 15; // Use pure spin loop for < 15ms
    let use_high_precision = total_millis < 50;
    
    let double_click_delay_ns = 50_000_000; // 50ms
    
    // ... [Input struct setup code remains the same as your original] ...
    let fixed_inputs = if use_fixed_pos {
        let mut inputs = [INPUT::default(), INPUT::default()];
        inputs[0].r#type = INPUT_MOUSE;
        inputs[0].Anonymous.mi.dwFlags = btn_down | MOUSEEVENTF_ABSOLUTE;
        inputs[0].Anonymous.mi.dx = norm_x;
        inputs[0].Anonymous.mi.dy = norm_y;
        inputs[1].r#type = INPUT_MOUSE;
        inputs[1].Anonymous.mi.dwFlags = btn_up | MOUSEEVENTF_ABSOLUTE;
        inputs[1].Anonymous.mi.dx = norm_x;
        inputs[1].Anonymous.mi.dy = norm_y;
        Some(inputs)
    } else { None };
    
    let current_inputs = if !use_fixed_pos {
        let mut inputs = [INPUT::default(), INPUT::default()];
        inputs[0].r#type = INPUT_MOUSE;
        inputs[0].Anonymous.mi.dwFlags = btn_down;
        inputs[1].r#type = INPUT_MOUSE;
        inputs[1].Anonymous.mi.dwFlags = btn_up;
        Some(inputs)
    } else { None };

    let mut is_running = IS_RUNNING.load(Ordering::Relaxed);
    let mut last_tsc = get_high_precision_time();
    let mut click_count = 0u64;
    
    // Safety disable corner detection variables
    let mut corner_detection_counter = 0;
    let mut last_corner_check_tsc = get_high_precision_time();
    let corner_check_interval_ns = 100_000_000; // Check every 100ms
    let corner_threshold = 10; // Need 10 consecutive corner detections to trigger
    
    let target_delay_ns = total_millis as u64 * 1_000_000;
    let random_range_ns = if use_random && random_offset > 0 {
        random_offset as u64 * 1_000_000
    } else { 0 };

    while is_running {
        if use_fixed_pos {
            let _ = SetCursorPos(fixed_x, fixed_y);
            SendInput(fixed_inputs.as_ref().unwrap(), input_size);
        } else {
            SendInput(current_inputs.as_ref().unwrap(), input_size);
        }

        if type_idx == 1 {
            let double_click_start_tsc = get_high_precision_time();
            while get_nanoseconds_since(double_click_start_tsc) < double_click_delay_ns && is_running {
                cpu_relax();
                if click_count % 10 == 0 { is_running = IS_RUNNING.load(Ordering::Relaxed); }
            }
            if is_running {
                if use_fixed_pos { SendInput(fixed_inputs.as_ref().unwrap(), input_size); } 
                else { SendInput(current_inputs.as_ref().unwrap(), input_size); }
            }
        }

        current_clicks += 1;
        click_count += 1;
        CLICK_COUNT.store(click_count, Ordering::Relaxed);
        
        // Safety disable corner detection
        if STATE.safety_disable_enabled && get_nanoseconds_since(last_corner_check_tsc) >= corner_check_interval_ns {
            last_corner_check_tsc = get_high_precision_time();
            let mut cursor_pos = POINT::default();
            if GetCursorPos(&mut cursor_pos).is_ok() {
                let screen_width = GetSystemMetrics(SM_CXSCREEN);
                let screen_height = GetSystemMetrics(SM_CYSCREEN);
                
                // Check if cursor is in any corner (within 5 pixels)
                let corner_tolerance = 5;
                let in_corner = 
                    (cursor_pos.x <= corner_tolerance && cursor_pos.y <= corner_tolerance) || // Top-left
                    (cursor_pos.x >= screen_width - corner_tolerance && cursor_pos.y <= corner_tolerance) || // Top-right
                    (cursor_pos.x <= corner_tolerance && cursor_pos.y >= screen_height - corner_tolerance) || // Bottom-left
                    (cursor_pos.x >= screen_width - corner_tolerance && cursor_pos.y >= screen_height - corner_tolerance); // Bottom-right
                
                if in_corner {
                    corner_detection_counter += 1;
                    if corner_detection_counter >= corner_threshold {
                        // Safety disable triggered - stop clicking and show popup
                        IS_RUNNING.store(false, Ordering::Relaxed);
                        let title = to_wstring("AutoClicker - Safety Disable");
                        let message = to_wstring(
                            "Safety disable triggered!\n\n\
                            The mouse appears to be stuck in a corner. \
                            Auto clicking has been stopped to prevent unintended behavior.\n\n\
                            You can disable this feature in Settings if needed."
                        );
                        
                        MessageBoxW(
                            hwnd,
                            PCWSTR(message.as_ptr()),
                            PCWSTR(title.as_ptr()),
                            MB_OK | MB_ICONWARNING | MB_TOPMOST
                        );
                        
                        // Update UI to stopped state
                        let stopped_title = if USE_DEFAULT_SETTINGS {
                            w!("Stopped - (OP)timized Auto Clicker {} (Default Mode)", VERSION)
                        } else {
                            w!("Stopped - (OP)timized Auto Clicker {}", VERSION)
                        };
                        let _ = SetWindowTextW(hwnd, PCWSTR(stopped_title.as_ptr()));
                        EnableWindow(GetDlgItem(hwnd, IDC_BTN_START), TRUE);
                        EnableWindow(GetDlgItem(hwnd, IDC_EDIT_HOURS), TRUE);
                        EnableWindow(GetDlgItem(hwnd, IDC_EDIT_MINS), TRUE);
                        EnableWindow(GetDlgItem(hwnd, IDC_EDIT_SECS), TRUE);
                        EnableWindow(GetDlgItem(hwnd, IDC_EDIT_MILLIS), TRUE);
                        EnableWindow(GetDlgItem(hwnd, IDC_BTN_STOP), FALSE);
                        break;
                    }
                } else {
                    corner_detection_counter = 0; // Reset counter when not in corner
                }
            }
        }
        
        is_running = IS_RUNNING.load(Ordering::Relaxed);
        if !is_running { break; }

        // Check if hotkey is still held down for repeat while held mode
        if repeat_while_held && !HOTKEY_HELD.load(Ordering::Relaxed) {
            let stopped_title = if USE_DEFAULT_SETTINGS {
                w!("Stopped - (OP)timized Auto Clicker {} (Default Mode)", VERSION)
            } else {
                w!("Stopped - (OP)timized Auto Clicker {}", VERSION)
            };
            IS_RUNNING.store(false, Ordering::Relaxed);
            let _ = SetWindowTextW(hwnd, PCWSTR(stopped_title.as_ptr()));
            EnableWindow(GetDlgItem(hwnd, IDC_BTN_START), TRUE);
            EnableWindow(GetDlgItem(hwnd, IDC_EDIT_HOURS), TRUE);
            EnableWindow(GetDlgItem(hwnd, IDC_EDIT_MINS), TRUE);
            EnableWindow(GetDlgItem(hwnd, IDC_EDIT_SECS), TRUE);
            EnableWindow(GetDlgItem(hwnd, IDC_EDIT_MILLIS), TRUE);
            break;
        }

        if repeat_finite && current_clicks >= repeat_count {
            let stopped_title = if USE_DEFAULT_SETTINGS {
                w!("Stopped - (OP)timized Auto Clicker {} (Default Mode)", VERSION)
            } else {
                w!("Stopped - (OP)timized Auto Clicker {}", VERSION)
            };
            IS_RUNNING.store(false, Ordering::Relaxed);
            let _ = SetWindowTextW(hwnd, PCWSTR(stopped_title.as_ptr()));
            EnableWindow(GetDlgItem(hwnd, IDC_BTN_START), TRUE);
            EnableWindow(GetDlgItem(hwnd, IDC_EDIT_HOURS), TRUE);
            EnableWindow(GetDlgItem(hwnd, IDC_EDIT_MINS), TRUE);
            EnableWindow(GetDlgItem(hwnd, IDC_EDIT_SECS), TRUE);
            EnableWindow(GetDlgItem(hwnd, IDC_EDIT_MILLIS), TRUE);
            EnableWindow(GetDlgItem(hwnd, IDC_BTN_STOP), FALSE);
            break;
        }

        // Check if time duration limit has been reached
        if repeat_time && start_time.elapsed().as_nanos() >= repeat_duration_nanos {
            let stopped_title = if USE_DEFAULT_SETTINGS {
                w!("Stopped - (OP)timized Auto Clicker {} (Default Mode)", VERSION)
            } else {
                w!("Stopped - (OP)timized Auto Clicker {}", VERSION)
            };
            IS_RUNNING.store(false, Ordering::Relaxed);
            let _ = SetWindowTextW(hwnd, PCWSTR(stopped_title.as_ptr()));
            EnableWindow(GetDlgItem(hwnd, IDC_BTN_START), TRUE);
            EnableWindow(GetDlgItem(hwnd, IDC_EDIT_HOURS), TRUE);
            EnableWindow(GetDlgItem(hwnd, IDC_EDIT_MINS), TRUE);
            EnableWindow(GetDlgItem(hwnd, IDC_EDIT_SECS), TRUE);
            EnableWindow(GetDlgItem(hwnd, IDC_EDIT_MILLIS), TRUE);
            EnableWindow(GetDlgItem(hwnd, IDC_BTN_STOP), FALSE);
            break;
        }

        // Logic for 0ms (infinite speed)
        if target_delay_ns == 0 && (!use_random || random_offset == 0) {
            // If 0ms interval, skip timing logic completely, just spin aggressively
            // We add a tiny yield every few clicks to prevent complete OS freeze
            if click_count % 100 == 0 {
                 thread::yield_now(); 
            }
            last_tsc = get_high_precision_time();
            continue;
        }

        let mut current_sleep_ns = target_delay_ns;
        if use_random && random_offset > 0 {
            let random_ns = simd_rng.as_mut().unwrap().gen_range(-(random_range_ns as i64), random_range_ns as i64);
            current_sleep_ns = (target_delay_ns as i64 + random_ns) as u64;
        }

        let target_delay_tsc = current_sleep_ns * 3; 

        if use_ultra_precision {
            // Busy wait (Spin loop)
            let target_tsc = last_tsc + target_delay_tsc;
            loop {
                let current_tsc = get_high_precision_time();
                if current_tsc >= target_tsc || !is_running { break; }
                cpu_relax();
                if click_count % 50 == 0 { is_running = IS_RUNNING.load(Ordering::Relaxed); }
            }
        } else if use_high_precision {
            // Hybrid
            let remaining_tsc = target_delay_tsc.saturating_sub(get_high_precision_time().saturating_sub(last_tsc));
            // We can sleep if remaining time is > 2ms because timeBeginPeriod(1) is active
            if remaining_tsc > 6_000_000 { 
                thread::sleep(Duration::from_nanos(remaining_tsc / 3));
            }
            let target_tsc = last_tsc + target_delay_tsc;
            while get_high_precision_time() < target_tsc && is_running {
                cpu_relax();
                if click_count % 100 == 0 { is_running = IS_RUNNING.load(Ordering::Relaxed); }
            }
        } else {
            // Standard sleep
            let remaining_ns = current_sleep_ns.saturating_sub(get_nanoseconds_since(last_tsc) / 3);
            if remaining_ns > 500_000 {
                thread::sleep(Duration::from_nanos(remaining_ns - 500_000));
            }
            let target_tsc = last_tsc + target_delay_tsc;
            while get_high_precision_time() < target_tsc && is_running {
                cpu_relax();
                if click_count % 200 == 0 { is_running = IS_RUNNING.load(Ordering::Relaxed); }
            }
        }
        
        last_tsc = get_high_precision_time();
        
        if click_count % 10000 == 0 {
            LAST_PERFORMANCE_CHECK.store(last_tsc, Ordering::Relaxed);
        }
    }

    // Reset timer resolution when thread finishes
    timeEndPeriod(1);
}

unsafe fn toggle_start_stop() {
    let h_wnd = STATE.h_main_wnd;
    let currently_running = IS_RUNNING.load(Ordering::Relaxed);

    if currently_running {
        IS_RUNNING.store(false, Ordering::Relaxed);
        HOTKEY_HELD.store(false, Ordering::Relaxed);
        
        // Don't clean up hotkey tracker hook here - it needs to stay active
        // to track key releases for repeat while held functionality
        // The hook will be cleaned up in WM_DESTROY
        
        // Don't block the UI thread
        if let Ok(mut handle_lock) = CLICKER_THREAD_HANDLE.try_lock() {
            if let Some(handle) = handle_lock.take() {
                std::mem::forget(handle);
            }
        }

        let stopped_title = if USE_DEFAULT_SETTINGS {
            w!("Stopped - (OP)timized Auto Clicker {} (Default Mode)", VERSION)
        } else {
            w!("Stopped - (OP)timized Auto Clicker {}", VERSION)
        };
        let _ = SetWindowTextW(h_wnd, PCWSTR(stopped_title.as_ptr()));
        EnableWindow(GetDlgItem(h_wnd, IDC_BTN_START), TRUE);
        EnableWindow(GetDlgItem(h_wnd, IDC_BTN_START), TRUE);
        EnableWindow(GetDlgItem(h_wnd, IDC_EDIT_HOURS), TRUE);
        EnableWindow(GetDlgItem(h_wnd, IDC_EDIT_MINS), TRUE);
        EnableWindow(GetDlgItem(h_wnd, IDC_EDIT_SECS), TRUE);
        EnableWindow(GetDlgItem(h_wnd, IDC_EDIT_MILLIS), TRUE);
        
        // Enable/disable stop button based on repeat mode
        let repeat_while_held = is_dlg_button_checked(h_wnd, IDC_RADIO_REPEAT_HOLD);
        if !repeat_while_held {
            EnableWindow(GetDlgItem(h_wnd, IDC_BTN_STOP), FALSE);
        }
    } else {
        save_settings();
        IS_RUNNING.store(true, Ordering::Relaxed);
        
        // Set high priority for timing-critical thread
        let _ = SetPriorityClass(GetCurrentProcess(), HIGH_PRIORITY_CLASS);
        let _ = SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_HIGHEST);
        
        let running_title = if USE_DEFAULT_SETTINGS {
            w!("Running... - (OP)timized Auto Clicker {} (Default Mode)", VERSION)
        } else {
            w!("Running... - (OP)timized Auto Clicker {}", VERSION)
        };
        let _ = SetWindowTextW(h_wnd, PCWSTR(running_title.as_ptr()));
        EnableWindow(GetDlgItem(h_wnd, IDC_BTN_START), FALSE);
        
        // Only enable stop button if not in repeat while held mode
        let repeat_while_held = is_dlg_button_checked(h_wnd, IDC_RADIO_REPEAT_HOLD);
        if !repeat_while_held {
            EnableWindow(GetDlgItem(h_wnd, IDC_BTN_STOP), TRUE);
        }

        EnableWindow(GetDlgItem(h_wnd, IDC_EDIT_HOURS), FALSE);
        EnableWindow(GetDlgItem(h_wnd, IDC_EDIT_MINS), FALSE);
        EnableWindow(GetDlgItem(h_wnd, IDC_EDIT_SECS), FALSE);
        EnableWindow(GetDlgItem(h_wnd, IDC_EDIT_MILLIS), FALSE);

        // Capture params for thread
        let h = get_int_from_edit(h_wnd, IDC_EDIT_HOURS);
        let m = get_int_from_edit(h_wnd, IDC_EDIT_MINS);
        let s = get_int_from_edit(h_wnd, IDC_EDIT_SECS);
        let ms = get_int_from_edit(h_wnd, IDC_EDIT_MILLIS);
        let use_random = is_dlg_button_checked(h_wnd, IDC_CHECK_RANDOM);
        let random_offset = get_int_from_edit(h_wnd, IDC_EDIT_RANDOM);
        let repeat_finite = is_dlg_button_checked(h_wnd, IDC_RADIO_REPEAT_TIMES);
        let repeat_count = get_int_from_edit(h_wnd, IDC_EDIT_REPEAT_COUNT);
        let repeat_time = is_dlg_button_checked(h_wnd, IDC_RADIO_REPEAT_TIME);
        let repeat_duration = get_float_from_edit(h_wnd, IDC_EDIT_REPEAT_TIME);
        let repeat_while_held = is_dlg_button_checked(h_wnd, IDC_RADIO_REPEAT_HOLD);
        let btn_idx = SendMessageW(GetDlgItem(h_wnd, IDC_COMBO_MOUSE_BTN), CB_GETCURSEL, WPARAM(0), LPARAM(0)).0 as i32;
        let type_idx = SendMessageW(GetDlgItem(h_wnd, IDC_COMBO_CLICK_TYPE), CB_GETCURSEL, WPARAM(0), LPARAM(0)).0 as i32;
        let use_fixed = is_dlg_button_checked(h_wnd, IDC_RADIO_POS_PICK);
        let fx = get_int_from_edit(h_wnd, IDC_EDIT_X);
        let fy = get_int_from_edit(h_wnd, IDC_EDIT_Y);

        let hwnd_val = h_wnd.0;

        // Ensure hotkey tracker hook is active for repeat while held functionality
        if repeat_while_held && STATE.h_hotkey_tracker_hook.0 == 0 {
            let h_instance = HINSTANCE(GetModuleHandleW(None).unwrap_or_default().0);
            STATE.h_hotkey_tracker_hook = SetWindowsHookExW(
                WINDOWS_HOOK_ID(WH_KEYBOARD_LL.0 as i32), 
                Some(hotkey_tracker_proc), 
                h_instance, 
                0
            ).unwrap_or(HHOOK(0));
        }

        let handle = thread::Builder::new()
            .name("ultra-precise-autoclicker".to_string())
            .spawn(move || {
                clicker_loop(hwnd_val, h, m, s, ms, use_random, random_offset, repeat_finite, repeat_count, repeat_time, repeat_duration, repeat_while_held, btn_idx, type_idx, use_fixed, fx, fy);
            })
            .expect("Failed to spawn ultra-precise autoclicker thread");

        // Use try_lock to avoid blocking UI thread
        if let Ok(mut handle_lock) = CLICKER_THREAD_HANDLE.try_lock() {
            *handle_lock = Some(handle);
        }
    }
}

// --- Hooks and Tooltips ---

extern "system" fn tooltip_wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let hdc = BeginPaint(hwnd, &mut ps);
                let mut rect = RECT::default();
                let _ = GetClientRect(hwnd, &mut rect);

                let h_brush = CreateSolidBrush(rgb(255, 255, 225));
                FillRect(hdc, &rect, h_brush);
                DeleteObject(h_brush);
                
                let stock_brush = GetStockObject(BLACK_BRUSH);
                FrameRect(hdc, &rect, HBRUSH(stock_brush.0));

                let mut pt = POINT::default();
                let _ = GetCursorPos(&mut pt);
                // Use stack-allocated buffer for small text
                let text = format!("X: {} Y: {}", pt.x, pt.y);
                let mut text_w = to_wstring(&text);

                SetBkMode(hdc, TRANSPARENT);
                // Center the text in the tooltip window
                let mut rect = RECT::default();
                let _ = GetClientRect(hwnd, &mut rect);
                let _ = DrawTextW(hdc, &mut text_w, &mut rect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
                
                EndPaint(hwnd, &ps);
                LRESULT(0)
            },
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

extern "system" fn mouse_hook_proc(n_code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        if n_code >= 0 {
            let p_mouse = &*(lparam.0 as *const MSLLHOOKSTRUCT);
            
            if STATE.h_tooltip_wnd.0 != 0 {
                // Get current cursor position directly - this should be most reliable
                let mut cursor_pos = POINT::default();
                if GetCursorPos(&mut cursor_pos).is_ok() {
                    // Position tooltip relative to actual cursor position
                    let _ = SetWindowPos(STATE.h_tooltip_wnd, HWND(-1isize as _), 
                        cursor_pos.x + 15, cursor_pos.y + 15, 0, 0, 
                        SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOZORDER);
                    
                    // Force tooltip to repaint with new coordinates
                    InvalidateRect(STATE.h_tooltip_wnd, None, TRUE);
                }
            }

            if wparam.0 as u32 == WM_LBUTTONDOWN {
                let x = p_mouse.pt.x;
                let y = p_mouse.pt.y;

                if STATE.h_mouse_hook.0 != 0 {
                    let _ = UnhookWindowsHookEx(STATE.h_mouse_hook);
                    STATE.h_mouse_hook = HHOOK(0);
                }
                if STATE.h_tooltip_wnd.0 != 0 {
                    let _ = DestroyWindow(STATE.h_tooltip_wnd);
                    STATE.h_tooltip_wnd = HWND(0);
                }

                ShowWindow(STATE.h_main_wnd, SW_RESTORE);
                SetForegroundWindow(STATE.h_main_wnd);

                set_int_to_edit(STATE.h_main_wnd, IDC_EDIT_X, x);
                set_int_to_edit(STATE.h_main_wnd, IDC_EDIT_Y, y);
                
                // Manually check radio button
                check_radio_button(STATE.h_main_wnd, IDC_RADIO_POS_CURRENT, IDC_RADIO_POS_PICK, IDC_RADIO_POS_PICK);
                EnableWindow(GetDlgItem(STATE.h_main_wnd, IDC_BTN_PICK_LOC), TRUE);

                save_settings();
                STATE.is_picking_location = false;
                
                return LRESULT(1); // Consume click
            }
        }
        CallNextHookEx(STATE.h_mouse_hook, n_code, wparam, lparam)
    }
}

extern "system" fn hotkey_tracker_proc(n_code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        if n_code >= 0 && IS_RUNNING.load(Ordering::Relaxed) {
            let p_keyboard = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
            let key = VIRTUAL_KEY(p_keyboard.vkCode as u16);
            
            // Check if this matches our hotkey
            if key == STATE.current_hotkey.vk {
                let mut modifiers_match = true;
                
                // Check modifier states
                if STATE.current_hotkey.modifiers.0 & windows::Win32::UI::Input::KeyboardAndMouse::MOD_SHIFT.0 != 0 {
                    let shift_state = (windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState(VK_SHIFT.0 as i32) & 0x8000u16 as i16) != 0;
                    if !shift_state { modifiers_match = false; }
                }
                if STATE.current_hotkey.modifiers.0 & windows::Win32::UI::Input::KeyboardAndMouse::MOD_CONTROL.0 != 0 {
                    let ctrl_state = (windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState(VK_CONTROL.0 as i32) & 0x8000u16 as i16) != 0;
                    if !ctrl_state { modifiers_match = false; }
                }
                if STATE.current_hotkey.modifiers.0 & windows::Win32::UI::Input::KeyboardAndMouse::MOD_ALT.0 != 0 {
                    let alt_state = (windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState(VK_MENU.0 as i32) & 0x8000u16 as i16) != 0;
                    if !alt_state { modifiers_match = false; }
                }
                
                if modifiers_match {
                    match wparam.0 as u32 {
                        WM_KEYDOWN => {
                            HOTKEY_HELD.store(true, Ordering::Relaxed);
                        },
                        // use allow attribute because rust analyzer complains about unused and non-snake case
                        #[allow(non_snake_case, unused_variables)]
                        WM_KEYUP => {
                            // WM_KEYUP is used here to detect key release
                            HOTKEY_HELD.store(false, Ordering::Relaxed);
                        },
                    }
                }
            }
        }
        CallNextHookEx(STATE.h_hotkey_tracker_hook, n_code, wparam, lparam)
    }
}

extern "system" fn keyboard_hook_proc(n_code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        if n_code >= 0 && STATE.is_listening_for_key {
            if wparam.0 as u32 == WM_KEYDOWN {
                let p_keyboard = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
                let key = VIRTUAL_KEY(p_keyboard.vkCode as u16);
                
                // Get current modifier state
                let mut modifiers = HOT_KEY_MODIFIERS(0);
                
                // Check if modifier keys are currently held down
                let shift_state = (windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState(VK_SHIFT.0 as i32) & 0x8000u16 as i16) != 0;
                let ctrl_state = (windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState(VK_CONTROL.0 as i32) & 0x8000u16 as i16) != 0;
                let alt_state = (windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState(VK_MENU.0 as i32) & 0x8000u16 as i16) != 0;
                
                if shift_state {
                    modifiers.0 |= windows::Win32::UI::Input::KeyboardAndMouse::MOD_SHIFT.0;
                }
                if ctrl_state {
                    modifiers.0 |= windows::Win32::UI::Input::KeyboardAndMouse::MOD_CONTROL.0;
                }
                if alt_state {
                    modifiers.0 |= windows::Win32::UI::Input::KeyboardAndMouse::MOD_ALT.0;
                }
                
                // Exclude modifier keys from being selected as hotkeys
                if key != VK_SHIFT && key != VK_CONTROL && key != VK_MENU {
                    STATE.temp_hotkey = HotkeyCombo::new(key, modifiers);
                    let display_text = get_hotkey_display_text(STATE.temp_hotkey);
                    let display_w = to_wstring(&display_text);
                    
                    // Update the button text in the hotkey dialog
                    if STATE.h_hotkey_wnd.0 != 0 {
                        let _ = SetWindowTextW(GetDlgItem(STATE.h_hotkey_wnd, IDC_HK_BTN_DISPLAY), PCWSTR(display_w.as_ptr()));
                    }
                    
                    STATE.is_listening_for_key = false;
                    
                    // Clean up the keyboard hook
                    if STATE.h_keyboard_hook.0 != 0 {
                        let _ = UnhookWindowsHookEx(STATE.h_keyboard_hook);
                        STATE.h_keyboard_hook = HHOOK(0);
                    }
                    
                    return LRESULT(1); // Consume the key press
                }
            }
        }
        CallNextHookEx(STATE.h_keyboard_hook, n_code, wparam, lparam)
    }
}

unsafe fn start_pick_location() {
    STATE.is_picking_location = true;
    ShowWindow(STATE.h_main_wnd, SW_MINIMIZE);

    let class_name = w!("TooltipClass");
    let h_instance = HINSTANCE(GetModuleHandleW(None).unwrap_or_default().0);

    let wc = windows::Win32::UI::WindowsAndMessaging::WNDCLASSEXW {
        cbSize: std::mem::size_of::<windows::Win32::UI::WindowsAndMessaging::WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(tooltip_wnd_proc),
        hInstance: h_instance,
        hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
        hbrBackground: HBRUSH(GetStockObject(WHITE_BRUSH).0),
        lpszClassName: PCWSTR(class_name.as_ptr()),
        ..Default::default()
    };
    RegisterClassExW(&wc);

    // Create tooltip with initial position at (0,0) and let the hook position it
    STATE.h_tooltip_wnd = CreateWindowExW(
        WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
        class_name,
        None,
        WS_POPUP | WS_VISIBLE,
        0, 0, 120, 25,
        None, None, h_instance, None
    );

    // Position tooltip at current cursor location immediately
    let mut cursor_pos = POINT::default();
    if GetCursorPos(&mut cursor_pos).is_ok() {
        let _ = SetWindowPos(STATE.h_tooltip_wnd, HWND(-1isize as _), 
            cursor_pos.x + 15, cursor_pos.y + 15, 0, 0, 
            SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOZORDER);
    }

    STATE.h_mouse_hook = SetWindowsHookExW(WINDOWS_HOOK_ID(WH_MOUSE_LL.0 as i32), Some(mouse_hook_proc), h_instance, 0).unwrap();
}

// --- Settings Dialog ---

extern "system" fn settings_dlg_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_CREATE => {
                let mut rc_main = RECT::default();
                let mut rc_dlg = RECT::default();
                let _ = GetWindowRect(STATE.h_main_wnd, &mut rc_main);
                let _ = GetWindowRect(hwnd, &mut rc_dlg);
                
                let x = rc_main.left + (rc_main.right - rc_main.left) / 2 - (rc_dlg.right - rc_dlg.left) / 2;
                let y = rc_main.top + (rc_main.bottom - rc_main.top) / 2 - (rc_dlg.bottom - rc_dlg.top) / 2;
                let _ = SetWindowPos(hwnd, HWND(0), x, y, 0, 0, SWP_NOSIZE | SWP_NOZORDER);

                create_ctrl(w!("STATIC"), w!("Settings"), SS_LEFT as u32, 10, 10, 100, 20, hwnd, -1);
                create_ctrl(w!("BUTTON"), w!("Suppress administrator popup at startup"), BS_AUTOCHECKBOX as u32, 20, 40, 250, 20, hwnd, IDC_SETTINGS_CHECK_SUPPRESS);
                create_ctrl(w!("BUTTON"), w!("Enable safety disable (corner detection)"), BS_AUTOCHECKBOX as u32, 20, 65, 250, 20, hwnd, IDC_SETTINGS_CHECK_SAFETY_DISABLE);
                
                if STATE.suppress_admin_popup {
                    check_dlg_button(hwnd, IDC_SETTINGS_CHECK_SUPPRESS, true);
                }
                if STATE.safety_disable_enabled {
                    check_dlg_button(hwnd, IDC_SETTINGS_CHECK_SAFETY_DISABLE, true);
                }

                create_ctrl(w!("BUTTON"), w!("Ok"), BS_PUSHBUTTON as u32, 50, 105, 80, 30, hwnd, IDC_SETTINGS_BTN_OK);
                create_ctrl(w!("BUTTON"), w!("Cancel"), BS_PUSHBUTTON as u32, 160, 105, 80, 30, hwnd, IDC_SETTINGS_BTN_CANCEL);
                LRESULT(0)
            },
            WM_COMMAND => {
                let id = (wparam.0 & 0xFFFF) as i32;
                match id {
                    IDC_SETTINGS_BTN_OK => {
                        STATE.suppress_admin_popup = is_dlg_button_checked(hwnd, IDC_SETTINGS_CHECK_SUPPRESS);
                        STATE.safety_disable_enabled = is_dlg_button_checked(hwnd, IDC_SETTINGS_CHECK_SAFETY_DISABLE);
                        save_settings();
                        let _ = DestroyWindow(hwnd);
                    },
                    IDC_SETTINGS_BTN_CANCEL => { 
                        let _ = DestroyWindow(hwnd); 
                    },
                    _ => {}
                }
                LRESULT(0)
            },
            WM_DESTROY => {
                EnableWindow(STATE.h_main_wnd, TRUE);
                SetForegroundWindow(STATE.h_main_wnd);
                STATE.h_settings_wnd = HWND(0);
                LRESULT(0)
            },
            WM_CTLCOLORSTATIC | WM_CTLCOLORBTN => {
                SetBkMode(HDC(wparam.0 as isize), TRANSPARENT);
                LRESULT(GetStockObject(windows::Win32::Graphics::Gdi::NULL_BRUSH).0 as _)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

unsafe fn open_settings_dialog() {
    EnableWindow(STATE.h_main_wnd, FALSE);

    let class_name = w!("SettingsDlgClass");
    let h_instance = HINSTANCE(GetModuleHandleW(None).unwrap_or_default().0);
    
    let wc = windows::Win32::UI::WindowsAndMessaging::WNDCLASSEXW {
        cbSize: std::mem::size_of::<windows::Win32::UI::WindowsAndMessaging::WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(settings_dlg_proc),
        hInstance: h_instance,
        hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
        hbrBackground: HBRUSH((COLOR_BTNFACE + 1) as _),
        lpszClassName: PCWSTR(class_name.as_ptr()),
        ..Default::default()
    };
    RegisterClassExW(&wc);

    STATE.h_settings_wnd = CreateWindowExW(
        WS_EX_DLGMODALFRAME | WS_EX_TOPMOST,
        class_name,
        w!("Settings"),
        WS_POPUP | WS_CAPTION | WS_SYSMENU | WS_VISIBLE,
        0, 0, 300, 180,
        STATE.h_main_wnd,
        None,
        h_instance,
        None
    );
}

// --- Hotkey Dialog ---

extern "system" fn hotkey_dlg_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_CREATE => {
                let mut rc_main = RECT::default();
                let mut rc_dlg = RECT::default();
                let _ = GetWindowRect(STATE.h_main_wnd, &mut rc_main);
                let _ = GetWindowRect(hwnd, &mut rc_dlg);
                
                let x = rc_main.left + (rc_main.right - rc_main.left) / 2 - (rc_dlg.right - rc_dlg.left) / 2;
                let y = rc_main.top + (rc_main.bottom - rc_main.top) / 2 - (rc_dlg.bottom - rc_dlg.top) / 2;
                let _ = SetWindowPos(hwnd, HWND(0), x, y, 0, 0, SWP_NOSIZE | SWP_NOZORDER);

                create_ctrl(w!("STATIC"), w!("Hotkey Setting"), SS_LEFT as u32, 10, 10, 100, 20, hwnd, -1);
                
                // Modifier checkboxes
                create_ctrl(w!("BUTTON"), w!("Shift"), BS_AUTOCHECKBOX as u32, 20, 40, 60, 20, hwnd, IDC_HK_CHECK_SHIFT);
                create_ctrl(w!("BUTTON"), w!("Ctrl"), BS_AUTOCHECKBOX as u32, 90, 40, 60, 20, hwnd, IDC_HK_CHECK_CTRL);
                create_ctrl(w!("BUTTON"), w!("Alt"), BS_AUTOCHECKBOX as u32, 160, 40, 60, 20, hwnd, IDC_HK_CHECK_ALT);
                
                create_ctrl(w!("BUTTON"), w!("Start / Stop"), BS_PUSHBUTTON as u32, 20, 70, 120, 35, hwnd, IDC_HK_BTN_SET);
                
                let display_text = get_hotkey_display_text(STATE.temp_hotkey);
                let display_w = to_wstring(&display_text);
                create_ctrl(w!("BUTTON"), PCWSTR(display_w.as_ptr()), WS_CHILD.0 | WS_VISIBLE.0 | WS_DISABLED.0, 150, 70, 120, 35, hwnd, IDC_HK_BTN_DISPLAY);

                create_ctrl(w!("BUTTON"), w!("Ok"), BS_PUSHBUTTON as u32, 50, 130, 80, 30, hwnd, IDC_HK_BTN_OK);
                create_ctrl(w!("BUTTON"), w!("Cancel"), BS_PUSHBUTTON as u32, 160, 130, 80, 30, hwnd, IDC_HK_BTN_CANCEL);
                
                // Set initial modifier checkbox states
                check_dlg_button(hwnd, IDC_HK_CHECK_SHIFT, STATE.temp_hotkey.modifiers.0 & windows::Win32::UI::Input::KeyboardAndMouse::MOD_SHIFT.0 != 0);
                check_dlg_button(hwnd, IDC_HK_CHECK_CTRL, STATE.temp_hotkey.modifiers.0 & windows::Win32::UI::Input::KeyboardAndMouse::MOD_CONTROL.0 != 0);
                check_dlg_button(hwnd, IDC_HK_CHECK_ALT, STATE.temp_hotkey.modifiers.0 & windows::Win32::UI::Input::KeyboardAndMouse::MOD_ALT.0 != 0);
                
                LRESULT(0)
            },
            WM_COMMAND => {
                let id = (wparam.0 & 0xFFFF) as i32;
                match id {
                    IDC_HK_BTN_SET => {
                        STATE.is_listening_for_key = true;
                        let _ = SetWindowTextW(GetDlgItem(hwnd, IDC_HK_BTN_DISPLAY), w!("Press any key..."));
                        
                        // Set up the keyboard hook to capture key presses globally
                        let h_instance = HINSTANCE(GetModuleHandleW(None).unwrap_or_default().0);
                        STATE.h_keyboard_hook = SetWindowsHookExW(
                            WINDOWS_HOOK_ID(WH_KEYBOARD_LL.0 as i32), 
                            Some(keyboard_hook_proc), 
                            h_instance, 
                            0
                        ).unwrap_or(HHOOK(0));
                    },
                    IDC_HK_BTN_OK => {
                        // Update temp_hotkey with selected modifiers
                        let mut modifiers = HOT_KEY_MODIFIERS(0);
                        if is_dlg_button_checked(hwnd, IDC_HK_CHECK_SHIFT) {
                            modifiers.0 |= windows::Win32::UI::Input::KeyboardAndMouse::MOD_SHIFT.0;
                        }
                        if is_dlg_button_checked(hwnd, IDC_HK_CHECK_CTRL) {
                            modifiers.0 |= windows::Win32::UI::Input::KeyboardAndMouse::MOD_CONTROL.0;
                        }
                        if is_dlg_button_checked(hwnd, IDC_HK_CHECK_ALT) {
                            modifiers.0 |= windows::Win32::UI::Input::KeyboardAndMouse::MOD_ALT.0;
                        }
                        
                        STATE.temp_hotkey.modifiers = modifiers;
                        STATE.current_hotkey = STATE.temp_hotkey;
                        
                        let _ = UnregisterHotKey(STATE.h_main_wnd, ID_HOTKEY_GLOBAL);
                        RegisterHotKey(STATE.h_main_wnd, ID_HOTKEY_GLOBAL, STATE.current_hotkey.modifiers, STATE.current_hotkey.vk.0 as u32).ok();
                        update_start_button_text();
                        save_settings();
                        let _ = DestroyWindow(hwnd);
                    },
                    IDC_HK_BTN_CANCEL => { 
                // Clean up keyboard hook if it's still active
                if STATE.h_keyboard_hook.0 != 0 {
                    let _ = UnhookWindowsHookEx(STATE.h_keyboard_hook);
                    STATE.h_keyboard_hook = HHOOK(0);
                }
                STATE.is_listening_for_key = false;
                let _ = DestroyWindow(hwnd); 
            },
                    _ => {}
                }
                LRESULT(0)
            },
            WM_DESTROY => {
                // Clean up keyboard hook if it's still active
                if STATE.h_keyboard_hook.0 != 0 {
                    let _ = UnhookWindowsHookEx(STATE.h_keyboard_hook);
                    STATE.h_keyboard_hook = HHOOK(0);
                }
                
                // Reset listening state
                STATE.is_listening_for_key = false;
                
                EnableWindow(STATE.h_main_wnd, TRUE);
                SetForegroundWindow(STATE.h_main_wnd);
                STATE.h_hotkey_wnd = HWND(0);
                LRESULT(0)
            },
            WM_CTLCOLORSTATIC | WM_CTLCOLORBTN => {
                SetBkMode(HDC(wparam.0 as isize), TRANSPARENT);
                LRESULT(GetStockObject(windows::Win32::Graphics::Gdi::NULL_BRUSH).0 as _)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

unsafe fn open_hotkey_dialog() {
    STATE.temp_hotkey = STATE.current_hotkey;
    STATE.is_listening_for_key = false;
    EnableWindow(STATE.h_main_wnd, FALSE);

    let class_name = w!("HotkeyDlgClass");
    let h_instance = HINSTANCE(GetModuleHandleW(None).unwrap_or_default().0);
    
    let wc = windows::Win32::UI::WindowsAndMessaging::WNDCLASSEXW {
        cbSize: std::mem::size_of::<windows::Win32::UI::WindowsAndMessaging::WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(hotkey_dlg_proc),
        hInstance: h_instance,
        hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
        hbrBackground: HBRUSH((COLOR_BTNFACE + 1) as _),
        lpszClassName: PCWSTR(class_name.as_ptr()),
        ..Default::default()
    };
    RegisterClassExW(&wc);

    STATE.h_hotkey_wnd = CreateWindowExW(
        WS_EX_DLGMODALFRAME | WS_EX_TOPMOST,
        class_name,
        w!("Hotkey Setting"),
        WS_POPUP | WS_CAPTION | WS_SYSMENU | WS_VISIBLE,
        0, 0, 300, 210,  // Increased height for modifier checkboxes
        STATE.h_main_wnd,
        None,
        h_instance,
        None
    );
}

// --- Main Window ---

unsafe fn create_ctrl(class_name: PCWSTR, text: PCWSTR, style: u32, x: i32, y: i32, w: i32, h: i32, parent: HWND, id: i32) -> HWND {
    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        class_name,
        text,
        WINDOW_STYLE(style | WS_CHILD.0 | WS_VISIBLE.0),
        x, y, w, h,
        parent,
        windows::Win32::UI::WindowsAndMessaging::HMENU(id as isize),
        None, None
    );
    SendMessageW(hwnd, windows::Win32::UI::WindowsAndMessaging::WM_SETFONT, WPARAM(STATE.h_font.0 as usize), LPARAM(1));
    hwnd
}

unsafe fn create_ui(hwnd: HWND) {
    STATE.h_font = CreateFontW(16, 0, 0, 0, FW_NORMAL.0 as i32, 
        0, 0, 0, 
        u32::from(ANSI_CHARSET.0), u32::from(OUT_DEFAULT_PRECIS.0), u32::from(CLIP_DEFAULT_PRECIS.0), 
        u32::from(DEFAULT_QUALITY.0), u32::from(DEFAULT_PITCH.0 | FF_SWISS.0), w!("Segoe UI"));

    // Interval - Increased width and better spacing
    create_ctrl(WC_BUTTON, w!("Click interval"), WS_GROUP.0 | BS_GROUPBOX as u32, 10, 10, 465, 80, hwnd, -1);
    create_ctrl(WC_EDIT, w!("0"), WS_BORDER.0 | ES_NUMBER as u32 | ES_RIGHT as u32, 30, 35, 50, 22, hwnd, IDC_EDIT_HOURS);
    create_ctrl(WC_STATIC, w!("hours"), SS_LEFT as u32, 85, 38, 40, 20, hwnd, -1);
    create_ctrl(WC_EDIT, w!("0"), WS_BORDER.0 | ES_NUMBER as u32 | ES_RIGHT as u32, 130, 35, 50, 22, hwnd, IDC_EDIT_MINS);
    create_ctrl(WC_STATIC, w!("mins"), SS_LEFT as u32, 185, 38, 40, 20, hwnd, -1);
    create_ctrl(WC_EDIT, w!("0"), WS_BORDER.0 | ES_NUMBER as u32 | ES_RIGHT as u32, 230, 35, 50, 22, hwnd, IDC_EDIT_SECS);
    create_ctrl(WC_STATIC, w!("secs"), SS_LEFT as u32, 285, 38, 40, 20, hwnd, -1);
    create_ctrl(WC_EDIT, w!("100"), WS_BORDER.0 | ES_NUMBER as u32 | ES_RIGHT as u32, 330, 35, 50, 22, hwnd, IDC_EDIT_MILLIS);
    create_ctrl(WC_STATIC, w!("milliseconds"), SS_LEFT as u32, 385, 38, 100, 20, hwnd, -1);

    // Random - Better positioning
    create_ctrl(WC_BUTTON, w!("Random offset"), BS_AUTOCHECKBOX as u32, 50, 62, 120, 20, hwnd, IDC_CHECK_RANDOM);
    create_ctrl(WC_EDIT, w!("40"), WS_BORDER.0 | ES_NUMBER as u32 | ES_RIGHT as u32, 170, 62, 50, 22, hwnd, IDC_EDIT_RANDOM);
    create_ctrl(WC_STATIC, w!("milliseconds"), SS_LEFT as u32, 225, 65, 80, 20, hwnd, -1);

    // Options - Increased width for better layout
    create_ctrl(WC_BUTTON, w!("Click options"), WS_GROUP.0 | BS_GROUPBOX as u32, 10, 100, 275, 90, hwnd, -1);
    // Pre-allocate static strings for combo boxes to avoid repeated allocations
    const MOUSE_BUTTONS: [&str; 3] = ["Left", "Right", "Middle"];
    const CLICK_TYPES: [&str; 2] = ["Single", "Double"];
    
    create_ctrl(WC_STATIC, w!("Mouse button:"), SS_LEFT as u32, 20, 125, 90, 20, hwnd, -1);
    let h_combo_btn = create_ctrl(WC_COMBOBOX, w!(""), CBS_DROPDOWNLIST as u32 | WS_VSCROLL.0, 110, 122, 100, 100, hwnd, IDC_COMBO_MOUSE_BTN);
    for btn_name in MOUSE_BUTTONS.iter() {
        SendMessageW(h_combo_btn, CB_ADDSTRING, WPARAM(0), LPARAM(to_wstring(btn_name).as_ptr() as isize));
    }
    SendMessageW(h_combo_btn, CB_SETCURSEL, WPARAM(0), LPARAM(0));

    create_ctrl(WC_STATIC, w!("Click type:"), SS_LEFT as u32, 20, 155, 90, 20, hwnd, -1);
    let h_combo_type = create_ctrl(WC_COMBOBOX, w!(""), CBS_DROPDOWNLIST as u32 | WS_VSCROLL.0, 110, 152, 100, 100, hwnd, IDC_COMBO_CLICK_TYPE);
    for type_name in CLICK_TYPES.iter() {
        SendMessageW(h_combo_type, CB_ADDSTRING, WPARAM(0), LPARAM(to_wstring(type_name).as_ptr() as isize));
    }
    SendMessageW(h_combo_type, CB_SETCURSEL, WPARAM(0), LPARAM(0));

    // Repeat - Increased width and better spacing
    create_ctrl(WC_BUTTON, w!("Click repeat"), WS_GROUP.0 | BS_GROUPBOX as u32, 295, 100, 185, 140, hwnd, -1);
    create_ctrl(WC_BUTTON, w!("Repeat"), BS_AUTORADIOBUTTON as u32 | WS_GROUP.0, 305, 125, 70, 20, hwnd, IDC_RADIO_REPEAT_TIMES);
    create_ctrl(WC_EDIT, w!("1"), WS_BORDER.0 | ES_NUMBER as u32 | ES_RIGHT as u32, 380, 125, 50, 22, hwnd, IDC_EDIT_REPEAT_COUNT);
    create_ctrl(WC_STATIC, w!("times"), SS_LEFT as u32, 435, 128, 40, 20, hwnd, -1);
    create_ctrl(WC_BUTTON, w!("Repeat for"), BS_AUTORADIOBUTTON as u32, 305, 155, 80, 20, hwnd, IDC_RADIO_REPEAT_TIME);
    create_ctrl(WC_EDIT, w!("5.0"), WS_BORDER.0 | ES_RIGHT as u32, 390, 155, 50, 22, hwnd, IDC_EDIT_REPEAT_TIME);
    create_ctrl(WC_STATIC, w!("secs"), SS_LEFT as u32, 445, 158, 40, 20, hwnd, -1);
    create_ctrl(WC_BUTTON, w!("Repeat while held"), BS_AUTORADIOBUTTON as u32, 305, 185, 160, 20, hwnd, IDC_RADIO_REPEAT_HOLD);
    let h_radio_until = create_ctrl(WC_BUTTON, w!("Repeat until stopped"), BS_AUTORADIOBUTTON as u32, 305, 210, 160, 20, hwnd, IDC_RADIO_REPEAT_UNTIL);
    SendMessageW(h_radio_until, BM_SETCHECK, WPARAM(BST_CHECKED.0 as usize), LPARAM(0));

    // Position - Increased width and better layout
    create_ctrl(WC_BUTTON, w!("Cursor position"), WS_GROUP.0 | BS_GROUPBOX as u32, 10, 250, 465, 70, hwnd, -1);
    let h_radio_cur = create_ctrl(WC_BUTTON, w!("Current location"), BS_AUTORADIOBUTTON as u32 | WS_GROUP.0, 20, 280, 130, 20, hwnd, IDC_RADIO_POS_CURRENT);
    SendMessageW(h_radio_cur, BM_SETCHECK, WPARAM(BST_CHECKED.0 as usize), LPARAM(0));
    create_ctrl(WC_BUTTON, w!("Pick location"), BS_AUTORADIOBUTTON as u32, 160, 280, 20, 20, hwnd, IDC_RADIO_POS_PICK);
    create_ctrl(WC_BUTTON, w!("Pick location"), BS_PUSHBUTTON as u32, 185, 278, 100, 26, hwnd, IDC_BTN_PICK_LOC);
    EnableWindow(GetDlgItem(hwnd, IDC_BTN_PICK_LOC), FALSE);

    create_ctrl(WC_STATIC, w!("X"), SS_LEFT as u32, 320, 282, 10, 20, hwnd, -1);
    create_ctrl(WC_EDIT, w!("0"), WS_BORDER.0 | ES_NUMBER as u32 | ES_RIGHT as u32, 335, 280, 40, 22, hwnd, IDC_EDIT_X);
    create_ctrl(WC_STATIC, w!("Y"), SS_LEFT as u32, 385, 282, 10, 20, hwnd, -1);
    create_ctrl(WC_EDIT, w!("0"), WS_BORDER.0 | ES_NUMBER as u32 | ES_RIGHT as u32, 400, 280, 40, 22, hwnd, IDC_EDIT_Y);

    // Main Buttons - Better spacing and positioning
    create_ctrl(WC_BUTTON, w!("Start (F2)"), BS_PUSHBUTTON as u32, 65, 340, 180, 40, hwnd, IDC_BTN_START);
    create_ctrl(WC_BUTTON, w!("Stop (F2)"), BS_PUSHBUTTON as u32, 255, 340, 180, 40, hwnd, IDC_BTN_STOP);
    EnableWindow(GetDlgItem(hwnd, IDC_BTN_STOP), FALSE);
    create_ctrl(WC_BUTTON, w!("Hotkey setting"), BS_PUSHBUTTON as u32, 65, 395, 180, 30, hwnd, IDC_BTN_HOTKEY);
    create_ctrl(WC_BUTTON, w!("Settings"), BS_PUSHBUTTON as u32, 255, 395, 180, 30, hwnd, IDC_BTN_SETTINGS);

    load_settings();
}

extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_CREATE => {
                STATE.h_main_wnd = hwnd;
                create_ui(hwnd);
                RegisterHotKey(hwnd, ID_HOTKEY_GLOBAL, STATE.current_hotkey.modifiers, STATE.current_hotkey.vk.0 as u32).ok();
                
                // Set up hotkey tracker hook for repeat while held functionality
                let h_instance = HINSTANCE(GetModuleHandleW(None).unwrap_or_default().0);
                STATE.h_hotkey_tracker_hook = SetWindowsHookExW(
                    WINDOWS_HOOK_ID(WH_KEYBOARD_LL.0 as i32), 
                    Some(hotkey_tracker_proc), 
                    h_instance, 
                    0
                ).unwrap_or(HHOOK(0));
                
                check_admin_and_warn();
                LRESULT(0)
            },
            WM_COMMAND => {
                let id = (wparam.0 & 0xFFFF) as i32;
                match id {
                    IDC_BTN_START => { 
                    if !IS_RUNNING.load(Ordering::Relaxed) { 
                        toggle_start_stop(); 
                    } 
                },
                IDC_BTN_STOP => { 
                    if IS_RUNNING.load(Ordering::Relaxed) { 
                        // Check if repeat while held is enabled - if so, don't allow manual stop
                        let repeat_while_held = is_dlg_button_checked(hwnd, IDC_RADIO_REPEAT_HOLD);
                        if !repeat_while_held {
                            toggle_start_stop(); 
                        }
                    } 
                },
                    IDC_BTN_HOTKEY => open_hotkey_dialog(),
                    IDC_BTN_SETTINGS => open_settings_dialog(),
                    IDC_BTN_PICK_LOC => start_pick_location(),
                    IDC_RADIO_POS_PICK => { EnableWindow(GetDlgItem(hwnd, IDC_BTN_PICK_LOC), TRUE); },
                    IDC_RADIO_POS_CURRENT => { EnableWindow(GetDlgItem(hwnd, IDC_BTN_PICK_LOC), FALSE); },
                    IDC_RADIO_REPEAT_HOLD => { 
                        EnableWindow(GetDlgItem(hwnd, IDC_BTN_STOP), FALSE); 
                        // Also disable stop if currently running and repeat while held is selected
                        if IS_RUNNING.load(Ordering::Relaxed) {
                            EnableWindow(GetDlgItem(hwnd, IDC_BTN_STOP), FALSE);
                        }
                    },
                    IDC_RADIO_REPEAT_UNTIL => { 
                        if IS_RUNNING.load(Ordering::Relaxed) {
                            EnableWindow(GetDlgItem(hwnd, IDC_BTN_STOP), TRUE);
                        }
                    },
                    IDC_RADIO_REPEAT_TIMES => { 
                        if IS_RUNNING.load(Ordering::Relaxed) {
                            EnableWindow(GetDlgItem(hwnd, IDC_BTN_STOP), TRUE);
                        }
                    },
                    IDC_RADIO_REPEAT_TIME => { 
                        if IS_RUNNING.load(Ordering::Relaxed) {
                            EnableWindow(GetDlgItem(hwnd, IDC_BTN_STOP), TRUE);
                        }
                    },
                    _ => {}
                }
                LRESULT(0)
            },
            WM_HOTKEY => {
                if (wparam.0 as i32) == ID_HOTKEY_GLOBAL {
                    // Check if repeat while held is enabled
                    let repeat_while_held = is_dlg_button_checked(STATE.h_main_wnd, IDC_RADIO_REPEAT_HOLD);
                    
                    if repeat_while_held {
                        // For repeat while held mode, set the held state and start clicking
                        HOTKEY_HELD.store(true, Ordering::Relaxed);
                        if !IS_RUNNING.load(Ordering::Relaxed) {
                            toggle_start_stop();
                        }
                    } else {
                        // Normal toggle behavior
                        toggle_start_stop();
                    }
                }
                LRESULT(0)
            },
            WM_DESTROY => {
                save_settings();
                IS_RUNNING.store(false, Ordering::Relaxed);
                
                // Don't block on thread join during shutdown
                if let Ok(mut handle_lock) = CLICKER_THREAD_HANDLE.try_lock() {
                    if let Some(handle) = handle_lock.take() {
                        std::mem::forget(handle);
                    }
                }
                
                let _ = UnregisterHotKey(hwnd, ID_HOTKEY_GLOBAL);
                
                // Clean up hooks
                if STATE.h_mouse_hook.0 != 0 {
                    let _ = UnhookWindowsHookEx(STATE.h_mouse_hook);
                }
                if STATE.h_keyboard_hook.0 != 0 {
                    let _ = UnhookWindowsHookEx(STATE.h_keyboard_hook);
                }
                if STATE.h_hotkey_tracker_hook.0 != 0 {
                    let _ = UnhookWindowsHookEx(STATE.h_hotkey_tracker_hook);
                }
                
                if STATE.h_font.0 != 0 {
                    DeleteObject(STATE.h_font);
                }
                PostQuitMessage(0);
                LRESULT(0)
            },
            WM_CTLCOLORSTATIC | WM_CTLCOLORBTN => {
                SetBkMode(HDC(wparam.0 as isize), TRANSPARENT);
                LRESULT(GetStockObject(windows::Win32::Graphics::Gdi::NULL_BRUSH).0 as _)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}


fn main() {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    
    unsafe {
        // Check for --default or -d flag
        USE_DEFAULT_SETTINGS = args.iter().any(|arg| arg == "--default" || arg == "-d");
        
        // Completely detach from console window on startup
        let _ = FreeConsole();

        let icex = INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_STANDARD_CLASSES | ICC_WIN95_CLASSES,
        };
        InitCommonControlsEx(&icex);

        let h_instance = HINSTANCE(GetModuleHandleW(None).unwrap_or_default().0);
        let class_name = w!("RustAutoClickerClass");

        let wc = windows::Win32::UI::WindowsAndMessaging::WNDCLASSEXW {
            cbSize: std::mem::size_of::<windows::Win32::UI::WindowsAndMessaging::WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc),
            hInstance: h_instance,
            hIcon: {
                // Try to load icon from file at runtime, fallback to default if not found
                let icon_path = "icon.ico"; // Icon file path relative to executable
                let wide_path: Vec<u16> = icon_path.encode_utf16().chain(std::iter::once(0)).collect();
                LoadImageW(
                    None,
                    PCWSTR(wide_path.as_ptr()),
                    IMAGE_ICON,
                    0, 0, // Use default size
                    LR_LOADFROMFILE
                ).map(|h| std::mem::transmute(h)).unwrap_or_else(|_| {
                    LoadIconW(None, IDI_APPLICATION).unwrap_or_default()
                })
            },
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            hbrBackground: HBRUSH((COLOR_BTNFACE + 1) as _),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        RegisterClassExW(&wc);

        let title_text = if USE_DEFAULT_SETTINGS {
                format!("Stopped - (OP)timized Auto Clicker {} (Default Mode)", VERSION)
            } else {
                format!("Stopped - (OP)timized Auto Clicker {}", VERSION)
            };
        let title_wide: Vec<u16> = title_text.encode_utf16().chain(std::iter::once(0)).collect();
        
        let _hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class_name,
            PCWSTR(title_wide.as_ptr()),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_VISIBLE,
            CW_USEDEFAULT, CW_USEDEFAULT, 500, 480,
            None, None, h_instance, None
        );

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
