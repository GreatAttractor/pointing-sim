#version 330 core

in vec2 tex_coord;
out vec4 output_color;

uniform sampler2D source_texture;
uniform float brightness;

void main()
{
    vec4 color = texture(source_texture, tex_coord);
    color.rgb *= brightness;

    output_color = color;
}
