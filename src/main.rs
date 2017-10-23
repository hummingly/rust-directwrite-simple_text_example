extern crate gdi32;
extern crate kernel32;
extern crate user32;
extern crate winapi;
mod utils;

use utils::*;

use winapi::*;
use user32::*;
use kernel32::*;
use gdi32::GetDeviceCaps;

use std::ptr::{null, null_mut};
use std::mem;

//STRUCTURES
pub struct SimpleText {
    hwnd: HWND,
    wtext: Vec<u16>,
    wtext_length: u32,
    brush: *mut ID2D1SolidColorBrush,
    render_target: *mut ID2D1HwndRenderTarget,
    d2d1_factory: *mut ID2D1Factory,
    dwrite_factory: *mut IDWriteFactory,
    text_format: *mut IDWriteTextFormat,
}

impl SimpleText {
    fn initialize() -> Self {
        SimpleText {
            hwnd: null_mut(),
            wtext: Vec::new(),
            wtext_length: 0,
            brush: null_mut(),
            render_target: null_mut(),
            d2d1_factory: null_mut(),
            dwrite_factory: null_mut(),
            text_format: null_mut(),
        }
    }
}

//D2D1 SETUP
fn create_directx_resources(app: &mut SimpleText) {
    unsafe {
        let mut d2_factory: *mut c_void = null_mut();
        let factory_options = D2D1_FACTORY_OPTIONS {
            debugLevel: D2D1_DEBUG_LEVEL_NONE,
        };

        let d2d1_factory = create_d2d1_factory(
            D2D1_FACTORY_TYPE_MULTI_THREADED,
            &UuidOfID2D1Factory,
            &factory_options as *const D2D1_FACTORY_OPTIONS,
            &mut d2_factory,
        );

        if d2d1_factory != S_OK {
            error_msgbox("Could not create D2D1 factory.");
        } else {
            app.d2d1_factory = d2_factory as *mut ID2D1Factory;
        }

        let mut dw_factory: *mut IUnknown = null_mut();

        let dwrite_factory = create_dwrite_factory(
            DWRITE_FACTORY_TYPE_SHARED,
            &UuidOfIDWriteFactory,
            &mut dw_factory,
        );

        if dwrite_factory != S_OK {
            error_msgbox("Could not create Dwrite factory.");
        } else {
            app.dwrite_factory = dw_factory as *mut IDWriteFactory;
        }

        let text = "Hello World using DirectWrite!";
        app.wtext_length = text.len() as u32;
        app.wtext = text.to_wide();

        let dwrite_factory: &mut IDWriteFactory = &mut *app.dwrite_factory;

        if dwrite_factory.CreateTextFormat(
            "Gabriola".to_wide().as_ptr(),
            null_mut(),
            DWRITE_FONT_WEIGHT_REGULAR,
            DWRITE_FONT_STYLE_NORMAL,
            DWRITE_FONT_STRETCH_NORMAL,
            72.0,
            "en-us".to_wide().as_ptr(),
            &mut app.text_format,
        ) != S_OK
        {
            error_msgbox("Could not create text format.");
        }

        let text_format: &mut IDWriteTextFormat = &mut *app.text_format;

        if text_format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER) != S_OK {
            error_msgbox("Failed to center text horizontally.");
        }

        if text_format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER) != S_OK {
            error_msgbox("Failed to center text vertically.");
        }
    }
}

fn set_d2d_resources(app: &mut SimpleText) {
    unsafe {
        if app.d2d1_factory.is_null() {
            error_msgbox("There is nothing to render!");
        } else {
            let mut rect: RECT = WinStruct::default();

            GetClientRect(app.hwnd, &mut rect as *mut RECT);

            let d2d_rect = D2D1_SIZE_U {
                width: (rect.right - rect.left) as u32,
                height: (rect.bottom - rect.top) as u32,
            };

            let render_properties: D2D1_RENDER_TARGET_PROPERTIES = WinStruct::default();

            let hwnd_render_properties = D2D1_HWND_RENDER_TARGET_PROPERTIES {
                hwnd: app.hwnd,
                pixelSize: d2d_rect,
                presentOptions: D2D1_PRESENT_OPTIONS_NONE,
            };

            let factory: &mut ID2D1Factory = &mut *app.d2d1_factory;

            if factory.CreateHwndRenderTarget(
                &render_properties,
                &hwnd_render_properties,
                &mut app.render_target,
            ) != S_OK
            {
                error_msgbox("Could not create render target!");
            }

            let render_target: &mut ID2D1HwndRenderTarget = &mut *app.render_target;

            let black = Brush::solid_color(0.0, 0.0, 0.0);

            if render_target.CreateSolidColorBrush(&black, null(), &mut app.brush) != S_OK {
                error_msgbox("Could not create brush!");
            }
        }
    }
}

//RENDER METHOD
fn on_paint(app: &mut SimpleText) -> HRESULT {
    unsafe {
        let d2d1_matrix: D2D1_MATRIX_3X2_F = WinStruct::default();
        let mut rect: RECT = WinStruct::default();
        GetClientRect(app.hwnd, &mut rect as *mut RECT);

        let screen = GetDC(null_mut());
        let dpi_scale_x = GetDeviceCaps(screen, LOGPIXELSX) / 96;
        let dpi_scale_y = GetDeviceCaps(screen, LOGPIXELSY) / 96;

        let layout_rect = D2D1_RECT_F {
            left: (rect.left / dpi_scale_x) as f32,
            top: (rect.top / dpi_scale_y) as f32,
            right: ((rect.right - rect.left) / dpi_scale_x) as f32,
            bottom: ((rect.bottom - rect.top) / dpi_scale_y) as f32,
        };

        let white = Brush::solid_color(255.0, 255.0, 255.0);

        let render = &mut *app.render_target;
        render.BeginDraw();

        render.SetTransform(&d2d1_matrix);

        render.Clear(&white);

        render.DrawText(
            app.wtext.as_ptr(),
            app.wtext_length,
            app.text_format,
            &layout_rect,
            &mut **app.brush as *mut ID2D1Brush,
            D2D1_DRAW_TEXT_OPTIONS(0),
            DWRITE_MEASURING_MODE(0),
        );

        render.EndDraw(null_mut(), null_mut())
    }
}

//RELEASE RESOURCES
fn safe_release(app: &mut SimpleText) {
    unsafe {
        if !app.render_target.is_null() {
            (*app.brush).Release();
            (*app.render_target).Release();

            app.brush = null_mut();
            app.render_target = null_mut();
        }
    }
}

fn release_resources(app: &mut SimpleText) {
    unsafe {
        safe_release(app);

        if !app.dwrite_factory.is_null() {
            (*app.d2d1_factory).Release();
            app.d2d1_factory = null_mut();
        }

        if !app.dwrite_factory.is_null() {
            (*app.dwrite_factory).Release();
            (*app.text_format).Release();

            app.dwrite_factory = null_mut();
            app.text_format = null_mut();
        }
    }
}

//MESSAGE PROCESSING
unsafe extern "system" fn wndproc(
    hwnd: HWND,
    message: UINT32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let app_ptr = GetWindowLongPtrW(hwnd, 0) as *mut SimpleText;
    let mut app: &mut SimpleText = &mut *(app_ptr as *mut SimpleText);
    match message {
        WM_PAINT => {
            set_d2d_resources(app);
            if on_paint(app) == D2DERR_RECREATE_TARGET {
                safe_release(app);
            }
            0
        }
        WM_SIZE => {
            let width = GET_X_LPARAM(lparam);
            let height = GET_Y_LPARAM(lparam);

            if !app_ptr.is_null() {
                let render_size = D2D_SIZE_U {
                    width: width as u32,
                    height: height as u32,
                };

                let render = &mut *app.render_target;
                render.Resize(&render_size);
            }
            0
        }
        WM_DESTROY => {
            release_resources(&mut app);
            PostQuitMessage(0);
            1
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}

//WINDOW CREATION
pub fn init_class() {
    unsafe {
        let class = "directwrite_example".to_wide();
        let wndcl = WNDCLASSEXW {
            cbSize: mem::size_of::<WNDCLASSEXW>() as UINT32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: mem::size_of::<SimpleText>() as INT32,
            hInstance: GetModuleHandleW(null_mut()),
            hIcon: 0 as HICON,
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hbrBackground: COLOR_WINDOWFRAME as HBRUSH,
            lpszMenuName: null(),
            lpszClassName: class.as_ptr() as *const u16,
            hIconSm: 0 as HICON,
        };

        if RegisterClassExW(&wndcl) == 0 {
            error_msgbox("Could not register class!");
            PostQuitMessage(0);
        } else {
            RegisterClassExW(&wndcl);
        };
    }
}

fn create_window(app: &mut SimpleText, class: &[u16], window: &[u16]) {
    unsafe {
        let hwnd = CreateWindowExW(
            WS_EX_COMPOSITED,
            class.as_ptr(),
            window.as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            600,
            400,
            null_mut(),
            null_mut(),
            GetModuleHandleW(null_mut()),
            null_mut(),
        );

        if hwnd.is_null() {
            error_msgbox("Could not create window!");
            PostQuitMessage(0);
        } else {
            app.hwnd = hwnd;
        }
    }
}

//ASSOCIATE STRUCTURES/DATA
fn set_window(app: &mut SimpleText) {
    unsafe {
        SetWindowLongPtrW(app.hwnd, 0, app as *mut SimpleText as LONG_PTR);
    }
}

fn main() {
    unsafe {
        let mut app = SimpleText::initialize();

        let class = "directwrite_example".to_wide();
        let window = "Hello World!".to_wide();

        init_class();
        create_window(&mut app, &class, &window);
        set_window(&mut app);

        create_directx_resources(&mut app);
        set_d2d_resources(&mut app);

        let mut msg: MSG = WinStruct::default();

        while GetMessageW(&mut msg as *mut MSG, 0 as HWND, 0, 0) != 0 {
            TranslateMessage(&msg as *const MSG);
            DispatchMessageW(&msg as *const MSG);
        }
    }
}
