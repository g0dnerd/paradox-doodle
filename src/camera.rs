use std::f32::consts;

pub struct Camera {
    pub screen_size: (u32, u32),
    pub angle_y: f32,
    pub angle_xz: f32,
    pub dist: f32,
}

const MODEL_CENTER_Y: f32 = 2.0;

impl Camera {
    pub fn to_uniform_data(&self) -> [f32; 16 * 3 + 4] {
        // aspect ratio
        let aspect = self.screen_size.0 as f32 / self.screen_size.1 as f32;
        let proj = glam::Mat4::perspective_rh(consts::FRAC_PI_4, aspect, 1.0, 400.0);
        let cam_pos = glam::Vec3::new(
            self.angle_xz.cos() * self.angle_y.sin() * self.dist,
            self.angle_xz.sin() * self.dist + MODEL_CENTER_Y,
            self.angle_xz.cos() * self.angle_y.cos() * self.dist,
        );
        let view = glam::Mat4::look_at_rh(
            cam_pos,
            glam::Vec3::new(0f32, MODEL_CENTER_Y, 0.0),
            glam::Vec3::Y,
        );
        let proj_inv = proj.inverse();

        let mut raw = [0f32; 16 * 3 + 4];
        raw[..16].copy_from_slice(&AsRef::<[f32; 16]>::as_ref(&proj)[..]);
        raw[16..32].copy_from_slice(&AsRef::<[f32; 16]>::as_ref(&proj_inv)[..]);
        raw[32..48].copy_from_slice(&AsRef::<[f32; 16]>::as_ref(&view)[..]);
        raw[48..51].copy_from_slice(&AsRef::<[f32; 3]>::as_ref(&cam_pos)[..]);
        raw[51] = 1.0;
        raw
    }
}
