#version 330 core

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

in vec3 position;
in vec3 normal;

out vec3 view_normal;
out vec3 view_position;

void main()
{
    mat4 view_model = view * model;
    vec4 view_model_position = view_model * vec4(position, 1.0);
    vec4 projected = projection * view_model_position;

    view_position = view_model_position.xyz;

    mat3 normal_matrix = mat3(view) * transpose(inverse(mat3(model)));
    view_normal = normalize(normal_matrix * normal);

    // negating Y, because we render to a texture before displaying,
    // and texture rows are stored top-to-bottom
    gl_Position = vec4(projected.x, -projected.y, projected.z, projected.w);
}
