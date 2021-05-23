fn main() {
    windows::build!(
        Windows::Win32::System::SystemServices::{
            PWSTR,
            HINSTANCE, GetModuleHandleW,
            LRESULT, S_OK, S_FALSE, TRUE, FALSE,
            DPI_AWARENESS_CONTEXT,
            D2DERR_RECREATE_TARGET,
            BS_OWNERDRAW,
        },
        Windows::Win32::System::Com::{
            CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
            CoTaskMemFree, 
        },
        Windows::Win32::System::Diagnostics::Debug::{
            ERROR_CANCELLED
        },
        Windows::Foundation::Numerics::{
            Matrix3x2,
        },
        Windows::Win32::UI::WindowsAndMessaging::{
            WNDCLASSEXW, CS_HREDRAW, CS_VREDRAW,
            RegisterClassExW,
            CreateWindowExW, CW_USEDEFAULT, WS_VISIBLE, WS_OVERLAPPEDWINDOW, WS_CHILD, WS_TABSTOP, WS_CLIPCHILDREN, WS_CLIPSIBLINGS,
            ShowWindow, SW_SHOW, DestroyWindow, 
            SetWindowPos, SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, SWP_NOSENDCHANGING,
            CreateMenu, CreatePopupMenu, AppendMenuW, SetMenu, DeleteMenu, DestroyMenu, GetMenu, EnableMenuItem,
            MENU_ITEM_FLAGS, MF_POPUP, MF_STRING, MF_ENABLED, MF_DISABLED, MF_SEPARATOR, MF_BYPOSITION,
            MessageBoxW, MB_OK,
            GetClientRect,

            ODS_DISABLED, ODS_SELECTED, ODS_FOCUS,

            MSG, GetMessageW, TranslateMessage, DispatchMessageW, PostQuitMessage, PostMessageW,
            DefWindowProcW, HWND, WPARAM, LPARAM,
            SetWindowLongPtrW, GetWindowLongPtrW, GWLP_USERDATA, GWLP_HINSTANCE, CREATESTRUCTW,
            WM_NCCREATE, WM_CREATE, WM_CLOSE, WM_DESTROY, WM_COMMAND, WM_PAINT, WM_SIZE, WM_DPICHANGED, WM_DISPLAYCHANGE, WM_DRAWITEM,
        },
        Windows::Win32::UI::MenusAndResources::{
            HMENU,
        },
        Windows::Win32::UI::Controls::{
            DRAWITEMSTRUCT,
        },
        Windows::Win32::UI::KeyboardAndMouseInput::{
            EnableWindow,
        },
        Windows::Win32::UI::Shell::{
            IShellItem, IFileOpenDialog, IFileSaveDialog, FileOpenDialog, FileSaveDialog, SIGDN_FILESYSPATH, COMDLG_FILTERSPEC
        },
        Windows::Win32::UI::DisplayDevices::{
            RECT,
        },
        Windows::Win32::UI::HiDpi::{
            SetProcessDpiAwarenessContext,
        },
        Windows::Win32::Graphics::Gdi::{
            HDC, PAINTSTRUCT, HBRUSH, BeginPaint, EndPaint, InvalidateRect,
        },
        Windows::Win32::Graphics::Direct2D::{
            ID2D1Factory, D2D1CreateFactory,
            ID2D1HwndRenderTarget, ID2D1DCRenderTarget, ID2D1SolidColorBrush, ID2D1StrokeStyle,
            D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_FACTORY_OPTIONS,
            D2D1_RENDER_TARGET_PROPERTIES, D2D1_PIXEL_FORMAT, D2D1_RENDER_TARGET_TYPE_DEFAULT, D2D1_ALPHA_MODE_IGNORE, D2D1_RENDER_TARGET_USAGE_GDI_COMPATIBLE,
            D2D1_HWND_RENDER_TARGET_PROPERTIES, D2D1_PRESENT_OPTIONS_IMMEDIATELY, D2D1_PRESENT_OPTIONS_NONE,
            D2D1_BRUSH_PROPERTIES,
            D2D_SIZE_U, D2D_SIZE_F, D2D1_COLOR_F,
            D2D1_ELLIPSE, D2D_RECT_F,
        },
        Windows::Win32::Graphics::Dxgi::{
            DXGI_FORMAT_B8G8R8A8_UNORM,
        }
    );
}