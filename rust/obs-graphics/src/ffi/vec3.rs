//! `struct vec3` from `libobs/graphics/vec3.h`. `vec3.c` is not ported yet;
//! the type is here for the vec4 shims.

/// `struct vec3`. Like `struct vec4` it is a union with `__m128`, so it has
/// four floats (`w` is padding the C code keeps at 0), size 16, align 16.
#[repr(C, align(16))]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
