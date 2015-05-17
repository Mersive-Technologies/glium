/*!
Contains everything related to vertex sources.

When you draw, you need to pass one or several sources of vertex attributes. This is done with
the first parameter to the `draw` function.

## Vertex

The main trait of this module is `Vertex`, which must be implemented on structs whose instances
describe individual vertices. The trait is unsafe to implement, so you are encouraged to use the
`implement_vertex!` macro instead:

```
# #[macro_use]
# extern crate glium;
# extern crate glutin;
# fn main() {
#[derive(Copy, Clone)]
struct MyVertex {
    position: [f32; 3],
    texcoords: [f32; 2],
}

// you must pass the list of members to the macro
implement_vertex!(MyVertex, position, texcoords);
# }
```

## Vertex buffer

Once you have a struct that implements the `Vertex` trait, you can build an array of vertices and
upload it to the video memory by creating a `VertexBuffer`.

```no_run
# let display: glium::Display = unsafe { ::std::mem::uninitialized() };
# #[derive(Copy, Clone)]
# struct MyVertex {
#     position: [f32; 3],
#     texcoords: [f32; 2],
# }
# impl glium::vertex::Vertex for MyVertex {
#     fn build_bindings() -> glium::vertex::VertexFormat { unimplemented!() }
# }
let data = &[
    MyVertex {
        position: [0.0, 0.0, 0.4],
        texcoords: [0.0, 1.0]
    },
    MyVertex {
        position: [12.0, 4.5, -1.8],
        texcoords: [1.0, 0.5]
    },
    MyVertex {
        position: [-7.124, 0.1, 0.0],
        texcoords: [0.0, 0.4]
    },
];

let vertex_buffer = glium::vertex::VertexBuffer::new(&display, data);
```

## Drawing

When you draw, you can pass either a single vertex source or a tuple of multiple sources.
Each source can be:

 - A reference to a `VertexBuffer`.
 - A slice of a vertex buffer, by calling `vertex_buffer.slice(start .. end).unwrap()`.
 - A vertex buffer where each element corresponds to an instance, by
   caling `vertex_buffer.per_instance()`.
 - The same with a slice, by calling `vertex_buffer.slice(start .. end).unwrap().per_instance()`.
 - A marker indicating a number of vertex sources, with `glium::vertex::EmptyVertexAttributes`.
 - A marker indicating a number of instances, with `glium::vertex::EmptyInstanceAttributes`.

```no_run
# use glium::Surface;
# let display: glium::Display = unsafe { ::std::mem::uninitialized() };
# #[derive(Copy, Clone)]
# struct MyVertex { position: [f32; 3], texcoords: [f32; 2], }
# impl glium::vertex::Vertex for MyVertex {
#     fn build_bindings() -> glium::vertex::VertexFormat { unimplemented!() }
# }
# let program: glium::program::Program = unsafe { ::std::mem::uninitialized() };
# let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
# let uniforms = glium::uniforms::EmptyUniforms;
# let vertex_buffer: glium::vertex::VertexBuffer<MyVertex> = unsafe { ::std::mem::uninitialized() };
# let vertex_buffer2: glium::vertex::VertexBuffer<MyVertex> = unsafe { ::std::mem::uninitialized() };
# let mut frame = display.draw();
// drawing with a single vertex buffer
frame.draw(&vertex_buffer, &indices, &program, &uniforms, &Default::default()).unwrap();

// drawing with two parallel vertex buffers
frame.draw((&vertex_buffer, &vertex_buffer2), &indices, &program,
           &uniforms, &Default::default()).unwrap();

// drawing without a vertex source
frame.draw(glium::vertex::EmptyVertexAttributes { len: 12 }, &indices, &program,
           &uniforms, &Default::default()).unwrap();

// drawing a slice of a vertex buffer
frame.draw(vertex_buffer.slice(6 .. 24).unwrap(), &indices, &program,
           &uniforms, &Default::default()).unwrap();

// drawing slices of two vertex buffers
frame.draw((vertex_buffer.slice(6 .. 24).unwrap(), vertex_buffer2.slice(128 .. 146).unwrap()),
           &indices, &program, &uniforms, &Default::default()).unwrap();

// treating `vertex_buffer2` as a source of attributes per-instance instead of per-vertex
frame.draw((&vertex_buffer, vertex_buffer2.per_instance_if_supported().unwrap()), &indices,
           &program, &uniforms, &Default::default()).unwrap();

// instancing without any per-instance attribute
frame.draw((&vertex_buffer, glium::vertex::EmptyInstanceAttributes { len: 36 }), &indices,
           &program, &uniforms, &Default::default()).unwrap();
```

Note that if you use `index::EmptyIndices` as indices the length of all vertex sources must
be the same, or a `DrawError::VerticesSourcesLengthMismatch` will be produced.

In all situation, the length of all per-instance sources must match, or
`DrawError::InstancesCountMismatch` will be retured.

*/
use std::iter::Chain;
use std::option::IntoIter;

pub use self::buffer::{VertexBuffer, VertexBufferAny, Mapping};
pub use self::buffer::{VertexBufferSlice, VertexBufferAnySlice};
pub use self::format::{AttributeType, VertexFormat};

mod buffer;
mod format;

/// Describes the source to use for the vertices when drawing.
#[derive(Clone)]
pub enum VerticesSource<'a> {
    /// A buffer uploaded in the video memory.
    ///
    /// The second and third parameters are the offset and length of the buffer.
    /// The fourth parameter tells whether or not this buffer is "per instance" (true) or
    /// "per vertex" (false).
    VertexBuffer(&'a VertexBufferAny, usize, usize, bool),

    Marker { len: usize, per_instance: bool },
}

/// Objects that can be used as vertex sources.
pub trait IntoVerticesSource<'a> {
    /// Builds the `VerticesSource`.
    fn into_vertices_source(self) -> VerticesSource<'a>;
}

impl<'a> IntoVerticesSource<'a> for VerticesSource<'a> {
    fn into_vertices_source(self) -> VerticesSource<'a> {
        self
    }
}

/// Marker that can be passed instead of a buffer to indicate an empty list of buffers.
pub struct EmptyVertexAttributes { pub len: usize }

impl<'a> IntoVerticesSource<'a> for EmptyVertexAttributes {
    fn into_vertices_source(self) -> VerticesSource<'a> {
        VerticesSource::Marker { len: self.len, per_instance: false }
    }
}

/// Marker that can be passed instead of a buffer to indicate an empty list of buffers.
pub struct EmptyInstanceAttributes { pub len: usize }

impl<'a> IntoVerticesSource<'a> for EmptyInstanceAttributes {
    fn into_vertices_source(self) -> VerticesSource<'a> {
        VerticesSource::Marker { len: self.len, per_instance: true }
    }
}

/// Marker that instructs glium that the buffer is to be used per instance.
pub struct PerInstance<'a>(VertexBufferAnySlice<'a>);

impl<'a> IntoVerticesSource<'a> for PerInstance<'a> {
    fn into_vertices_source(self) -> VerticesSource<'a> {
        match self.0.into_vertices_source() {
            VerticesSource::VertexBuffer(buf, off, len, false) => {
                VerticesSource::VertexBuffer(buf, off, len, true)
            },
            _ => unreachable!()
        }
    }
}

/// Objects that describe multiple vertex sources.
pub trait MultiVerticesSource<'a> {
    /// Iterator that enumerates each source.
    type Iterator: Iterator<Item = VerticesSource<'a>>;

    /// Iterates over the `VerticesSource`.
    fn iter(self) -> Self::Iterator;
}

impl<'a, T> MultiVerticesSource<'a> for T
    where T: IntoVerticesSource<'a>
{
    type Iterator = IntoIter<VerticesSource<'a>>;

    fn iter(self) -> IntoIter<VerticesSource<'a>> {
        Some(self.into_vertices_source()).into_iter()
    }
}

macro_rules! impl_for_tuple {
    ($t:ident) => (
        impl<'a, $t> MultiVerticesSource<'a> for ($t,)
            where $t: IntoVerticesSource<'a>
        {
            type Iterator = IntoIter<VerticesSource<'a>>;

            fn iter(self) -> IntoIter<VerticesSource<'a>> {
                Some(self.0.into_vertices_source()).into_iter()
            }
        }
    );

    ($t1:ident, $t2:ident) => (
        #[allow(non_snake_case)]
        impl<'a, $t1, $t2> MultiVerticesSource<'a> for ($t1, $t2)
            where $t1: IntoVerticesSource<'a>, $t2: IntoVerticesSource<'a>
        {
            type Iterator = Chain<<($t1,) as MultiVerticesSource<'a>>::Iterator,
                                  <($t2,) as MultiVerticesSource<'a>>::Iterator>;

            fn iter(self) -> Chain<<($t1,) as MultiVerticesSource<'a>>::Iterator,
                                   <($t2,) as MultiVerticesSource<'a>>::Iterator>
            {
                let ($t1, $t2) = self;
                Some($t1.into_vertices_source()).into_iter().chain(($t2,).iter())
            }
        }

        impl_for_tuple!($t2);
    );

    ($t1:ident, $($t2:ident),+) => (
        #[allow(non_snake_case)]
        impl<'a, $t1, $($t2),+> MultiVerticesSource<'a> for ($t1, $($t2),+)
            where $t1: IntoVerticesSource<'a>, $($t2: IntoVerticesSource<'a>),+
        {
            type Iterator = Chain<<($t1,) as MultiVerticesSource<'a>>::Iterator,
                                  <($($t2),+) as MultiVerticesSource<'a>>::Iterator>;

            fn iter(self) -> Chain<<($t1,) as MultiVerticesSource<'a>>::Iterator,
                                  <($($t2),+) as MultiVerticesSource<'a>>::Iterator>
            {
                let ($t1, $($t2),+) = self;
                Some($t1.into_vertices_source()).into_iter().chain(($($t2),+).iter())
            }
        }

        impl_for_tuple!($($t2),+);
    );
}

impl_for_tuple!(A, B, C, D, E, F, G);

/// Trait for structures that represent a vertex.
///
/// Instead of implementing this trait yourself, it is recommended to use the `implement_vertex!`
/// macro instead.
// TODO: this should be `unsafe`, but that would break the syntax extension
pub trait Vertex: Copy + Sized {
    /// Builds the `VertexFormat` representing the layout of this element.
    fn build_bindings() -> VertexFormat;
}

/// Trait for types that can be used as vertex attributes.
pub unsafe trait Attribute: Sized {
    /// Get the type of data.
    fn get_type() -> AttributeType;
}
