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

pub fn makeVao() !gl.GLuint {
    const vertices: []const f32 = &.{
        0.5,  0.5,  -0.5,
        0.5,  -0.5, -0.5,
        0.5,  0.5,  0.5,
        0.5,  -0.5, 0.5,
        -0.5, 0.5,  -0.5,
        -0.5, -0.5, -0.5,
        -0.5, 0.5,  0.5,
        -0.5, -0.5, 0.5,
    };

    const position_indices: []const u32 = &.{
        4, 2, 0,
        2, 7, 3,
        6, 5, 7,
        1, 7, 5,
        0, 3, 1,
        4, 1, 5,
        4, 6, 2,
        2, 6, 7,
        6, 4, 5,
        1, 3, 7,
        0, 2, 3,
        4, 0, 1,
    };

    const normal_indices: []const u32 = &.{
        0, 0, 0,
        1, 1, 1,
        2, 2, 2,
        3, 3, 3,
        4, 4, 4,
        5, 5, 5,
        0, 0, 0,
        1, 1, 1,
        2, 2, 2,
        3, 3, 3,
        4, 4, 4,
        5, 5, 5,
    };

    const normals: []const f32 = &.{
        0.0,  1.0,  0.0,
        0.0,  0.0,  1.0,
        -1.0, 0.0,  0.0,
        0.0,  -1.0, 0.0,
        1.0,  0.0,  0.0,
        0.0,  0.0,  -1.0,
    };

    var combined_buffer: [36 * 3 * 2]f32 = undefined;
    var combined_buffer_idx: usize = 0;

    for (position_indices, normal_indices) |vidx, nidx| {
        defer combined_buffer_idx += 6;
        const vert = vertices[vidx * 3 .. (vidx + 1) * 3];
        const norm = normals[nidx * 3 .. (nidx + 1) * 3];

        combined_buffer[combined_buffer_idx + 0] = vert[0];
        combined_buffer[combined_buffer_idx + 1] = vert[1];
        combined_buffer[combined_buffer_idx + 2] = vert[2];
        combined_buffer[combined_buffer_idx + 3] = norm[0];
        combined_buffer[combined_buffer_idx + 4] = norm[1];
        combined_buffer[combined_buffer_idx + 5] = norm[2];
    }

    var vao: gl.GLuint = 0;
    var vbo: gl.GLuint = 0;

    gl.glGenVertexArrays(1, &vao);
    gl.glGenBuffers(1, &vbo);

    gl.glBindVertexArray(vao);

    gl.glBindBuffer(gl.GL_ARRAY_BUFFER, vbo);
    gl.glBufferData(gl.GL_ARRAY_BUFFER, combined_buffer.len * @sizeOf(f32), &combined_buffer, gl.GL_STATIC_DRAW);

    gl.glVertexAttribPointer(0, 3, gl.GL_FLOAT, gl.GL_FALSE, 6 * @sizeOf(f32), null);
    gl.glEnableVertexAttribArray(0);

    gl.glVertexAttribPointer(1, 3, gl.GL_FLOAT, gl.GL_FALSE, 6 * @sizeOf(f32), @ptrFromInt(12));
    gl.glEnableVertexAttribArray(1);

    gl.glBindBuffer(gl.GL_ARRAY_BUFFER, 0);

    return vao;
}

fn matMul(a: [16]f32, b: [16]f32) [16]f32 {
    var ret: [16]f32 = .{0} ** 16;
    for (0..16) |idx| {
        const row = idx / 4;
        const col = idx % 4;

        for (0..4) |mat_idx| {
            ret[idx] += a[row * 4 + mat_idx] * b[mat_idx * 4 + col];
        }
    }
    return ret;
}

const identity: [16]f32 = .{
    1, 0, 0, 0,
    0, 1, 0, 0,
    0, 0, 1, 0,
    0, 0, 0, 1,
};

fn makeZRotation(amount: f32) [16]f32 {
    const sx = @sin(amount);
    const cx = @cos(amount);

    var matrix = identity;
    matrix[0] = cx;
    matrix[1] = -sx;
    matrix[4] = sx;
    matrix[5] = cx;

    return matrix;
}

fn makeYRotation(amount: f32) [16]f32 {
    const sx = @sin(amount);
    const cx = @cos(amount);

    var matrix = identity;
    matrix[0] = cx;
    matrix[2] = -sx;
    matrix[8] = sx;
    matrix[10] = cx;

    return matrix;
}

fn hueToRgb(hue: f32) [3]f32 {
    const segment = (@as(u32, @intFromFloat(hue)) % 360) / 60;
    const x = (1 - @abs(std.math.modf(hue / 60 / 2).fpart * 2 - 1));

    return switch (segment) {
        0 => .{ 1, x, 0 },
        1 => .{ x, 1, 0 },
        2 => .{ 0, 1, x },
        3 => .{ 0, x, 1 },
        4 => .{ x, 0, 1 },
        5 => .{ 1, 0, x },
        else => unreachable,
    };
}

pub fn main() !void {
    _ = glfw.glfwSetErrorCallback(errorCallback);

    if (glfw.glfwInit() != glfw.GLFW_TRUE) {
        return error.GlfwInit;
    }

    glfw.glfwWindowHint(glfw.GLFW_CONTEXT_VERSION_MAJOR, 4);
    glfw.glfwWindowHint(glfw.GLFW_CONTEXT_VERSION_MINOR, 5);
    glfw.glfwWindowHint(glfw.GLFW_OPENGL_PROFILE, glfw.GLFW_OPENGL_CORE_PROFILE);

    const window = glfw.glfwCreateWindow(800, 800, "ziggorama", null, null);
    if (window == null) {
        return error.InitWindow;
    }

    glfw.glfwMakeContextCurrent(window);

    const program = try makeProgram();
    const vao = try makeVao();

    gl.glEnable(gl.GL_DEPTH_TEST);
    gl.glDepthFunc(gl.GL_LESS);

    const color_uniform = gl.glGetUniformLocation(program, "color");
    const world_transform_uniform = gl.glGetUniformLocation(program, "world_transform");
    const translation: [16]f32 = .{
        1.0, 0.0, 0.0, 0.1,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 2.0,
        0.0, 0.0, 0.0, 1.0,
    };

    var last = try std.time.Instant.now();
    var y_rot: f32 = 0.5;
    var hue: f32 = 0;

    while (glfw.glfwWindowShouldClose(window) == 0) {
        const now = try std.time.Instant.now();
        defer last = now;
        var elapsed: f32 = @floatFromInt(now.since(last));
        elapsed /= std.time.ns_per_s;

        gl.glClearColor(0.1, 0.2, 0.3, 1.0);
        gl.glClearDepth(std.math.inf(f32));
        gl.glClear(gl.GL_COLOR_BUFFER_BIT | gl.GL_DEPTH_BUFFER_BIT);

        y_rot += elapsed * 2 * std.math.pi * 0.25;
        while (y_rot > 2 * std.math.pi) {
            y_rot -= 2 * std.math.pi;
        }

        hue += elapsed * 30;
        hue = @mod(hue, 360);

        const x = matMul(
            makeYRotation(y_rot),
            makeZRotation(0.5),
        );
        const obj_matrix = matMul(translation, x);

        gl.glUseProgram(program);
        gl.glUniformMatrix4fv(world_transform_uniform, 1, gl.GL_TRUE, &obj_matrix);

        const rgb = hueToRgb(hue);
        gl.glUniform3f(color_uniform, rgb[0], rgb[1], rgb[2]);

        gl.glBindVertexArray(vao);
        gl.glDrawArrays(gl.GL_TRIANGLES, 0, 36);

        glfw.glfwSwapBuffers(window);
        glfw.glfwPollEvents();
    }

    glfw.glfwTerminate();
}

pub fn errorCallback(err: c_int, msg: [*c]const u8) callconv(.C) void {
    _ = err;
    panic("GLFW error: {s}\n", .{msg});
}
