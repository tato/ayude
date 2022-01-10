// imgui-sys 0.7.0 https://github.com/imgui-rs/imgui-rs/tree/v0.7.0/imgui-sys/third-party
// winit support https://github.com/imgui-rs/imgui-rs/blob/v0.7.0/imgui-winit-support/src/lib.rs#L406
// dear imgui 1.80 https://github.com/ocornut/imgui/tree/58075c4414b985b352d10718b02a8c43f25efd7c
// imgui.h 1.80 https://github.com/ocornut/imgui/blob/58075c4414b985b352d10718b02a8c43f25efd7c/imgui.h#L251
// programmer guide 1.80 https://github.com/ocornut/imgui/blob/58075c4414b985b352d10718b02a8c43f25efd7c/imgui.cpp#L169

pub struct ImGui {}

impl ImGui {
    pub fn init() -> Self {
        unsafe {
            imgui_sys::igCreateContext(std::ptr::null_mut());

            // let io = &*imgui_sys::igGetIO();

            // io.backend_flags.insert(BackendFlags::HAS_MOUSE_CURSORS);
            // io.backend_flags.insert(BackendFlags::HAS_SET_MOUSE_POS);
            // io[Key::Tab] = VirtualKeyCode::Tab as _;
            // io[Key::LeftArrow] = VirtualKeyCode::Left as _;
            // io[Key::RightArrow] = VirtualKeyCode::Right as _;
            // io[Key::UpArrow] = VirtualKeyCode::Up as _;
            // io[Key::DownArrow] = VirtualKeyCode::Down as _;
            // io[Key::PageUp] = VirtualKeyCode::PageUp as _;
            // io[Key::PageDown] = VirtualKeyCode::PageDown as _;
            // io[Key::Home] = VirtualKeyCode::Home as _;
            // io[Key::End] = VirtualKeyCode::End as _;
            // io[Key::Insert] = VirtualKeyCode::Insert as _;
            // io[Key::Delete] = VirtualKeyCode::Delete as _;
            // io[Key::Backspace] = VirtualKeyCode::Back as _;
            // io[Key::Space] = VirtualKeyCode::Space as _;
            // io[Key::Enter] = VirtualKeyCode::Return as _;
            // io[Key::Escape] = VirtualKeyCode::Escape as _;
            // io[Key::KeyPadEnter] = VirtualKeyCode::NumpadEnter as _;
            // io[Key::A] = VirtualKeyCode::A as _;
            // io[Key::C] = VirtualKeyCode::C as _;
            // io[Key::V] = VirtualKeyCode::V as _;
            // io[Key::X] = VirtualKeyCode::X as _;
            // io[Key::Y] = VirtualKeyCode::Y as _;
            // io[Key::Z] = VirtualKeyCode::Z as _;

            // pub fn attach_window(&mut self, io: &mut Io, window: &Window, hidpi_mode: HiDpiMode) {
            //     let (hidpi_mode, hidpi_factor) = hidpi_mode.apply(window.get_hidpi_factor());
            //     self.hidpi_mode = hidpi_mode;
            //     self.hidpi_factor = hidpi_factor;
            //     io.display_framebuffer_scale = [hidpi_factor as f32, hidpi_factor as f32];
            //     if let Some(logical_size) = window.get_inner_size() {
            //         let logical_size = self.scale_size_from_winit(window, logical_size);
            //         io.display_size = [logical_size.width as f32, logical_size.height as f32];
            //     }
            // }
        }

        Self {}
    }

    pub fn start_frame(&self) {}
}
