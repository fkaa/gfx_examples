// Copyright 2014 The Gfx-rs Developers.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;

extern crate image;

use gfx::attrib::Floater;
use gfx::traits::*;

use std::path::Path;
use std::io::Cursor;

gfx_vertex!( Vertex {
    a_Pos@ pos: [f32; 2],
    a_Uv@ uv: [f32; 2],
});

impl Vertex {
    fn new(p: [f32; 2], u: [f32; 2]) -> Vertex {
        Vertex {
            pos: p,
            uv: u,
        }
    }
}

gfx_parameters!( Params/ParamsLink {
    t_Color@ color: gfx::shade::TextureParam<R>,
    t_Flow@ flow: gfx::shade::TextureParam<R>,
    t_Noise@ noise: gfx::shade::TextureParam<R>,
    f_Offset0@ offset0: f32,
    f_Offset1@ offset1: f32,
});

fn load_texture<R, F>(factory: &mut F, data: &[u8]) -> Result<gfx::handle::Texture<R>, String>
        where R: gfx::Resources, F: gfx::device::Factory<R> {
    let img = image::load(Cursor::new(data), image::PNG).unwrap();

    let mut img = match img {
        image::DynamicImage::ImageRgba8(img) => img,
        img => img.to_rgba()
    };
    let (width, height) = img.dimensions();
    let tex_info = gfx::tex::TextureInfo {
        width: width as u16,
        height: height as u16,
        depth: 1,
        levels: 1,
        kind: gfx::tex::TextureKind::Texture2D,
        format: gfx::tex::RGBA8
    };

    Ok(factory.create_texture_static(tex_info, &img).unwrap())
}

pub fn main() {
    let mut canvas = gfx_window_glutin::init(glutin::WindowBuilder::new()
        .with_dimensions(800, 600).build().unwrap())
        .into_canvas();
    canvas.output.window.set_title("Flowmap example");

    let vertex_data = [
        Vertex::new([-1.0, -1.0], [0.0, 0.0]),
        Vertex::new([ 1.0, -1.0], [1.0, 0.0]),
        Vertex::new([ 1.0,  1.0], [1.0, 1.0]),

        Vertex::new([-1.0, -1.0], [0.0, 0.0]),
        Vertex::new([ 1.0,  1.0], [1.0, 1.0]),
        Vertex::new([-1.0,  1.0], [0.0, 1.0]),
    ];

    let mesh = canvas.factory.create_mesh(&vertex_data);

    let water_texture = load_texture(&mut canvas.factory, &include_bytes!("image/water.png")[..]).unwrap();
    let flow_texture = load_texture(&mut canvas.factory, &include_bytes!("image/flow.png")[..]).unwrap();
    let noise_texture = load_texture(&mut canvas.factory, &include_bytes!("image/noise.png")[..]).unwrap();

    let program = {
        let vs = gfx::ShaderSource {
            glsl_120: Some(include_bytes!("shader/flowmap_120.glslv")),
            glsl_150: Some(include_bytes!("shader/flowmap_150.glslv")),
            .. gfx::ShaderSource::empty()
        };
        let fs = gfx::ShaderSource {
            glsl_120: Some(include_bytes!("shader/flowmap_120.glslf")),
            glsl_150: Some(include_bytes!("shader/flowmap_150.glslf")),
            .. gfx::ShaderSource::empty()
        };
        canvas.factory.link_program_source(vs, fs, &canvas.device.get_capabilities())
                      .unwrap()
    };

    let uniforms = Params {
        color: (water_texture, None),
        flow: (flow_texture, None),
        noise: (noise_texture, None),
        offset0: 0f32,
        offset1: 0.5f32,
        _r: std::marker::PhantomData,
    };
    let mut batch = gfx::batch::OwnedBatch::new(mesh, program, uniforms).unwrap();

    let mut cycle0 = 0.0f32;
    let mut cycle1 = 0.5f32;

    'main: loop {
        use std::f32;

        // quit when Esc is pressed.
        for event in canvas.output.window.poll_events() {
            match event {
                glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) => break 'main,
                glutin::Event::Closed => break 'main,
                _ => {},
            }
        }

        // since we sample our diffuse texture twice we need to lerp between
        // them to get a smooth transition (shouldn't even be noticable).

        // they start half a cycle apart (0.5) and is later used to calculate
        // the interpolation amount via `2.0 * abs(cycle0 - .5f)`
        cycle0 += 0.0025f32;
        if cycle0 > 1f32 {
            cycle0 -= 1f32;
        }

        cycle1 += 0.0025f32;
        if cycle1 > 1f32 {
            cycle1 -= 1f32;
        }

        batch.param.offset0 = cycle0;
        batch.param.offset1 = cycle1;

        canvas.clear(gfx::ClearData {
            color: [0.3, 0.3, 0.3, 1.0],
            depth: 1.0,
            stencil: 0,
        });

        canvas.draw(&batch).unwrap();

        canvas.present();
    }
}