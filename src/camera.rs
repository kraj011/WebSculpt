use winit::{event::{ElementState, KeyEvent, WindowEvent}, keyboard::{KeyCode, PhysicalKey}};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);
 
pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32
}

impl Camera {
    pub fn build_vp_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}


#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_vp_matrix().into();
    }
}


pub struct CameraController {
    pub speed: f32,
    pub left: bool,
    pub right: bool,
    pub forward: bool,
    pub backward: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            left: false,
            right: false,
            forward: false,
            backward: false
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    state,
                    physical_key: PhysicalKey::Code(keycode),
                    ..
                },
                ..
             } => {
                let is_pressed = *state == ElementState::Pressed;

                match keycode {
                    KeyCode::ArrowLeft | KeyCode::KeyA => self.left = is_pressed,
                    KeyCode::ArrowRight | KeyCode::KeyD => self.right = is_pressed,
                    KeyCode::ArrowDown | KeyCode::KeyS => self.backward = is_pressed,
                    KeyCode::ArrowUp | KeyCode::KeyW => self.forward = is_pressed,
                    _ => {}
                }

                true
             },
             _ => false
        }
    }
    
    pub fn update_camera(&self, camera: &mut Camera) {
        use cgmath::InnerSpace;
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();

        if self.forward {
            camera.eye += forward_norm * self.speed;
        }
        if self.backward {
            camera.eye -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(camera.up);

        let forward = camera.target - camera.eye;
        let forward_mag = forward.magnitude();

        if self.right {
            camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
        }
        if self.left {
            camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }


    }
}