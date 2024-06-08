struct Matrix {
    size: vec2<u32>,
    numbers: array<f32>
}

@group(0) @binding(0) var<storage, read> lhsMatrix: Matrix;
@group(0) @binding(1) var<storage, read> rhsMatrix: Matrix;
@group(0) @binding(2) var<storage, read_write> outMatrix: Matrix;

@compute @workgroup_size(8, 8)
fn add(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if global_id.x >= u32(lhsMatrix.size.x) || global_id.y >= u32(rhsMatrix.size.y) {
        return;
    }

    outMatrix.size = vec2(lhsMatrix.size.x, rhsMatrix.size.y);
    let off = global_id.x * u32(lhsMatrix.size.y) + global_id.y;
    outMatrix.numbers[off] = lhsMatrix.numbers[off] + rhsMatrix.numbers[off];
}
