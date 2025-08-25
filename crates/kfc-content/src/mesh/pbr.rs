use std::io::Cursor;

use half::f16;
use kfc::io::{ReadExt, WriteExt};

use crate::math::*;

#[derive(Debug, Clone)]
pub struct PbrVertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub tangent: Vec4,
    pub uv: Vec2,
    pub color: [u8; 4],
}

pub fn decode_pbr_vertex(
    data: &[u32],
    offset: Vec3,
    scale: Vec3,
) -> PbrVertex {
    let packed_vertex_1 = data[0];
    let packed_vertex_2 = data[1];
    let packed_normal = data[2];
    let packed_tangent = data[3];
    let packed_color = data[4];
    let packed_uv = data[5];

    let position = Vec3::new(
        (packed_vertex_1 >> 11) as f32,
        (((packed_vertex_1 & 0x7FF) << 10) | (packed_vertex_2 >> 21)) as f32,
        (packed_vertex_2 & 0x1FFFFF) as f32,
    ) * scale + offset;

    let normal = Vec3::new(
        ((packed_normal << 22) >> 22) as f32 / 511.0,
        (((packed_normal << 12) >> 22) as f32) / 511.0,
        (((packed_normal << 2) >> 22) as f32) / 511.0,
    );

    let tangent = Vec4::new(
        ((packed_tangent << 22) >> 22) as f32 / 511.0,
        (((packed_tangent << 12) >> 22) as f32) / 511.0,
        (((packed_tangent << 2) >> 22) as f32) / 511.0,
        (packed_tangent >> 30) as f32,
    );

    let uv = Vec2::new(
        f16::from_bits((packed_uv & 0xFFFF) as u16).to_f32(),
        f16::from_bits((packed_uv >> 16) as u16).to_f32(),
    );

    let color = [
        (packed_color & 0xFF) as u8,
        ((packed_color >> 8) & 0xFF) as u8,
        ((packed_color >> 16) & 0xFF) as u8,
        ((packed_color >> 24) & 0xFF) as u8,
    ];

    PbrVertex {
        position,
        normal,
        tangent,
        color,
        uv,
    }
}

pub fn decode_pbr_to_vertices(
    data: &[u8],
    offset: Vec3,
    scale: Vec3,
) -> std::io::Result<Vec<PbrVertex>> {
    let mut vertices = Vec::with_capacity(data.len() / 24);
    let mut cursor = Cursor::new(data);
    let mut buf = [0u32; 6];

    while cursor.position() < data.len() as u64 {
        for e in buf.iter_mut() {
            *e = cursor.read_u32()?;
        }

        let vertex = decode_pbr_vertex(&buf, offset, scale);
        vertices.push(vertex);
    }

    Ok(vertices)
}

pub fn decode_pbr_to_bytes(
    data: &[u8],
    count: usize,
    offset: Vec3,
    scale: Vec3,
) -> std::io::Result<Vec<u8>> {
    let result = Vec::with_capacity(count * 52);
    let mut reader = Cursor::new(data);
    let mut writer = Cursor::new(result);
    let mut buf = [0u32; 6];

    for _ in 0..count {
        for e in buf.iter_mut() {
            *e = reader.read_u32()?;
        }

        let vertex = decode_pbr_vertex(&buf, offset, scale);

        writer.write_f32(vertex.position.x)?;
        writer.write_f32(vertex.position.y)?;
        writer.write_f32(vertex.position.z)?;

        writer.write_f32(vertex.normal.x)?;
        writer.write_f32(vertex.normal.y)?;
        writer.write_f32(vertex.normal.z)?;

        writer.write_f32(vertex.tangent.x)?;
        writer.write_f32(vertex.tangent.y)?;
        writer.write_f32(vertex.tangent.z)?;
        writer.write_f32(vertex.tangent.w)?;

        writer.write_f32(vertex.uv.x)?;
        writer.write_f32(vertex.uv.y)?;

        writer.write_u8(vertex.color[0])?;
        writer.write_u8(vertex.color[1])?;
        writer.write_u8(vertex.color[2])?;
        writer.write_u8(vertex.color[3])?;
    }

    Ok(writer.into_inner())
}
