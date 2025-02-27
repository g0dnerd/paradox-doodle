#version 330 core

in vec3 norm;
out vec4 fragColor;

uniform vec3 color;

uniform vec3 sunDir = vec3(-0.67, -0.67, 0.3);
uniform float sunStrength = 0.9;
uniform float ambientStrength = 0.1;

void main() {
  float faceSunAmount = max(-dot(norm, sunDir) * sunStrength, 0.0);

  fragColor = vec4(color * (ambientStrength + faceSunAmount), 1.0);
}
