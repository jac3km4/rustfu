use crate::render::{Render, SpriteTransform};
use crate::types::{Animation, Color, Shape, Sprite};
use euclid::Transform2D;
use std::collections::HashMap;

use glow::HasContext;
use std::rc::Rc;

const BASE_SCALE: f32 = 4.;

pub struct Program<C: HasContext> {
    context: Rc<C>,
    program: C::Program,
}

impl<C: HasContext> Program<C> {
    pub fn new(gl: Rc<C>, shaders: &[ShaderSource]) -> Result<Program<C>, String> {
        unsafe {
            let program = gl.create_program()?;
            for source in shaders {
                let shader = Shader::load(gl.clone(), source)?;
                gl.attach_shader(program, shader.shader);
            }
            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                Err(gl.get_program_info_log(program))?
            }
            gl.use_program(Some(program));

            Ok(Program { context: gl, program })
        }
    }

    pub fn default(gl: Rc<C>) -> Result<Program<C>, String> {
        #[cfg(target_arch = "wasm32")]
        let version = "300 es";
        #[cfg(not(target_arch = "wasm32"))]
        let version = "330";

        let vertex_shader =
            ShaderSource::with_version(version, include_str!("../shaders/shader.vert"), ShaderType::Vertex);
        let fragment_shader =
            ShaderSource::with_version(version, include_str!("../shaders/shader.frag"), ShaderType::Fragment);

        unsafe {
            gl.enable(glow::BLEND);
            gl.blend_func(glow::ONE, glow::ONE_MINUS_SRC_ALPHA);
        }
        Program::new(gl.clone(), &[vertex_shader, fragment_shader])
    }
}

impl<C: HasContext> Drop for Program<C> {
    fn drop(&mut self) {
        unsafe { self.context.delete_program(self.program) }
    }
}

pub struct ShaderSource(String, ShaderType);

impl ShaderSource {
    pub fn with_version(version: &str, source: &str, shader_type: ShaderType) -> ShaderSource {
        ShaderSource(format!("#version {} {}", version, source), shader_type)
    }
}

pub enum ShaderType {
    Vertex,
    Fragment,
}

struct Shader<C: HasContext> {
    context: Rc<C>,
    shader: C::Shader,
}

impl<C: HasContext> Shader<C> {
    fn load(gl: Rc<C>, source: &ShaderSource) -> Result<Shader<C>, String> {
        unsafe {
            let type_enum = match source.1 {
                ShaderType::Fragment => glow::FRAGMENT_SHADER,
                ShaderType::Vertex => glow::VERTEX_SHADER,
            };
            let shader = gl.create_shader(type_enum)?;
            gl.shader_source(shader, &source.0);
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                Err(gl.get_shader_info_log(shader))
            } else {
                Ok(Shader { context: gl, shader })
            }
        }
    }
}

impl<C: HasContext> Drop for Shader<C> {
    fn drop(&mut self) {
        unsafe { self.context.delete_shader(self.shader) }
    }
}

pub struct Texture<C: HasContext> {
    context: Rc<C>,
    texture: C::Texture,
}

impl<C: HasContext> Texture<C> {
    pub fn new(gl: Rc<C>, image: image::RgbaImage) -> Result<Texture<C>, String> {
        unsafe {
            let texture = gl.create_texture()?;
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                image.width() as i32,
                image.height() as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                Some(&image.into_raw()),
            );
            Ok(Texture { context: gl, texture })
        }
    }

    pub fn bind(&self) {
        unsafe { self.context.bind_texture(glow::TEXTURE_2D, Some(self.texture)) }
    }
}

impl<C: HasContext> Drop for Texture<C> {
    fn drop(&mut self) {
        unsafe { self.context.delete_texture(self.texture) }
    }
}

pub struct DefaultLocations<C: HasContext> {
    position: u32,
    tex_coords: u32,
    matrix: C::UniformLocation,
    color: C::UniformLocation,
}

impl<C: HasContext> DefaultLocations<C> {
    pub fn from(program: &Program<C>) -> Option<DefaultLocations<C>> {
        let locations = unsafe {
            DefaultLocations {
                position: program.context.get_attrib_location(program.program, "position")?,
                tex_coords: program.context.get_attrib_location(program.program, "tex_coords")?,
                matrix: program.context.get_uniform_location(program.program, "matrix")?,
                color: program.context.get_uniform_location(program.program, "colors")?,
            }
        };
        Some(locations)
    }
}

pub struct SpriteVertex<C: HasContext> {
    context: Rc<C>,
    position: C::Buffer,
    tex_coords: C::Buffer,
    ebo: C::Buffer,
    vao: C::VertexArray,
}

impl<C: HasContext> SpriteVertex<C> {
    fn new(gl: Rc<C>, locations: &DefaultLocations<C>, shape: &Shape) -> Result<SpriteVertex<C>, String> {
        let right = shape.offset_x + shape.width as f32;
        let left = shape.offset_x;
        let top = shape.offset_y + shape.height as f32;
        let bottom = shape.offset_y;
        let positions = [left, top, right, top, right, bottom, left, bottom];
        let tex_coords = [
            shape.left,
            shape.top,
            shape.right,
            shape.top,
            shape.right,
            shape.bottom,
            shape.left,
            shape.bottom,
        ];
        unsafe {
            let vao = gl.create_vertex_array()?;
            gl.bind_vertex_array(Some(vao));

            let position_buf = gl.create_buffer()?;
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(position_buf));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, raw_byte_slice(&positions), glow::STATIC_DRAW);
            gl.vertex_attrib_pointer_f32(locations.position, 2, glow::FLOAT, false, 0, 0);
            gl.enable_vertex_attrib_array(locations.position);

            let tex_coord_buf = gl.create_buffer()?;
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(tex_coord_buf));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, raw_byte_slice(&tex_coords), glow::STATIC_DRAW);
            gl.vertex_attrib_pointer_f32(locations.tex_coords, 2, glow::FLOAT, false, 0, 0);
            gl.enable_vertex_attrib_array(locations.tex_coords);

            let element_buf = gl.create_buffer()?;
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(element_buf));
            gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, &[0u8, 1, 2, 2, 3, 0], glow::STATIC_DRAW);

            Ok(SpriteVertex {
                context: gl,
                position: position_buf,
                tex_coords: tex_coord_buf,
                ebo: element_buf,
                vao,
            })
        }
    }
}

impl<C: HasContext> Drop for SpriteVertex<C> {
    fn drop(&mut self) {
        unsafe {
            self.context.delete_vertex_array(self.vao);
            self.context.delete_buffer(self.position);
            self.context.delete_buffer(self.tex_coords);
            self.context.delete_buffer(self.ebo);
        }
    }
}

pub struct RenderState<'a, C: HasContext> {
    context: Rc<C>,
    vertexes: &'a mut HashMap<i16, SpriteVertex<C>>,
    texture: &'a Texture<C>,
    locations: &'a DefaultLocations<C>,
    viewport: (u32, u32),
}

impl<'a, C: HasContext> RenderState<'a, C> {
    pub fn new(
        context: Rc<C>,
        vertexes: &'a mut HashMap<i16, SpriteVertex<C>>,
        texture: &'a Texture<C>,
        locations: &'a DefaultLocations<C>,
        viewport: (u32, u32),
    ) -> Self {
        RenderState {
            context,
            vertexes,
            texture,
            locations,
            viewport,
        }
    }

    pub fn render(&mut self, animation: &Animation, sprite: &Sprite, frame: u32) {
        let scale = animation.index.clone().and_then(|i| i.scale).unwrap_or(1.);
        self.texture.bind();
        self.render_sprite(animation, sprite, SpriteTransform::scale(scale, scale), frame)
    }
}

impl<'a, C: HasContext> Render for RenderState<'a, C> {
    fn render(&mut self, shape: &Shape, transformation: SpriteTransform) -> () {
        let gl = self.context.clone();
        let locations = self.locations;
        let vert = self
            .vertexes
            .entry(shape.id)
            .or_insert_with(|| SpriteVertex::new(gl.clone(), locations, shape).expect("Could not load vertex"));

        let matrix = transformation
            .position
            .post_transform(&viewport_transform(self.viewport))
            .to_row_arrays();

        let matrix_data: [f32; 9] = [
            matrix[0][0],
            matrix[0][1],
            0.,
            matrix[1][0],
            matrix[1][1],
            0.,
            matrix[2][0],
            matrix[2][1],
            1.,
        ];

        let Color {
            red,
            green,
            blue,
            alpha,
        } = transformation.color.color();

        unsafe {
            gl.uniform_matrix_3_f32_slice(Some(&locations.matrix), false, &matrix_data);
            gl.uniform_4_f32(Some(&locations.color), red, green, blue, alpha);
            gl.bind_vertex_array(Some(vert.vao));
            gl.draw_elements(glow::TRIANGLES, 6, glow::UNSIGNED_BYTE, 0)
        }
    }
}

fn viewport_transform(viewport: (u32, u32)) -> Transform2D<f32, (), ()> {
    Transform2D::create_scale(BASE_SCALE / viewport.0 as f32, -BASE_SCALE / viewport.1 as f32)
}

unsafe fn raw_byte_slice<A>(buf: &[A]) -> &[u8] {
    let size = buf.len() * std::mem::size_of::<A>();
    std::slice::from_raw_parts_mut(buf.as_ptr() as *mut u8, size)
}
