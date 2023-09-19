/*!
cfg_if! branches are separated into functions mainly to help the LSP
*/

use std::io::{BufReader, Cursor};
#[cfg(not(target_arch = "wasm32"))]
use std::{
    fs::{self},
    path::Path,
};

use anyhow::{anyhow, Result};
use cfg_if::cfg_if;
use wgpu::util::DeviceExt;

use crate::{model, texture};

#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let origin = location.origin().unwrap();

    reqwest::Url::parse(&origin)
        .unwrap()
        .join("assets/")
        .unwrap()
        .join(file_name)
        .unwrap()
}

pub async fn load_string(file_name: &str) -> Result<String> {
    cfg_if! {
        if #[cfg(target_arch =  "wasm32")] {
            load_string_wasm(file_name).await
        } else {
            load_string_native(file_name).await
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[inline]
async fn load_string_wasm(file_name: &str) -> Result<String> {
    let url = format_url(file_name);
    let txt = reqwest::get(url).await?.text().await?;

    return Ok(txt);
}

#[cfg(not(target_arch = "wasm32"))]
#[inline]
async fn load_string_native(file_name: &str) -> Result<String> {
    let public_dir = Path::new("public/assets");
    let render_dir = Path::new("assets");
    let path = if public_dir.exists() {
        public_dir.join(file_name)
    } else {
        render_dir.join(file_name)
    };
    let txt = fs::read_to_string(path)?;

    Ok(txt)
}

pub async fn load_binary(file_name: &str) -> Result<Vec<u8>> {
    cfg_if! {
        if #[cfg(target_arch =  "wasm32")] {
            load_binary_wasm(file_name).await
        } else {
            load_binary_native(file_name).await
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[inline]
async fn load_binary_wasm(file_name: &str) -> Result<Vec<u8>> {
    let url = format_url(file_name);
    let data = reqwest::get(url).await?.bytes().await?.to_vec();

    Ok(data)
}

#[cfg(not(target_arch = "wasm32"))]
#[inline]
async fn load_binary_native(file_name: &str) -> Result<Vec<u8>> {
    let public_dir = Path::new("public/assets");
    let render_dir = Path::new("assets");

    let path = if public_dir.exists() {
        public_dir.join(file_name)
    } else {
        render_dir.join(file_name)
    };

    let data = fs::read(path)?;

    Ok(data)
}

pub async fn load_texture(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> Result<texture::Texture> {
    let data = load_binary(file_name).await?;
    texture::Texture::from_bytes(device, queue, &data, file_name)
}

pub async fn load_model(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
) -> Result<model::Model> {
    let obj_text = load_string(file_name).await?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    let (models, obj_materials) = tobj::load_obj_buf_async(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| async move {
            let mat_text = load_string(&p).await.unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    )
    .await?;

    let materials = futures::future::try_join_all(obj_materials?.into_iter().map(|m| async {
        let diffuse_texture = load_texture(
            &m.diffuse_texture
                .ok_or(anyhow!("Material dosn't have a texture name."))?,
            device,
            queue,
        )
        .await?;

        let texture_view = wgpu::BindingResource::TextureView(&diffuse_texture.view);
        let texture_sampler = wgpu::BindingResource::Sampler(&diffuse_texture.sampler);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: texture_view,
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: texture_sampler,
                },
            ],
        });

        Ok::<_, anyhow::Error>(model::Material {
            name: m.name,
            diffuse_texture,
            bind_group,
        })
    }))
    .await?;

    let meshes = models
        .into_iter()
        .map(|m| {
            let vertices: Vec<_> = (0..m.mesh.positions.len() / 3)
                .map(|i| {
                    anyhow::Ok(model::ModelVertex {
                        position: m.mesh.positions[i * 3..i * 3 + 3].try_into()?,
                        tex_coords: m.mesh.texcoords[i * 2..i * 2 + 2].try_into()?,
                        normal: m.mesh.normals[i * 3..i * 3 + 3].try_into()?,
                    })
                })
                .collect::<Result<_>>()?;

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", file_name)),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", file_name)),
                contents: bytemuck::cast_slice(&m.mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            Ok(model::Mesh {
                name: file_name.to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
                material: m.mesh.material_id.unwrap_or(0),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(model::Model { meshes, materials })
}
