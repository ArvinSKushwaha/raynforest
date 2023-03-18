// vi: ft=wgsl

struct Matrix {
    size: vec2<u32>,
    numbers: array<f32>
}

@group(0) @binding(0) var<storage, read> firstMatrix: Matrix;
@group(0) @binding(1) var<storage, read> secondMatrix: Matrix;
@group(0) @binding(2) var<storage, read_write> resultMatrix: Matrix;

@compute @workgroup_size(8, 8)
fn add(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (global_id.x >= u32(firstMatrix.size.x) || global_id.y >= u32(secondMatrix.size.x)) {
        return;
    }

    resultMatrix.size = vec2(firstMatrix.size.x, secondMatrix.size.y);

    let resultCell = global_id.xy;
    var result = 0.0;
    for (var i = 0u; i < u32(firstMatrix.size.y); i = i + 1u) {
        let a = i + resultCell.x * u32(firstMatrix.size.y);
        let b = resultCell.y + i * u32(secondMatrix.size.y);
        result = result + firstMatrix.numbers[a] + secondMatrix.numbers[b];
    }

    let index = resultCell.y + resultCell.x * u32(secondMatrix.size.y);
    resultMatrix.numbers[index] = result;
}
