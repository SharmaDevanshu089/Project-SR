import bpy, numpy as np

obj = bpy.data.objects["Terrain_3D"]
print("Dimensions:", obj.dimensions)
print("Bound box (local):", [tuple(v) for v in obj.bound_box])

mod = obj.modifiers.get("Displace_Height")
if mod and mod.texture and mod.texture.image:
    img = mod.texture.image
    print("Height image size:", img.size[:], "colorspace:", img.colorspace_settings.name)
    px = np.array(img.pixels[:]).reshape(img.size[1], img.size[0], img.channels)
    print("min/max/mean:", px.min(), px.max(), px.mean())
    print("any NaN:", np.isnan(px).any(), " any Inf:", np.isinf(px).any())
    print("mod.strength / mid_level:", mod.strength, mod.mid_level)
else:
    print("Modifier or its image is missing/unlinked")