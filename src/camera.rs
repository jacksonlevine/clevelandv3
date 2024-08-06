
use bevy::prelude::*;




/// Converts a direction vector to Euler angles (yaw, pitch, roll).
pub fn direction_to_euler(direction: Vec3) -> Vec3 {
    // Normalize the input direction
    let direction = direction.normalize();

    // Calculate yaw (rotation around Y axis)
    let yaw = direction.x.atan2(direction.z);

    // Calculate pitch (rotation around X axis)
    let pitch = (-direction.y).asin();

    // Roll is typically zero when converting a direction to Euler angles (depends on the use case)
    let roll = 0.0;

    Vec3::new(pitch, yaw, roll)
}

/// Converts a Vec3 containing Euler angles (in radians) to a normalized direction vector.
pub fn euler_to_direction(euler_angles: Vec3) -> Vec3 {
    // Create a quaternion from the Euler angles
    let quat = Quat::from_euler(
        EulerRot::XYZ,  // Rotation order: first pitch (X), then yaw (Y), then roll (Z)
        euler_angles.x, // Yaw around the Y-axis
        euler_angles.y, // Pitch around the X-axis
        euler_angles.z  // Roll around the Z-axis
    );

    // Assume forward direction is along positive z-axis
    let forward = Vec3::new(0.0, 0.0, 1.0);

    // Rotate the forward vector by the quaternion
    let direction = quat * forward;

    // Normalize the direction vector to handle any numerical inaccuracies
    direction.normalize()
}



#[derive(Resource, Default)]
pub struct JCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,

    pub direction: Vec3,
    pub position: Vec3,
    pub right: Vec3,
    pub up: Vec3,

    pub model: Mat4,
    pub projection: Mat4,
    pub view: Mat4,
    pub mvp: Mat4,

    pub velocity: Vec3,

    pub far: f32,
    pub near: f32,

    pub xzdir: Vec3,
}

impl JCamera {
    pub fn new() -> JCamera {
        let direction = Vec3::new(0.0, 0.0, 1.0);
        let position = Vec3::new(0.0, 0.0, 0.0);
        let right = Vec3::new(0.0, 1.0, 0.0).cross(direction).normalize();
        let fov: f32 = 80.0;
        let far = 560.0;
        let near = 0.01;
        let up = direction.cross(right);

        let model = Mat4::IDENTITY;
        let projection = Mat4::perspective_rh_gl(fov.to_radians(), 1280.0 / 720.0, near, far);
        let view = Mat4::look_at_rh(position, position + direction, up);
        JCamera {
            yaw: 0.0,
            pitch: 0.0,
            fov,
            direction,
            position,
            right,
            up: direction.cross(right),
            model,
            projection,
            view,
            mvp: projection * model * view,
            velocity: Vec3::new(0.0, 0.0, 0.0),
            far,
            near,
            xzdir: Vec3::ZERO
        }
    }
    pub fn update_fov(&mut self, value: f32) {
        self.fov = value.clamp(50.0, 160.0);
        self.projection =
            Mat4::perspective_rh_gl(self.fov.to_radians(), 1280.0 / 720.0, self.near, self.far);
        self.recalculate();
    }
    pub fn recalculate(&mut self) {

        self.direction.x = self.yaw.to_radians().cos() as f32 * self.pitch.to_radians().cos() as f32;
        self.direction.y = self.pitch.to_radians().sin();
        self.direction.z = self.yaw.to_radians().sin() * self.pitch.to_radians().cos();
        self.direction = self.direction.normalize();

        self.right = Vec3::new(0.0, 1.0, 0.0).cross(self.direction).normalize();
        self.up = self.direction.cross(self.right);
        self.view = Mat4::look_at_rh(self.position, self.position + self.direction, self.up);
        self.mvp = self.projection * self.view * self.model;

        let noydir = Vec3::new(self.direction.x, 0.0, self.direction.z);
        if noydir.length() > 0.0 {
            self.xzdir = noydir.normalize();
        }
    }
  
}
