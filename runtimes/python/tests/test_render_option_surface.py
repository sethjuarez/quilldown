from quilldown_spec import RenderOptions


def test_render_options_include_full_render_surface() -> None:
    data = {
        "strict": True,
        "image_dpi": 144.0,
        "embed_svg": False,
        "svg_light_mode": False,
        "max_image_width_px": 480,
        "base_dir": "assets",
        "highlight_code": False,
        "theme": "solarized",
        "page_size": "a4",
        "orientation": "landscape",
        "margin": 0.5,
        "page_numbers": True,
        "table_of_contents": True,
        "language": "fr-FR",
        "allow_remote_images": True,
        "captions": True,
    }
    assert RenderOptions.load(data).save() == data
