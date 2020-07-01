
in vec2 position;
in vec2 tex_coords;
out vec2 v_tex_coords;

uniform mat3 matrix;

void main() {
    v_tex_coords = tex_coords;
    gl_Position = vec4(matrix * vec3(position, 1), 1);
}
