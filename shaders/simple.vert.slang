// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

[vk::binding(0, 0)]
ConstantBuffer<float4x4> model;

[vk::binding(0, 1)]
ConstantBuffer<float4x4> view;
[vk::binding(1, 1)]
ConstantBuffer<float4x4> proj;

struct PushConstants
{
    float4x4 pretransform;
};

[vk::push_constant]
PushConstants constants;

[shader("vertex")]
float4 main(float3 pos) : SV_Position
{
    return mul(constants.pretransform, mul(proj, mul(view, mul(model, float4(pos, 1.0)))));
}
