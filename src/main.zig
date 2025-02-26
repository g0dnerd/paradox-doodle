const glfw = @cImport({
    @cInclude("GLFW/glfw3.h");
});
const gl = @cImport({
    @cDefine("GL_GLEXT_PROTOTYPES", "1");
    @cInclude("GL/gl.h");
});
const std = @import("std");
const panic = std.debug.panic;

pub fn compileShader(source: [:0]const u8, shader_type: gl.GLuint) !gl.GLuint {
    const len_int: c_int = @intCast(source.len);
    const shader = gl.glCreateShader(shader_type);
    gl.glShaderSource(shader, 1, &source.ptr, &len_int);
    gl.glCompileShader(shader);

    var success: gl.GLint = 0;
    var buf: [4096]u8 = undefined;
    gl.glGetShaderiv(shader, gl.GL_COMPILE_STATUS, &success);
    if (success == 0) {
        var len: gl.GLsizei = 0;
        gl.glGetShaderInfoLog(shader, buf.len, &len, &buf);
        std.debug.print("{s}\n", .{buf[0..@intCast(len)]});
        return error.ShaderCompilation;
    }

    return shader;
}

pub fn makeProgram() !gl.GLuint {
    const vs = try compileShader(@embedFile("vs.glsl"), gl.GL_VERTEX_SHADER);
    defer gl.glDeleteShader(vs);
    const fs = try compileShader(@embedFile("fs.glsl"), gl.GL_FRAGMENT_SHADER);
    defer gl.glDeleteShader(fs);

    const program = gl.glCreateProgram();
    gl.glAttachShader(program, vs);
    gl.glAttachShader(program, fs);
    gl.glLinkProgram(program);

    var success: gl.GLint = 0;
    gl.glGetProgramiv(program, gl.GL_LINK_STATUS, &success);
    var buf: [4096]u8 = undefined;
    if (success == 0) {
        var len: gl.GLsizei = 0;
        gl.glGetProgramInfoLog(program, buf.len, &len, &buf);
        std.debug.print("{s}\n", .{buf[0..@intCast(len)]});
        return error.ProgramLink;
    }

    return program;
}

pub fn main() !void {
    _ = glfw.glfwSetErrorCallback(errorCallback);

    if (glfw.glfwInit() != glfw.GLFW_TRUE) {
        return error.GlfwInit;
    }

    glfw.glfwWindowHint(glfw.GLFW_CONTEXT_VERSION_MAJOR, 4);
    glfw.glfwWindowHint(glfw.GLFW_CONTEXT_VERSION_MINOR, 5);
    glfw.glfwWindowHint(glfw.GLFW_OPENGL_PROFILE, glfw.GLFW_OPENGL_CORE_PROFILE);

    const window = glfw.glfwCreateWindow(800, 600, "ziggorama", null, null);
    if (window == null) {
        return error.InitWindow;
    }

    glfw.glfwMakeContextCurrent(window);

    const program = try makeProgram();
    _ = program;

    while (glfw.glfwWindowShouldClose(window) == 0) {
        gl.glClearColor(0.2, 0.3, 0.3, 1.0);
        gl.glClear(gl.GL_COLOR_BUFFER_BIT);

        glfw.glfwSwapBuffers(window);
        glfw.glfwPollEvents();
    }

    glfw.glfwTerminate();
}

pub fn errorCallback(err: c_int, msg: [*c]const u8) callconv(.C) void {
    _ = err;
    panic("GLFW error: {s}\n", .{msg});
}
