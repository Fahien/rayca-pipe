// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

[vk::binding(0, 2)]
ConstantBuffer<float4> color;

[vk::binding(1, 2)]
Texture2D albedo;

[vk::binding(0, 3)]
[vk::input_attachment_index(0)]
SubpassInput scene_color;

[shader("fragment")]
float4 main() : SV_Target {
    return color * scene_color.SubpassLoad();
}
