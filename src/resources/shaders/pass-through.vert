#version 330 core

in vec2 position;
out vec2 tex_coord;

void main()
{
    // apply texture coords (0,0)-(1,1) to unit quad (-1,-1)-(1,1)
    tex_coord.xy = position.xy / 2 + vec2(0.5, 0.5);

    gl_Position.xy = position.xy;
    gl_Position.z = 0.0;
    gl_Position.w = 1.0;
}
