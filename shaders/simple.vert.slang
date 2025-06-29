// Copyright © 2021-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

[shader("vertex")]
float4 main(float3 pos) : SV_Position {
    return float4(pos, 1.0);
}
