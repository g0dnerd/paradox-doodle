#version 330 core

layout (location = 0) in vec3 aPos;

mat4x4 perspective = mat4x4(
  vec4(1.5, 0, 0, 0),
  vec4(0, 1.5, 0, 0),
  vec4(0, 0, 1, 1),
  vec4(0, 0, -2 * 0.01, 0)
);

uniform mat4x4 camera;

void main() {
  gl_Position = perspective * camera * vec4(aPos.x, aPos.y, aPos.z, 1.0);
}
