use std::{marker::PhantomData, ops::Range, os::windows::prelude::OsStringExt};

use memoize::memoize;
use memory::memory_macros::{c_class, xh};
use memory::types::Ptr;
use memory::{
    memory::{read, read_array},
    types::{IsValid, MemoryError, ReadResult, UPtr},
};

use crate::primatives::{
    FMatrix3x3, FMinimalViewInfo, FReal, FRotator, FTransform, FVector, FVector2D,
};

#[c_class]
pub struct TArray<T> {
    pub ptr: UPtr,
    pub size: u32,
    pub max: u32,
    pub phantom: std::marker::PhantomData<T>,
}

impl<T> IntoIterator for TArray<T> {
    type Item = T;

    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.to_vec().into_iter()
    }
}

impl<T> TArray<T> {
    pub fn cast<V>(&self) -> TArray<V> {
        assert_eq!(
            std::mem::size_of::<V>(),
            std::mem::size_of::<T>(),
            "Unable to cast between different sizes"
        );
        TArray::<V> {
            ptr: self.ptr,
            size: self.size,
            max: self.max,
            phantom: PhantomData::default(),
        }
    }
}

impl<T> std::default::Default for TArray<T> {
    fn default() -> Self {
        Self {
            ptr: 0,
            size: 0,
            max: 0,
            phantom: PhantomData::default(),
        }
    }
}

impl<T> TArray<T> {
    pub fn get_address(&self, index: usize) -> UPtr {
        self.ptr + (std::mem::size_of::<T>() * index) as UPtr
    }

    pub fn get_base_address(&self) -> UPtr {
        self.ptr
    }

    pub fn num(&self) -> usize {
        self.size as usize
    }
    pub fn len(&self) -> usize {
        self.size as usize
    }
    pub fn capacity(&self) -> usize {
        self.max as usize
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /** gets all items in the list, will return vec![] if is invalid */
    pub fn to_vec(&self) -> Vec<T> {
        // just some sanity check to not bsod
        if self.is_invalid() {
            return vec![];
        }

        if let Ok(out) = read_array::<T>(self.ptr, self.size as usize) {
            out
        } else {
            vec![]
        }
    }

    pub fn to_slice(&self, range: Range<usize>) -> Vec<T> {
        // just some sanity check to not bsod
        if self.is_invalid() {
            return vec![];
        }

        if let Ok(out) = read_array::<T>(self.get_address(range.start), range.len()) {
            out
        } else {
            vec![]
        }
    }

    pub fn index(&self, index: usize) -> ReadResult<T> {
        if index >= self.size as usize {
            Err(MemoryError::InvalidArg)?
        }
        read(self.get_address(index))
    }
}

impl<T> IsValid for TArray<T> {
    fn is_valid(&self) -> bool {
        // we limit at 20k because there is no reasonable array in ue with more items
        self.ptr.is_valid() && self.size <= self.max && self.size < 20_000
    }
}

#[c_class]
pub struct FString {
    data: TArray<u16>,
}

impl FString {
    pub fn to_string(&self) -> ReadResult<String> {
        let string_data = read_array::<u16>(self.data.ptr, self.data.size as usize)?;
        if let Some(result) = std::ffi::OsString::from_wide(&string_data).to_str() {
            Ok(result.to_string())
        } else {
            Err(MemoryError::BadData)
        }
    }
}

impl IsValid for FString {
    fn is_valid(&self) -> bool {
        self.data.is_valid()
    }
}

/// https://github.com/EpicGames/UnrealEngine/blob/4.26/Engine/Source/Runtime/Engine/Classes/Components/SkinnedMeshComponent.h#L253 VertexOffsetUsage + 0x10
#[c_class]
pub struct BoneArray {
    component_space_transforms_array: [TArray<FTransform>; 2],
}

impl BoneArray {
    pub fn bones(&self) -> Vec<FTransform> {
        let ret = self.component_space_transforms_array[0].to_vec();
        if !ret.is_empty() {
            return ret;
        }
        return self.component_space_transforms_array[1].to_vec();
    }

    pub fn index(&self, idx: usize) -> ReadResult<FTransform> {
        if self.component_space_transforms_array[0].is_valid() {
            if let Ok(value) = self.component_space_transforms_array[0].index(idx) {
                return Ok(value);
            }
        }

        if self.component_space_transforms_array[1].is_valid() {
            if let Ok(value) = self.component_space_transforms_array[1].index(idx) {
                return Ok(value);
            }
        }
        Err(MemoryError::BadData)
    }
}

impl FTransform {
    pub fn get_bone_with_rotation(&self, bone: &FTransform) -> FVector {
        self.rotation * (self.scale_3d * bone.translation) + self.translation
    }
}

impl FRotator {
    pub fn new(pitch: FReal, yaw: FReal, roll: FReal) -> Self {
        Self { pitch, yaw, roll }
    }

    pub fn to_matrix_camera(&self) -> FMatrix3x3 {
        let rad_pitch = self.pitch.to_radians();
        let rad_yaw = self.yaw.to_radians();
        let rad_roll = self.roll.to_radians();

        let sp = rad_pitch.sin();
        let cp = rad_pitch.cos();
        let sy = rad_yaw.sin();
        let cy = rad_yaw.cos();
        let sr = rad_roll.sin();
        let cr = rad_roll.cos();

        let mut matrix = FMatrix3x3::zero();

        matrix.cols[2][0] = cp * cy;
        matrix.cols[2][1] = cp * sy;
        matrix.cols[2][2] = sp;

        matrix.cols[0][0] = sr * sp * cy - cr * sy;
        matrix.cols[0][1] = sr * sp * sy + cr * cy;
        matrix.cols[0][2] = -sr * cp;

        matrix.cols[1][0] = -(cr * sp * cy + sr * sy);
        matrix.cols[1][1] = cy * sr - cr * sp * sy;
        matrix.cols[1][2] = cr * cp;

        matrix
    }

    pub fn rot_to_direction(pitch_rad: FReal, yaw_rad: FReal) -> FVector {
        FVector::new(
            yaw_rad.sin() * pitch_rad.cos(),
            yaw_rad.cos() * pitch_rad.cos(),
            pitch_rad.sin(),
        )
    }

    pub fn to_direction(&self) -> FVector {
        Self::rot_to_direction(
            self.pitch.to_radians(),
            std::f32::consts::FRAC_PI_2 - self.yaw.to_radians(),
        )
    }
}

#[derive(Debug)]
pub struct Camera {
    pub location: FVector,
    pub matrix: FMatrix3x3,
    pub inverse_fov: FReal,
    pub screen_size: FVector2D,
    pub screen_center: FVector2D,
}

impl Camera {
    pub fn new(minimal_info: FMinimalViewInfo, screen_size: FVector2D) -> Self {
        Self {
            location: minimal_info.location,
            matrix: minimal_info.rotation.to_matrix_camera(),
            inverse_fov: (minimal_info.fov * 0.00872665).tan() as FReal,
            screen_size,
            screen_center: screen_size * 0.5,
        }
    }

    /// world to screen (checked) with reasonable out of sceen check to not draw too much
    pub fn w2sc(&self, world_location: FVector) -> Option<FVector2D> {
        let result = self.w2s(world_location);
        const EXTRA: f32 = 200.0;

        if result.x < -EXTRA
            || result.y < -EXTRA
            || result.x > self.screen_size.x + EXTRA
            || result.y > self.screen_size.y + EXTRA
        {
            return None;
        }

        Some(result)
    }

    pub fn w2s(&self, world_location: FVector) -> FVector2D {
        let v_delta = world_location - self.location;
        let mut v_transformed = v_delta * self.matrix;

        if v_transformed.z < 1.0 {
            v_transformed.z = 1.0;
        }
        let multi = self.screen_center.x / (self.inverse_fov * v_transformed.z);

        FVector2D::new(
            self.screen_center.x + v_transformed.x * multi,
            self.screen_center.y - v_transformed.y * multi,
        )
    }
}

type FNameEntryId = i32;
#[c_class]
pub struct FName {
    /** Index into the Names array (used to find String portion of the string/number pair used for comparison) */
    comparison_index: FNameEntryId,

    /** Number portion of the string/number pair (stored internally as 1 more than actual, so zero'd memory will be the default, no-instance case) */
    //if !UE_FNAME_OUTLINE_NUMBER
    number: u32,
    // /** Index into the Names array (used to find String portion of the string/number pair used for display) */
    //#if WITH_CASE_PRESERVING_NAME
    display_index: FNameEntryId,
}

#[c_class]
pub struct UObject {
    vtable: UPtr,

    /** Flags used to track and report various object states. This needs to be 8 byte aligned on 32-bit
    platforms to reduce memory waste */
    object_flags: u32, //EObjectFlags,

    /** Index into GObjectArray...very private. */
    pub internal_index: i32,

    /** Class the object belongs to. */
    pub class_private: Ptr<UStruct>, //Ptr<UClass>,

    /** Name of this object */
    name_private: FName,

    /** Object this object resides in. */
    pub outer_private: Ptr<UObject>,
}

#[c_class]
pub struct UField {
    pub base: UObject,
    /** Next Field in the linked list */
    pub next: Ptr<UField>,
}

#[c_class]
pub struct FStructBaseChain {
    struct_base_chain_array: Ptr<Ptr<FStructBaseChain>>,
    num_struct_bases_in_chain_minus_one: i32,
}

#[c_class]
pub struct UStruct {
    pub base: UField,
    pub chain: FStructBaseChain,
    pub super_struct: Ptr<UStruct>,
}

fn is_a_class_string(mut super_class: Ptr<UStruct>, class_hash: u64) -> bool {
    while super_class.is_valid() {
        let next_class = super_class.read().expect("Good superclass");
        if class_hash == next_class.base.base.hash() {
            return true;
        }
        super_class = next_class.super_struct;
    }

    false
}

impl UObject {
    pub fn is_a_hash(&self, name: u64) -> bool {
        is_a_class_string(self.class_private, name)
    }

    pub fn hash(&self) -> u64 {
        self.name_private.to_string_hash().unwrap_or(0)
    }

    pub fn name(&self) -> Option<String> {
        self.name_private.to_string()
    }

    pub fn read_name(&self) -> ReadResult<String> {
        match self.name() {
            Some(s) => Ok(s),
            None => Err(MemoryError::BadData),
        }
    }
}

impl FName {
    pub fn to_string_hash(&self) -> Option<u64> {
        get_fname_hash(self.comparison_index)
    }
    pub fn to_string(&self) -> Option<String> {
        get_fname(self.comparison_index)
    }
}

#[memoize]
pub fn hash(data: String) -> u64 {
    xh!(data)
}

#[memoize]
pub fn get_fname_hash(index: i32) -> Option<u64> {
    get_fname(index).map(|x| hash(x))
}

pub static mut FNAME_POOL_PTR: UPtr = 0;
pub static mut VALORANT_KEY: u32 = 0;

pub const FNAME_POOL_STRIDE: u32 = 4;

extern "C" {
    #[allow(unused)]
    fn decrypt_fname(buf: *mut u8, len: u16, wide: u8, key: u32);
}

#[memoize]
pub fn get_fname(index: i32) -> Option<String> {
    unsafe {
        let block = index as u32 >> 16;
        let offset = index as u32 & ((1 << 16) - 1);

        let data_ptr: UPtr = (read::<Ptr<u8>>(FNAME_POOL_PTR + 0x10 + 8 * block as UPtr).ok()?
            + (offset * FNAME_POOL_STRIDE) as usize)
            .ptr;

        if data_ptr.is_invalid() {
            return None;
        }

        let header = read::<u16>(data_ptr + 4).ok()?;
        let length = header >> 1;
        let wide: u8 = header as u8 & 1;

        let mut data =
            read_array::<u8>(data_ptr + 6, length as usize * (wide as usize + 1)).ok()?;

        decrypt_fname(data.as_mut_ptr(), length, wide, VALORANT_KEY);

        Some(std::str::from_utf8(&data).ok()?.to_owned())
    }
}

#[c_class]
pub struct FText {
    pub data: Ptr<FTextData>,
    pad: [u8; 0x10],
}

// not real name, but real implemenation of ftext is complicated af for no reason
#[c_class]
pub struct FTextData {
    pad: [u8; 0x28],
    string: FString,
}

impl FText {
    pub fn to_string(&self) -> ReadResult<String> {
        self.to_fstring()?.to_string()
    }

    pub fn to_fstring(&self) -> ReadResult<FString> {
        let fstr = read::<FString>(self.data.ptr + 0x28)?;
        Ok(fstr)
    }
}
