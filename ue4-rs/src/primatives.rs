use memory::memory_macros::{c_class, c_enum};

pub type FReal = f32;

pub type FVector = vek::vec::vec3::Vec3<FReal>;
pub type FMatrix = vek::mat4::Mat4<FReal>;
pub type FMatrix3x3 = vek::mat3::Mat3<FReal>;
pub type FVector2D = vek::vec::vec2::Vec2<FReal>;
pub type FQuat = vek::quaternion::Quaternion<FReal>;

#[c_class]
pub struct FColor {
    pub b: u8,
    pub g: u8,
    pub r: u8,
    pub a: u8,
}

#[c_class]
pub struct FLinearColor {
    pub r : f32, 
	pub g : f32, 
	pub b : f32,
	pub a : f32,
}

/// https://github.com/EpicGames/UnrealEngine/blob/4.27/Engine/Source/Runtime/Core/Private/Math/Transform.cpp
/// https://github.com/EpicGames/UnrealEngine/blob/4.27/Engine/Source/Runtime/Core/Public/Math/TransformNonVectorized.h
#[c_class]
pub struct FTransform {
    pub rotation: FQuat,
    pub translation: FVector,
    // this is due to aligment of FQuat
    unknown_data00: FReal,
    pub scale_3d: FVector,
    unknown_data01: FReal,
}

#[c_class]
pub struct FRotator {
    pub pitch: FReal,
    pub yaw: FReal,
    pub roll: FReal,
}

#[c_class]
pub struct FBoxSphereBounds {
    pub origin: FVector,
    pub orig_box_extentin: FVector,
    pub sphere_radius: f32,
}

#[c_class]
pub struct FMinimalViewInfo {
    pub location: FVector,
    pub rotation: FRotator,
    pub fov: f32,
    pub desired_fov: f32,
    //pub base_fov: f32,
    pub ortho_width: f32,
    pub ortho_near_clip_plane: f32,
    pub ortho_far_clip_plane: f32,
    pub aspect_ratio: f32,
    pub flags: u8, // bConstrainAspectRatio, bUseFieldOfViewForLOD
    pub projection_mode: ECameraProjectionMode,
}

#[c_enum]
pub enum ECameraProjectionMode {
    Perspective = 0,
    Orthographic = 1,
    EcameraProjectionModeMax = 2,
}

#[c_class]
pub struct FCameraCacheEntry {
    pub timestamp: f32,
    pad: [u8; 12],
    pub pov: FMinimalViewInfo,
}