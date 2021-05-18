fn main() {
    windows::build!(
        Windows::Win32::System::SystemServices::{
            PWSTR,
            HINSTANCE, GetModuleHandleW,
            LRESULT, S_OK, S_FALSE, TRUE, FALSE,
            DPI_AWARENESS_CONTEXT,
        },
        Windows::Win32::System::Com::{
            CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
            CoTaskMemFree, 
        },
        Windows::Win32::System::Diagnostics::Debug::{
            ERROR_CANCELLED
        },
        Windows::Win32::UI::WindowsAndMessaging::{
            WNDCLASSEXW, CS_HREDRAW, CS_VREDRAW,
            RegisterClassExW,
            CreateWindowExW, CW_USEDEFAULT, WS_VISIBLE, WS_OVERLAPPEDWINDOW,
            ShowWindow, SW_SHOW, DestroyWindow,
            CreateMenu, CreatePopupMenu, AppendMenuW, SetMenu, DeleteMenu, DestroyMenu, GetMenu, EnableMenuItem,
            MENU_ITEM_FLAGS, MF_POPUP, MF_STRING, MF_ENABLED, MF_DISABLED, MF_SEPARATOR, MF_BYPOSITION,
            MessageBoxW, MB_OK,

            MSG, GetMessageW, TranslateMessage, DispatchMessageW, PostQuitMessage, PostMessageW,
            DefWindowProcW, HWND, WPARAM, LPARAM,
            SetWindowLongPtrW, GetWindowLongPtrW, GWLP_USERDATA, CREATESTRUCTW,
            WM_NCCREATE, WM_CREATE, WM_CLOSE, WM_DESTROY, WM_COMMAND,
        },
        Windows::Win32::UI::MenusAndResources::{
            HMENU,
        },
        Windows::Win32::UI::Shell::{
            IShellItem, IFileOpenDialog, IFileSaveDialog, FileOpenDialog, FileSaveDialog, SIGDN_FILESYSPATH, COMDLG_FILTERSPEC
        },
        Windows::Win32::UI::HiDpi::{
            SetProcessDpiAwarenessContext,
        },
    );
}