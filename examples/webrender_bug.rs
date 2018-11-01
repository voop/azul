extern crate azul;

use azul::prelude::*;

struct WebrenderTestCase { }

const CSS: &str = "
    #parent { background-color: #2f3136; }
    #child { width: 100px; height: 100px; background-color: red; }
";

impl Layout for WebrenderTestCase {
    fn layout(&self, _info: WindowInfo<Self>) -> Dom<Self>
    {
        // Uncomment the next block (and comment out the other block) to see the actual (correct) color:
        // Parent should have a color of #2f3136, compare this with the color in the Chrome color
        // picker - notice that it's much lighter! Child should have a color of red in both cases
/*
        Dom::new(NodeType::Div).with_id("parent")
            .with_child(Dom::new(NodeType::Div).with_id("child"))
*/

        let callback = GlTextureCallback(render_opengl_texture);
        let ptr = StackCheckedPointer::new(self, self).unwrap();

        Dom::new(NodeType::Div).with_id("parent")
            .with_child(Dom::new(NodeType::GlTexture((callback, ptr))).with_id("child"))
    }
}

fn render_opengl_texture(_ptr: &StackCheckedPointer<WebrenderTestCase>, info: WindowInfo<WebrenderTestCase>, dimensions: HidpiAdjustedBounds) -> Option<Texture> {
    use azul::glium::Surface;
    let window = info.window.read_only_window();
    let tex = window.create_texture(dimensions.physical_size.width as u32, dimensions.physical_size.height as u32);
    tex.as_surface().clear_color(1.0, 0.0, 0.0, 1.0); // clear the texture with red
    window.unbind_framebuffer();
    Some(tex)
}

fn main() {
    let app = App::new(WebrenderTestCase { }, AppConfig::default());
    app.run(Window::new(WindowCreateOptions::default(), Css::new_from_str(CSS).unwrap()).unwrap()).unwrap();
}