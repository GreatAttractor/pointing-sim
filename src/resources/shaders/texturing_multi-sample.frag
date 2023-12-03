#version 330 core

in vec2 tex_coord;
out vec4 output_color;

uniform sampler2DMS source_texture;

void main()
{
    vec4 color = vec4(0.0);

    ivec2 texel = ivec2(tex_coord * textureSize(source_texture)); //TODO: provide texture size as a uniform for better speed?

    //TODO: provide additional input with sample mask, sum only edge samples?
    for (int i = 0; i < 8; ++i) //TODO: provide sample count as uniform
    {
        color += texelFetch(source_texture, texel, i);
    }
    color /= 8.0;

    output_color = color;
}
