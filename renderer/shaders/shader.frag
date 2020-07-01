
precision mediump float;

in vec2 v_tex_coords;
out vec4 output;

uniform sampler2D tex;
uniform vec4 colors;

void main() {
    output = texture(tex, v_tex_coords) * colors;
}
