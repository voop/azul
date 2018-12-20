use {
    window::{WindowEvent, WindowInfo},
    text_cache::TextId,
    app_state::AppState,
    id_tree::{NodeId, Node, Arena, NodeHierarchy, NodeDataContainer},
    default_callbacks::{DefaultCallbackId, StackCheckedPointer},
    window::HidpiAdjustedBounds,
    text_layout::{Words, FontMetrics, TextSizePx},
};

/// Returns the preferred width, for example for an image, that would be the
/// original width (an image always wants to take up the original space)
pub(crate) fn get_preferred_width<T: Layout>(node_type: &NodeType<T>, image_cache: &FastHashMap<ImageId, ImageState>) -> Option<f32> {
    match node_type {
        NodeType::Image(i) => image_cache.get(i).and_then(|image_state| Some(image_state.get_dimensions().0)),
        NodeType::Label(_) | Text(_) => /* TODO: Calculate the minimum width for the text? */ None,
        _ => None,
    }
}

/// Given a certain width, returns the
pub(crate) fn get_preferred_height_based_on_width<T: Layout>(
    node_type: &NodeType<T>,
    div_width: TextSizePx,
    image_cache: &FastHashMap<ImageId, ImageState>,
    words: Option<&Words>,
    font_metrics: Option<FontMetrics>,
) -> Option<TextSizePx>
{
    use azul_css::{LayoutOverflow, TextOverflowBehaviour, TextOverflowBehaviourInner};

    match node_type {
        NodeType::Image(i) => image_cache.get(i).and_then(|image_state| {
            let (image_original_height, image_original_width) = image_state.get_dimensions();
            Some(div_width * (image_original_width / image_original_height))
        }),
        NodeType::Label(_) | NodeType::Text(_) => {
            let (words, font) = (words?, font_metrics?);
            let vertical_info = words.get_vertical_height(&LayoutOverflow {
                horizontal: TextOverflowBehaviour::Modified(TextOverflowBehaviourInner::Scroll),
                .. Default::default()
            }, &font, div_width);
            Some(vertical_info.vertical_height)
        }
        _ => None,
    }
}