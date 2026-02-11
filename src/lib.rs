/*
 *  VB Decompiler Plugin SDK (Rust Version)
 *  Copyright (c) 2001-2026 Sergey Chubchenko (DotFix Software). All rights reserved.
 *
 *  Website: https://www.vb-decompiler.org
 *  Support: admin@vb-decompiler.org
 *
 *  License:
 *      Permission is hereby granted to use, modify, and distribute this file 
 *      for the purpose of creating plugins for VB Decompiler.
 */

extern crate native_windows_gui as nwg;
extern crate native_windows_derive as nwd;

use nwd::NwgUi;
use nwg::NativeUi;
use std::ffi::{CString};
use std::os::raw::{c_char, c_void, c_int};
use std::ptr;
use std::cell::RefCell;

// --- GUI Structure Definition ---
#[derive(Default, NwgUi)]
pub struct PluginUi {
    // Window settings
    #[nwg_control(size: (400, 300), position: (300, 300), title: "VB Project (Rust)", flags: "WINDOW|VISIBLE")]
    #[nwg_events( OnWindowClose: [PluginUi::on_close] )]
    window: nwg::Window,

    // Text Box (Memo)
    #[nwg_control(text: "", size: (380, 240), position: (10, 10))]
    #[nwg_control(flags: "VISIBLE|MULTI_LINE|VSCROLL")]
    txt_vb_project: nwg::TextBox,

    // Close Button
    #[nwg_control(text: "Close", size: (100, 30), position: (150, 260))]
    #[nwg_events( OnButtonClick: [PluginUi::on_close_click] )]
    cmd_close: nwg::Button,
}

impl PluginUi {
    fn on_close_click(&self) {
        nwg::modal_info_message(&self.window, "Rust Plugin", "The plugin worked correctly");
        nwg::stop_thread_dispatch();
    }

    fn on_close(&self) {
        nwg::stop_thread_dispatch();
    }
}

// --- SDK Constants (vlType) ---
#[repr(i32)]
#[derive(Copy, Clone)]
pub enum VlType {
    GetVBProject = 1,
    SetVBProject = 2,
    GetFileName = 3,
    IsNativeCompilation = 4,
    ClearAllBuffers = 5,
    GetCompiler = 6,
    IsPacked = 7,
    SetStackCheckBoxValue = 8,
    SetAnalyzerCheckBoxValue = 9,
    GetVBFormName = 10,
    SetVBFormName = 11,
    GetVBForm = 12,
    SetVBForm = 13,
    GetVBFormCount = 14,
    GetSubMain = 20,
    SetSubMain = 21,
    GetModuleName = 30,
    SetModuleName = 31,
    GetModule = 32,
    SetModule = 33,
    GetModuleStringReferences = 34,
    SetModuleStringReferences = 35,
    SetModuleCount = 36,
    GetModuleFunctionName = 40,
    SetModuleFunctionName = 41,
    GetModuleFunctionAddress = 42,
    SetModuleFunctionAddress = 43,
    GetModuleFunction = 44,
    SetModuleFunction = 45,
    GetModuleFunctionStrRef = 46,
    SetModuleFunctionStrRef = 47,
    GetModuleFunctionCount = 48,
    GetActiveText = 50,
    SetActiveText = 51,
    GetActiveDisasmText = 52,
    SetActiveDisasmText = 53,
    SetActiveTextLine = 54,
    GetActiveModuleCoordinats = 55,
    GetVBDecompilerPath = 56,
    GetModuleFunctionCode = 57,
    SetStatusBarText = 58,
    GetFrxIconCount = 60,
    GetFrxIconOffset = 61,
    GetFrxIconSize = 62,
    GetModuleFunctionDisasm = 70,
    SetModuleFunctionDisasm = 71,
    UpdateAll = 100,
}

// --- Engine ---
type PluginEngineFn = unsafe extern "system" fn(i32, i32, i32, *mut c_void) -> *mut c_void;
static mut PLUGIN_ENGINE: Option<PluginEngineFn> = None;

// Use RefCell to hold UI data temporarily if needed, though NWG handles its state internally
thread_local! {
    static ACTIVE_TEXT_CACHE: RefCell<String> = RefCell::new(String::new());
}

// --- Helpers ---

unsafe fn init(engine_ptr: *mut c_void) -> bool {
    if engine_ptr.is_null() {
        return false;
    }
    let func: PluginEngineFn = std::mem::transmute(engine_ptr);
    PLUGIN_ENGINE = Some(func);
    true
}

unsafe fn get_value(vl_type: VlType, vl_number: i32, vl_fn_number: i32) -> String {
    if let Some(engine) = PLUGIN_ENGINE {
        let ptr = engine(vl_type as i32, vl_number, vl_fn_number, ptr::null_mut());
        if !ptr.is_null() {
            let wide_ptr = ptr as *const u16;
            let mut len = 0;
            while *wide_ptr.offset(len) != 0 {
                len += 1;
            }
            let slice = std::slice::from_raw_parts(wide_ptr, len as usize);
            return String::from_utf16_lossy(slice);
        }
    }
    String::new()
}

unsafe fn set_value(vl_type: VlType, value: &str, vl_number: i32, vl_fn_number: i32) {
    if let Some(engine) = PLUGIN_ENGINE {
        let c_str = CString::new(value).unwrap_or_default();
        engine(vl_type as i32, vl_number, vl_fn_number, c_str.into_raw() as *mut c_void);
    }
}

// --- Exports ---

#[no_mangle]
pub unsafe extern "system" fn VBDecompilerPluginName(_hwnd: *mut c_void, _h_rich: *mut c_void, buffer: *mut c_char, _reserved: c_int) {
    let name = "Test plugin written in Rust\0";
    let bytes = name.as_bytes();
    ptr::copy_nonoverlapping(bytes.as_ptr(), buffer as *mut u8, bytes.len());
}

#[no_mangle]
pub unsafe extern "system" fn VBDecompilerPluginAuthor(_hwnd: *mut c_void, _h_rich: *mut c_void, buffer: *mut c_char, _reserved: c_int) {
    let author = "YourName, rust@example.com\0";
    let bytes = author.as_bytes();
    ptr::copy_nonoverlapping(bytes.as_ptr(), buffer as *mut u8, bytes.len());
}

#[no_mangle]
pub unsafe extern "system" fn VBDecompilerPluginLoad(hwnd: *mut c_void, _h_rich: *mut c_void, _buffer: *mut c_void, engine: *mut c_void) {
    if !init(engine) {
        return;
    }

    // Logic: Modify Project Text
    let mut vb_project = get_value(VlType::GetVBProject, 0, 0);
    vb_project.push_str("\r\n' This text was added by Rust Plugin!");
    set_value(VlType::SetVBProject, &vb_project, 0, 0);
    set_value(VlType::UpdateAll, "", 0, 0);

    // Get Active Text
    let active_text = get_value(VlType::GetActiveText, 0, 0);

    // --- Show Form ---
    
    // Initialize NWG
    if let Err(_) = nwg::init() {
        return; 
    }
    
    // Set default font
    let mut font = nwg::Font::default();
    nwg::Font::builder()
        .family("Segoe UI")
        .size(16)
        .build(&mut font).ok();
    nwg::Font::set_global_default(Some(font));

    // Create UI
    let ui = PluginUi::build_ui(Default::default()).expect("Failed to build UI");
    
    // Set Text (Loaded from host)
    ui.txt_vb_project.set_text(&active_text);

    // Parent the Rust window to the VB Decompiler window (WinAPI SetParent)
    // This is optional but good for modal behavior simulation
    use winapi::um::winuser::SetParent;
    if !hwnd.is_null() {
       SetParent(ui.window.handle.hwnd().unwrap() as *mut _, hwnd as *mut _);
    }

    // Dispatch events (blocking call, acts like ShowModal)
    nwg::dispatch_thread_events();
}

