#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(not(windows))]
fn main() {
    eprintln!("APL Studio is available on Windows only");
}

#[cfg(windows)]
mod windows_app {
    use std::{
        ffi::{c_int, c_void, OsStr},
        fs,
        mem::{size_of, zeroed},
        os::windows::ffi::OsStrExt,
        path::PathBuf,
        ptr::{null, null_mut},
        thread,
    };

    type Bool = i32;
    type Dword = u32;
    type Hbrush = *mut c_void;
    type Hcursor = *mut c_void;
    type Hfont = *mut c_void;
    type Hinstance = *mut c_void;
    type Hmenu = *mut c_void;
    type Hwnd = *mut c_void;
    type Lparam = isize;
    type Lresult = isize;
    type Uint = u32;
    type Wparam = usize;

    type WndProc = Option<unsafe extern "system" fn(Hwnd, Uint, Wparam, Lparam) -> Lresult>;

    #[repr(C)]
    struct Accel {
        virtual_key: u8,
        key: u16,
        command: u16,
    }

    #[repr(C)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[repr(C)]
    struct Msg {
        hwnd: Hwnd,
        message: Uint,
        w_param: Wparam,
        l_param: Lparam,
        time: Dword,
        point: Point,
        private: Dword,
    }

    #[repr(C)]
    struct Rect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }

    #[repr(C)]
    struct WndClassW {
        style: Uint,
        wnd_proc: WndProc,
        cls_extra: c_int,
        wnd_extra: c_int,
        instance: Hinstance,
        icon: *mut c_void,
        cursor: Hcursor,
        background: Hbrush,
        menu_name: *const u16,
        class_name: *const u16,
    }

    #[repr(C)]
    struct CreateStructW {
        create_params: *mut c_void,
        instance: Hinstance,
        menu: Hmenu,
        parent: Hwnd,
        cy: c_int,
        cx: c_int,
        y: c_int,
        x: c_int,
        style: i32,
        name: *const u16,
        class: *const u16,
        ex_style: Dword,
    }

    #[repr(C)]
    struct MinMaxInfo {
        reserved: Point,
        max_size: Point,
        max_position: Point,
        min_track_size: Point,
        max_track_size: Point,
    }

    #[repr(C)]
    struct OpenFileNameW {
        struct_size: Dword,
        owner: Hwnd,
        instance: Hinstance,
        filter: *const u16,
        custom_filter: *mut u16,
        max_custom_filter: Dword,
        filter_index: Dword,
        file: *mut u16,
        max_file: Dword,
        file_title: *mut u16,
        max_file_title: Dword,
        initial_dir: *const u16,
        title: *const u16,
        flags: Dword,
        file_offset: u16,
        file_extension: u16,
        default_extension: *const u16,
        custom_data: Lparam,
        hook: *mut c_void,
        template_name: *const u16,
        reserved: *mut c_void,
        reserved_flags: Dword,
        flags_ex: Dword,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetModuleHandleW(module_name: *const u16) -> Hinstance;
    }

    #[link(name = "user32")]
    extern "system" {
        fn AppendMenuW(menu: Hmenu, flags: Uint, id: usize, text: *const u16) -> Bool;
        fn CreateAcceleratorTableW(accelerators: *const Accel, count: c_int) -> *mut c_void;
        fn CreateMenu() -> Hmenu;
        fn CreatePopupMenu() -> Hmenu;
        fn CreateWindowExW(
            ex_style: Dword,
            class_name: *const u16,
            window_name: *const u16,
            style: Dword,
            x: c_int,
            y: c_int,
            width: c_int,
            height: c_int,
            parent: Hwnd,
            menu: Hmenu,
            instance: Hinstance,
            param: *mut c_void,
        ) -> Hwnd;
        fn DefWindowProcW(hwnd: Hwnd, message: Uint, w_param: Wparam, l_param: Lparam) -> Lresult;
        fn DestroyAcceleratorTable(accelerator: *mut c_void) -> Bool;
        fn DestroyWindow(hwnd: Hwnd) -> Bool;
        fn DispatchMessageW(message: *const Msg) -> Lresult;
        fn EnableWindow(hwnd: Hwnd, enabled: Bool) -> Bool;
        fn GetClientRect(hwnd: Hwnd, rect: *mut Rect) -> Bool;
        fn GetMessageW(message: *mut Msg, hwnd: Hwnd, min: Uint, max: Uint) -> Bool;
        fn GetWindowLongPtrW(hwnd: Hwnd, index: c_int) -> isize;
        fn GetWindowTextLengthW(hwnd: Hwnd) -> c_int;
        fn GetWindowTextW(hwnd: Hwnd, text: *mut u16, max_count: c_int) -> c_int;
        fn LoadCursorW(instance: Hinstance, cursor_name: *const u16) -> Hcursor;
        fn MessageBoxW(hwnd: Hwnd, text: *const u16, caption: *const u16, kind: Uint) -> c_int;
        fn MoveWindow(
            hwnd: Hwnd,
            x: c_int,
            y: c_int,
            width: c_int,
            height: c_int,
            repaint: Bool,
        ) -> Bool;
        fn PostMessageW(hwnd: Hwnd, message: Uint, w_param: Wparam, l_param: Lparam) -> Bool;
        fn PostQuitMessage(exit_code: c_int);
        fn RegisterClassW(class: *const WndClassW) -> u16;
        fn SendMessageW(hwnd: Hwnd, message: Uint, w_param: Wparam, l_param: Lparam) -> Lresult;
        fn SetFocus(hwnd: Hwnd) -> Hwnd;
        fn SetMenu(hwnd: Hwnd, menu: Hmenu) -> Bool;
        fn SetWindowLongPtrW(hwnd: Hwnd, index: c_int, value: isize) -> isize;
        fn SetWindowTextW(hwnd: Hwnd, text: *const u16) -> Bool;
        fn ShowWindow(hwnd: Hwnd, command: c_int) -> Bool;
        fn TranslateMessage(message: *const Msg) -> Bool;
        fn TranslateAcceleratorW(
            hwnd: Hwnd,
            accelerator: *mut c_void,
            message: *const Msg,
        ) -> c_int;
        fn UpdateWindow(hwnd: Hwnd) -> Bool;
    }

    #[link(name = "gdi32")]
    extern "system" {
        fn CreateFontW(
            height: c_int,
            width: c_int,
            escapement: c_int,
            orientation: c_int,
            weight: c_int,
            italic: Dword,
            underline: Dword,
            strike_out: Dword,
            charset: Dword,
            output_precision: Dword,
            clip_precision: Dword,
            quality: Dword,
            pitch_and_family: Dword,
            face: *const u16,
        ) -> Hfont;
        fn DeleteObject(object: *mut c_void) -> Bool;
    }

    #[link(name = "comdlg32")]
    extern "system" {
        fn GetOpenFileNameW(file_name: *mut OpenFileNameW) -> Bool;
        fn GetSaveFileNameW(file_name: *mut OpenFileNameW) -> Bool;
    }

    const CLASS_NAME: &str = "APLStudioWindow";
    const APP_TITLE: &str = "APL Studio";
    const STARTER_SOURCE: &str = "AVInt x = 4\r\nAVInt y = (x + 2) * 3\r\n\r\nout y\r\n";

    const ID_OPEN: usize = 1001;
    const ID_SAVE: usize = 1002;
    const ID_SAVE_AS: usize = 1003;
    const ID_RUN: usize = 1004;
    const ID_EXIT: usize = 1005;
    const ID_EDITOR: usize = 1101;
    const ID_OUTPUT: usize = 1102;
    const ID_INPUT: usize = 1103;

    const WM_CREATE: Uint = 0x0001;
    const WM_DESTROY: Uint = 0x0002;
    const WM_SIZE: Uint = 0x0005;
    const WM_SETFOCUS: Uint = 0x0007;
    const WM_CLOSE: Uint = 0x0010;
    const WM_COMMAND: Uint = 0x0111;
    const WM_GETMINMAXINFO: Uint = 0x0024;
    const WM_SETFONT: Uint = 0x0030;
    const WM_NCCREATE: Uint = 0x0081;
    const WM_NCDESTROY: Uint = 0x0082;
    const WM_APP_RUN_COMPLETE: Uint = 0x8001;

    const GWLP_USERDATA: c_int = -21;
    const EN_CHANGE: usize = 0x0300;
    const BN_CLICKED: usize = 0;

    const WS_OVERLAPPEDWINDOW: Dword = 0x00cf_0000;
    const WS_CHILD: Dword = 0x4000_0000;
    const WS_VISIBLE: Dword = 0x1000_0000;
    const WS_BORDER: Dword = 0x0080_0000;
    const WS_VSCROLL: Dword = 0x0020_0000;
    const WS_HSCROLL: Dword = 0x0010_0000;
    const WS_TABSTOP: Dword = 0x0001_0000;
    const ES_MULTILINE: Dword = 0x0004;
    const ES_AUTOVSCROLL: Dword = 0x0040;
    const ES_AUTOHSCROLL: Dword = 0x0080;
    const ES_READONLY: Dword = 0x0800;
    const ES_WANTRETURN: Dword = 0x1000;
    const BS_PUSHBUTTON: Dword = 0;
    const SS_LEFT: Dword = 0;
    const CW_USEDEFAULT: c_int = i32::MIN;
    const SW_SHOWDEFAULT: c_int = 10;
    const COLOR_WINDOW: usize = 5;
    const IDC_ARROW: *const u16 = 32512usize as *const u16;
    const FVIRTKEY: u8 = 0x01;
    const FCONTROL: u8 = 0x08;
    const VK_F5: u16 = 0x74;

    const MF_STRING: Uint = 0;
    const MF_POPUP: Uint = 0x0010;
    const MF_SEPARATOR: Uint = 0x0800;

    const MB_ICONERROR: Uint = 0x0010;
    const MB_ICONWARNING: Uint = 0x0030;
    const MB_YESNOCANCEL: Uint = 0x0003;
    const IDYES: c_int = 6;
    const IDNO: c_int = 7;
    const IDCANCEL: c_int = 2;

    const OFN_OVERWRITEPROMPT: Dword = 0x0002;
    const OFN_FILEMUSTEXIST: Dword = 0x1000;
    const OFN_PATHMUSTEXIST: Dword = 0x0800;
    const OFN_EXPLORER: Dword = 0x0008_0000;

    struct AppState {
        editor: Hwnd,
        input: Hwnd,
        input_label: Hwnd,
        output: Hwnd,
        output_label: Hwnd,
        path_label: Hwnd,
        open_button: Hwnd,
        save_button: Hwnd,
        run_button: Hwnd,
        font: Hfont,
        current_path: Option<PathBuf>,
        dirty: bool,
        suppress_change: bool,
        running: bool,
    }

    impl AppState {
        fn new() -> Self {
            Self {
                editor: null_mut(),
                input: null_mut(),
                input_label: null_mut(),
                output: null_mut(),
                output_label: null_mut(),
                path_label: null_mut(),
                open_button: null_mut(),
                save_button: null_mut(),
                run_button: null_mut(),
                font: null_mut(),
                current_path: None,
                dirty: false,
                suppress_change: true,
                running: false,
            }
        }
    }

    impl Drop for AppState {
        fn drop(&mut self) {
            if !self.font.is_null() {
                unsafe { DeleteObject(self.font) };
            }
        }
    }

    pub fn run() {
        unsafe {
            let instance = GetModuleHandleW(null());
            let class_name = wide(CLASS_NAME);
            let class = WndClassW {
                style: 0x0001 | 0x0002,
                wnd_proc: Some(window_proc),
                cls_extra: 0,
                wnd_extra: 0,
                instance,
                icon: null_mut(),
                cursor: LoadCursorW(null_mut(), IDC_ARROW),
                background: (COLOR_WINDOW + 1) as Hbrush,
                menu_name: null(),
                class_name: class_name.as_ptr(),
            };

            if RegisterClassW(&class) == 0 {
                show_error(
                    null_mut(),
                    "Could not register the APL Studio window class.",
                );
                return;
            }

            let state = Box::new(AppState::new());
            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                wide(APP_TITLE).as_ptr(),
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                1050,
                760,
                null_mut(),
                null_mut(),
                instance,
                Box::into_raw(state).cast(),
            );

            if hwnd.is_null() {
                show_error(null_mut(), "Could not create the APL Studio window.");
                return;
            }

            ShowWindow(hwnd, SW_SHOWDEFAULT);
            UpdateWindow(hwnd);

            let accelerators = [
                Accel {
                    virtual_key: FVIRTKEY | FCONTROL,
                    key: b'O' as u16,
                    command: ID_OPEN as u16,
                },
                Accel {
                    virtual_key: FVIRTKEY | FCONTROL,
                    key: b'S' as u16,
                    command: ID_SAVE as u16,
                },
                Accel {
                    virtual_key: FVIRTKEY,
                    key: VK_F5,
                    command: ID_RUN as u16,
                },
            ];
            let accelerator =
                CreateAcceleratorTableW(accelerators.as_ptr(), accelerators.len() as c_int);

            let mut message: Msg = zeroed();
            while GetMessageW(&mut message, null_mut(), 0, 0) > 0 {
                if accelerator.is_null() || TranslateAcceleratorW(hwnd, accelerator, &message) == 0
                {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
            if !accelerator.is_null() {
                DestroyAcceleratorTable(accelerator);
            }
        }
    }

    unsafe extern "system" fn window_proc(
        hwnd: Hwnd,
        message: Uint,
        w_param: Wparam,
        l_param: Lparam,
    ) -> Lresult {
        if message == WM_NCCREATE {
            let create = &*(l_param as *const CreateStructW);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.create_params as isize);
        }

        let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppState;

        match message {
            WM_CREATE => {
                if state_ptr.is_null() {
                    return -1;
                }
                create_ui(hwnd, &mut *state_ptr);
                0
            }
            WM_SIZE => {
                if !state_ptr.is_null() {
                    layout(hwnd, &*state_ptr);
                }
                0
            }
            WM_GETMINMAXINFO => {
                let info = &mut *(l_param as *mut MinMaxInfo);
                info.min_track_size.x = 640;
                info.min_track_size.y = 480;
                0
            }
            WM_SETFOCUS => {
                if !state_ptr.is_null() {
                    SetFocus((*state_ptr).editor);
                }
                0
            }
            WM_COMMAND => {
                if !state_ptr.is_null() {
                    let id = w_param & 0xffff;
                    let notification = (w_param >> 16) & 0xffff;
                    handle_command(hwnd, &mut *state_ptr, id, notification);
                }
                0
            }
            WM_APP_RUN_COMPLETE => {
                if !state_ptr.is_null() && l_param != 0 {
                    let result = Box::from_raw(l_param as *mut String);
                    let state = &mut *state_ptr;
                    set_text(state.output, &result);
                    state.running = false;
                    EnableWindow(state.run_button, 1);
                    set_text(state.output_label, "Output");
                }
                0
            }
            WM_CLOSE => {
                if state_ptr.is_null() || confirm_discard(hwnd, &mut *state_ptr) {
                    DestroyWindow(hwnd);
                }
                0
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }
            WM_NCDESTROY => {
                if !state_ptr.is_null() {
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                    drop(Box::from_raw(state_ptr));
                }
                DefWindowProcW(hwnd, message, w_param, l_param)
            }
            _ => DefWindowProcW(hwnd, message, w_param, l_param),
        }
    }

    unsafe fn create_ui(hwnd: Hwnd, state: &mut AppState) {
        let instance = GetModuleHandleW(null());
        let menu = CreateMenu();
        let file_menu = CreatePopupMenu();
        AppendMenuW(
            file_menu,
            MF_STRING,
            ID_OPEN,
            wide("&Open...\tCtrl+O").as_ptr(),
        );
        AppendMenuW(
            file_menu,
            MF_STRING,
            ID_SAVE,
            wide("&Save\tCtrl+S").as_ptr(),
        );
        AppendMenuW(
            file_menu,
            MF_STRING,
            ID_SAVE_AS,
            wide("Save &As...").as_ptr(),
        );
        AppendMenuW(file_menu, MF_SEPARATOR, 0, null());
        AppendMenuW(file_menu, MF_STRING, ID_EXIT, wide("E&xit").as_ptr());
        AppendMenuW(menu, MF_POPUP, file_menu as usize, wide("&File").as_ptr());
        AppendMenuW(menu, MF_STRING, ID_RUN, wide("&Run").as_ptr());
        SetMenu(hwnd, menu);

        state.open_button = create_control(
            instance,
            hwnd,
            "BUTTON",
            "Open",
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_PUSHBUTTON,
            ID_OPEN,
        );
        state.save_button = create_control(
            instance,
            hwnd,
            "BUTTON",
            "Save",
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_PUSHBUTTON,
            ID_SAVE,
        );
        state.run_button = create_control(
            instance,
            hwnd,
            "BUTTON",
            "Run  F5",
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_PUSHBUTTON,
            ID_RUN,
        );
        state.path_label = create_control(
            instance,
            hwnd,
            "STATIC",
            "Untitled.apl",
            WS_CHILD | WS_VISIBLE | SS_LEFT,
            0,
        );
        state.editor = create_control(
            instance,
            hwnd,
            "EDIT",
            STARTER_SOURCE,
            WS_CHILD
                | WS_VISIBLE
                | WS_BORDER
                | WS_TABSTOP
                | WS_VSCROLL
                | WS_HSCROLL
                | ES_MULTILINE
                | ES_AUTOVSCROLL
                | ES_AUTOHSCROLL
                | ES_WANTRETURN,
            ID_EDITOR,
        );
        state.input_label = create_control(
            instance,
            hwnd,
            "STATIC",
            "Input",
            WS_CHILD | WS_VISIBLE | SS_LEFT,
            0,
        );
        state.input = create_control(
            instance,
            hwnd,
            "EDIT",
            "",
            WS_CHILD
                | WS_VISIBLE
                | WS_BORDER
                | WS_TABSTOP
                | WS_VSCROLL
                | ES_MULTILINE
                | ES_AUTOVSCROLL
                | ES_WANTRETURN,
            ID_INPUT,
        );
        state.output_label = create_control(
            instance,
            hwnd,
            "STATIC",
            "Output",
            WS_CHILD | WS_VISIBLE | SS_LEFT,
            0,
        );
        state.output = create_control(
            instance,
            hwnd,
            "EDIT",
            "Ready.",
            WS_CHILD
                | WS_VISIBLE
                | WS_BORDER
                | WS_TABSTOP
                | WS_VSCROLL
                | ES_MULTILINE
                | ES_AUTOVSCROLL
                | ES_READONLY,
            ID_OUTPUT,
        );

        let face = wide("Consolas");
        state.font = CreateFontW(-17, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 5, 0x31, face.as_ptr());

        for control in [
            state.editor,
            state.input,
            state.input_label,
            state.output,
            state.path_label,
            state.output_label,
            state.open_button,
            state.save_button,
            state.run_button,
        ] {
            SendMessageW(control, WM_SETFONT, state.font as Wparam, 1);
        }

        state.suppress_change = false;
        update_title(hwnd, state);
        layout(hwnd, state);
        SetFocus(state.editor);
    }

    unsafe fn create_control(
        instance: Hinstance,
        parent: Hwnd,
        class: &str,
        text: &str,
        style: Dword,
        id: usize,
    ) -> Hwnd {
        CreateWindowExW(
            0,
            wide(class).as_ptr(),
            wide(text).as_ptr(),
            style,
            0,
            0,
            0,
            0,
            parent,
            id as Hmenu,
            instance,
            null_mut(),
        )
    }

    unsafe fn layout(hwnd: Hwnd, state: &AppState) {
        let mut rect: Rect = zeroed();
        GetClientRect(hwnd, &mut rect);
        let width = (rect.right - rect.left).max(1);
        let height = (rect.bottom - rect.top).max(1);
        let margin = 10;
        let toolbar_height = 40;
        let button_width = 82;
        let button_height = 30;
        let output_height = (height / 4).clamp(110, 210);
        let input_height = (height / 7).clamp(65, 120);
        let section_label_height = 25;
        let editor_top = margin + toolbar_height;
        let output_top = height - margin - output_height;
        let input_top = output_top - section_label_height - 6 - input_height;
        let editor_height = (input_top - section_label_height - editor_top - 6).max(80);

        MoveWindow(
            state.open_button,
            margin,
            margin,
            button_width,
            button_height,
            1,
        );
        MoveWindow(
            state.save_button,
            margin + button_width + 8,
            margin,
            button_width,
            button_height,
            1,
        );
        MoveWindow(
            state.run_button,
            margin + (button_width + 8) * 2,
            margin,
            button_width + 12,
            button_height,
            1,
        );
        MoveWindow(
            state.path_label,
            margin + (button_width + 8) * 3 + 12,
            margin + 6,
            (width - 320).max(100),
            24,
            1,
        );
        MoveWindow(
            state.editor,
            margin,
            editor_top,
            width - margin * 2,
            editor_height,
            1,
        );
        MoveWindow(
            state.input_label,
            margin,
            input_top - section_label_height,
            width - margin * 2,
            section_label_height,
            1,
        );
        MoveWindow(
            state.input,
            margin,
            input_top,
            width - margin * 2,
            input_height,
            1,
        );
        MoveWindow(
            state.output_label,
            margin,
            output_top - section_label_height,
            width - margin * 2,
            section_label_height,
            1,
        );
        MoveWindow(
            state.output,
            margin,
            output_top,
            width - margin * 2,
            output_height,
            1,
        );
    }

    unsafe fn handle_command(hwnd: Hwnd, state: &mut AppState, id: usize, notification: usize) {
        if id == ID_EDITOR && notification == EN_CHANGE && !state.suppress_change {
            state.dirty = true;
            update_title(hwnd, state);
            return;
        }

        if notification != BN_CLICKED && notification != 0 {
            return;
        }

        match id {
            ID_OPEN => open_file(hwnd, state),
            ID_SAVE => {
                save_file(hwnd, state, false);
            }
            ID_SAVE_AS => {
                save_file(hwnd, state, true);
            }
            ID_RUN => run_source(hwnd, state),
            ID_EXIT => {
                if confirm_discard(hwnd, state) {
                    DestroyWindow(hwnd);
                }
            }
            _ => {}
        }
    }

    unsafe fn open_file(hwnd: Hwnd, state: &mut AppState) {
        if !confirm_discard(hwnd, state) {
            return;
        }
        let Some(path) = choose_file(hwnd, false) else {
            return;
        };

        match fs::read_to_string(&path) {
            Ok(source) => {
                state.suppress_change = true;
                set_text(state.editor, &source);
                state.suppress_change = false;
                state.current_path = Some(path);
                state.dirty = false;
                set_text(state.output, "Ready.");
                update_title(hwnd, state);
            }
            Err(error) => show_error(hwnd, &format!("Could not open the file:\r\n{error}")),
        }
    }

    unsafe fn save_file(hwnd: Hwnd, state: &mut AppState, save_as: bool) -> bool {
        let path = if save_as || state.current_path.is_none() {
            let Some(path) = choose_file(hwnd, true) else {
                return false;
            };
            path
        } else {
            state.current_path.clone().unwrap()
        };

        let source = get_text(state.editor);
        match fs::write(&path, source.as_bytes()) {
            Ok(()) => {
                state.current_path = Some(path);
                state.dirty = false;
                update_title(hwnd, state);
                true
            }
            Err(error) => {
                show_error(hwnd, &format!("Could not save the file:\r\n{error}"));
                false
            }
        }
    }

    unsafe fn confirm_discard(hwnd: Hwnd, state: &mut AppState) -> bool {
        if !state.dirty {
            return true;
        }

        let answer = MessageBoxW(
            hwnd,
            wide("Save changes before continuing?").as_ptr(),
            wide(APP_TITLE).as_ptr(),
            MB_YESNOCANCEL | MB_ICONWARNING,
        );
        match answer {
            IDYES => save_file(hwnd, state, false),
            IDNO => true,
            IDCANCEL => false,
            _ => false,
        }
    }

    unsafe fn run_source(hwnd: Hwnd, state: &mut AppState) {
        if state.running {
            return;
        }

        if state.current_path.is_some() && state.dirty && !save_file(hwnd, state, false) {
            return;
        }

        let source = get_text(state.editor);
        let input = parse_input(&get_text(state.input));
        state.running = true;
        EnableWindow(state.run_button, 0);
        set_text(state.output_label, "Running...");
        set_text(state.output, "Checking and running APL source...");
        let window = hwnd as isize;

        thread::spawn(move || {
            let result = execute_source(&source, input);
            let result_ptr = Box::into_raw(Box::new(result));
            let posted = unsafe {
                PostMessageW(window as Hwnd, WM_APP_RUN_COMPLETE, 0, result_ptr as Lparam)
            };
            if posted == 0 {
                unsafe { drop(Box::from_raw(result_ptr)) };
            }
        });
    }

    fn execute_source(source: &str, input: Vec<String>) -> String {
        let source = apl_compiler::compose_program(apl_compiler::STANDARD_PRELUDE, source);
        let program = match apl_parser::parse_program(&source) {
            Ok(program) => program,
            Err(error) => return format!("Parse error:\r\n{}", error.message),
        };

        if let Err(error) = apl_core::validate_program(&program) {
            return format!("Check error:\r\n{error:?}");
        }

        match apl_runtime::run_program(&program, input) {
            Ok(output) if output.stdout.is_empty() => "Program finished with no output.".to_owned(),
            Ok(output) => output.stdout,
            Err(apl_runtime::RuntimeError::Failed(message)) => {
                format!("APL fail:\r\n{message}")
            }
            Err(error) => format!("Runtime error:\r\n{error:?}"),
        }
    }

    fn parse_input(text: &str) -> Vec<String> {
        let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
        if normalized.is_empty() {
            Vec::new()
        } else {
            normalized.split('\n').map(str::to_owned).collect()
        }
    }

    unsafe fn choose_file(hwnd: Hwnd, save: bool) -> Option<PathBuf> {
        let mut buffer = vec![0u16; 32_768];
        let filter = wide("APL source (*.apl)\0*.apl\0All files (*.*)\0*.*\0");
        let extension = wide("apl");
        let mut dialog: OpenFileNameW = zeroed();
        dialog.struct_size = size_of::<OpenFileNameW>() as Dword;
        dialog.owner = hwnd;
        dialog.filter = filter.as_ptr();
        dialog.filter_index = 1;
        dialog.file = buffer.as_mut_ptr();
        dialog.max_file = buffer.len() as Dword;
        dialog.default_extension = extension.as_ptr();
        dialog.flags = OFN_EXPLORER | OFN_PATHMUSTEXIST;
        if save {
            dialog.flags |= OFN_OVERWRITEPROMPT;
        } else {
            dialog.flags |= OFN_FILEMUSTEXIST;
        }

        let accepted = if save {
            GetSaveFileNameW(&mut dialog)
        } else {
            GetOpenFileNameW(&mut dialog)
        };
        if accepted == 0 {
            return None;
        }

        let length = buffer.iter().position(|value| *value == 0).unwrap_or(0);
        Some(PathBuf::from(String::from_utf16_lossy(&buffer[..length])))
    }

    unsafe fn update_title(hwnd: Hwnd, state: &AppState) {
        let name = state
            .current_path
            .as_ref()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("Untitled.apl");
        let marker = if state.dirty { "*" } else { "" };
        set_text(hwnd, &format!("{marker}{name} - {APP_TITLE}"));
        set_text(state.path_label, &format!("{marker}{name}"));
    }

    unsafe fn get_text(hwnd: Hwnd) -> String {
        let length = GetWindowTextLengthW(hwnd).max(0) as usize;
        let mut buffer = vec![0u16; length + 1];
        let copied =
            GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as c_int).max(0) as usize;
        String::from_utf16_lossy(&buffer[..copied])
    }

    unsafe fn set_text(hwnd: Hwnd, text: &str) {
        let normalized = text.replace("\r\n", "\n").replace('\n', "\r\n");
        SetWindowTextW(hwnd, wide(&normalized).as_ptr());
    }

    unsafe fn show_error(hwnd: Hwnd, message: &str) {
        MessageBoxW(
            hwnd,
            wide(message).as_ptr(),
            wide(APP_TITLE).as_ptr(),
            MB_ICONERROR,
        );
    }

    fn wide(value: &str) -> Vec<u16> {
        OsStr::new(value).encode_wide().chain(Some(0)).collect()
    }

    #[cfg(test)]
    mod tests {
        use super::{execute_source, parse_input};

        #[test]
        fn executes_source_from_editor_buffer() {
            assert_eq!(
                execute_source("AVInt value = 6 * 7\nout value", Vec::new()),
                "42\n"
            );
        }

        #[test]
        fn reports_invalid_source_in_output_panel() {
            assert!(execute_source("AVInt = 4", Vec::new()).starts_with("Parse error:"));
        }

        #[test]
        fn passes_editor_input_lines_to_runtime() {
            let input = parse_input("41\r\nhello\r\n");
            assert_eq!(input, vec!["41", "hello", ""]);
            assert_eq!(
                execute_source(
                    "AVInt number = input\nAVStr word = input\nout number\nout word",
                    input,
                ),
                "41\nhello\n"
            );
        }
    }
}

#[cfg(windows)]
fn main() {
    windows_app::run();
}
