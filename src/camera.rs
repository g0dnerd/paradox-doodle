use std::f32::consts;

pub struct Camera {
    pub screen_size: (u32, u32),
    pub dist: f32,
    pub angle: f32,
}

const MODEL_CENTER_Y: f32 = 2.0;

impl Camera {
    pub fn to_uniform_data(&self) -> [f32; 16 * 3 + 4] {
        let aspect = self.screen_size.0 as f32 / self.screen_size.1 as f32;
        let proj = glam::Mat4::perspective_rh(consts::FRAC_PI_4, aspect, 1.0, 400.0);

        let eye = glam::Vec3::new(
            self.dist * self.angle.cos(),
            self.dist * 0.5,
            self.dist * self.angle.sin(),
        );
        let center = glam::Vec3::new(0.0, MODEL_CENTER_Y, 0.0);
        let up = glam::Vec3::Y;

        let view = glam::Mat4::look_at_rh(eye, center, up);
        let proj_inv = proj.inverse();

        let mut raw = [0f32; 16 * 3 + 4];
        raw[..16].copy_from_slice(&AsRef::<[f32; 16]>::as_ref(&proj)[..]);
        raw[16..32].copy_from_slice(&AsRef::<[f32; 16]>::as_ref(&proj_inv)[..]);
        raw[32..48].copy_from_slice(&AsRef::<[f32; 16]>::as_ref(&view)[..]);
        raw[48..51].copy_from_slice(&eye.to_array()[..]);
        raw[51] = 1.0;
        raw
    }

    #[allow(clippy::nonminimal_bool)]
    pub fn is_in_view(&self, pos: [f32; 3]) -> bool {
        let aspect = self.screen_size.0 as f32 / self.screen_size.1 as f32;
        let proj = glam::Mat4::perspective_rh(consts::FRAC_PI_4, aspect, 1.0, 400.0);
        let eye = glam::Vec3::new(
            self.dist * self.angle.cos(),
            self.dist * 0.5,
            self.dist * self.angle.sin(),
        );
        let center = glam::Vec3::new(0.0, MODEL_CENTER_Y, 0.0);
        let up = glam::Vec3::Y;
        let view = glam::Mat4::look_at_rh(eye, center, up);

        let view_proj_mat = proj * view;
        let bh_homogenous = glam::Vec4::new(pos[0], pos[1], pos[2], 1.0);

        let ndc_pos = view_proj_mat * bh_homogenous;
        let ndc_coords = ndc_pos.truncate() / ndc_pos.w;

        ndc_coords.x >= -1.0
            && ndc_coords.x <= 1.0
            && ndc_coords.y >= -1.0
            && ndc_coords.y <= 1.0
            && ndc_coords.z >= -1.0
            && ndc_coords.z <= 1.0
    }
}
