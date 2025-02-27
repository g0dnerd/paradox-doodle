#version 330 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNorm;

mat4x4 perspective = mat4x4(
  vec4(1.5, 0, 0, 0),
  vec4(0, 1.5, 0, 0),
  vec4(0, 0, 1, 1),
  vec4(0, 0, -2 * 0.01, 0)
);

uniform mat4x4 world_transform;
out vec3 norm;

void main() {
  gl_Position = perspective * world_transform * vec4(aPos.x, aPos.y, aPos.z, 1.0);
  norm = mat3(world_transform) * aNorm;
}
