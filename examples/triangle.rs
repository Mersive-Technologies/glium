#[macro_use]
extern crate glium;

mod support;

#[allow(unused_imports)]
use glium::{glutin, Surface};
use glium::index::PrimitiveType;
use crate::glutin::dpi::PhysicalSize;

fn main() {
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();

    let cb = glutin::ContextBuilder::new();
    let size = PhysicalSize {
        width: 800,
        height: 600,
    };
    let context = cb.build_headless(&event_loop, size).unwrap();
    let context = unsafe {
        context.treat_as_current()
    };
    let display = glium::backend::glutin::headless::Headless::new(context).unwrap();

    // building the vertex buffer, which contains all the vertices that we will draw
    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
            color: [f32; 3],
        }

        implement_vertex!(Vertex, position, color);

        glium::VertexBuffer::new(&display,
            &[
                Vertex { position: [-0.5, -0.5], color: [0.0, 1.0, 0.0] },
                Vertex { position: [ 0.0,  0.5], color: [0.0, 0.0, 1.0] },
                Vertex { position: [ 0.5, -0.5], color: [1.0, 0.0, 0.0] },
            ]
        ).unwrap()
    };

    // building the index buffer
    let index_buffer = glium::IndexBuffer::new(&display, PrimitiveType::TrianglesList,
                                               &[0u16, 1, 2]).unwrap();

    // compiling shaders and linking them together
    let program = program!(&display,
        140 => {
            vertex: "
                #version 140

                uniform mat4 matrix;

                in vec2 position;
                in vec3 color;

                out vec3 vColor;

                void main() {
                    gl_Position = vec4(position, 0.0, 1.0) * matrix;
                    vColor = color;
                }
            ",

            fragment: "
                #version 140
                in vec3 vColor;
                out vec4 f_color;

                void main() {
                    f_color = vec4(vColor, 1.0);
                }
            "
        },

        110 => {
            vertex: "
                #version 110

                uniform mat4 matrix;

                attribute vec2 position;
                attribute vec3 color;

                varying vec3 vColor;

                void main() {
                    gl_Position = vec4(position, 0.0, 1.0) * matrix;
                    vColor = color;
                }
            ",

            fragment: "
                #version 110
                varying vec3 vColor;

                void main() {
                    gl_FragColor = vec4(vColor, 1.0);
                }
            ",
        },

        100 => {
            vertex: "
                #version 100

                uniform lowp mat4 matrix;

                attribute lowp vec2 position;
                attribute lowp vec3 color;

                varying lowp vec3 vColor;

                void main() {
                    gl_Position = vec4(position, 0.0, 1.0) * matrix;
                    vColor = color;
                }
            ",

            fragment: "
                #version 100
                varying lowp vec3 vColor;

                void main() {
                    gl_FragColor = vec4(vColor, 1.0);
                }
            ",
        },
    ).unwrap();

    // Here we draw the black background and triangle to the screen using the previously
    // initialised resources.
    //
    // In this case we use a closure for simplicity, however keep in mind that most serious
    // applications should probably use a function that takes the resources as an argument.
    let draw = move || {
        // building the uniforms
        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32]
            ]
        };

        // drawing a frame
        let mut target = display.draw();
        target.clear_color(1.0, 0.0, 0.0, 1.0);
        target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &Default::default()).unwrap();
        target.finish().unwrap();

        let image: glium::texture::RawImage2d<'_, u8> = display.read_front_buffer().unwrap();
        let image = image::ImageBuffer::from_raw(image.width, image.height, image.data.into_owned()).unwrap();
        let image = image::DynamicImage::ImageRgba8(image).flipv();
        image.save("glium-example-screenshot.png").unwrap();
    };

    // Draw the triangle to the screen.
    draw();

}
