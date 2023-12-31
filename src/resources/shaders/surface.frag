#version 330 core

uniform mat4 view;

in vec3 view_normal;
in vec3 view_position;

out vec4 color;

const vec3 to_light_dir = normalize(vec3(-0.5, 1.0, 0.2));

void main()
{
    vec3 normal_toward_eye = normalize(faceforward(view_normal, view_position, view_normal));
    float dotp = max(0.0, dot(normal_toward_eye, normalize(mat3(view) * to_light_dir)));

    color = vec4(0.0, 0.0, 1.0 * dotp, 1.0);
}
