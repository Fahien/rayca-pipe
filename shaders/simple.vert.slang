// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

struct Model {
    float4x4 transform;
}

struct Camera {
    float4x4 view;
    float4x4 proj;
}

ConstantBuffer<Model> model;
[vk::binding(0, 1)]
ConstantBuffer<Camera> camera;

[shader("vertex")]
float4 main(float3 pos) : SV_Position {
    return float4(pos, 1.0);
}
