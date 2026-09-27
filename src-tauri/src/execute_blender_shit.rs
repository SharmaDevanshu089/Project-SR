use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn find_blender() -> Option<PathBuf> {
    let p = PathBuf::from(r"C:\Blender\blender.exe");
    if p.exists() {
        return Some(p);
    }

    if let Ok(path) = env::var("BLENDER_PATH") {
        let env_path = PathBuf::from(path);
        if env_path.exists() {
            return Some(env_path);
        }
    }

    if Command::new("blender").arg("--version").output().is_ok() {
        return Some(PathBuf::from("blender"));
    }

    None
}

const BLENDER_SCRIPT: &str = r#"
import bpy
import os

bpy.ops.wm.read_factory_settings(use_empty=True)

grid_size = 10.0
bpy.ops.mesh.primitive_grid_add(x_subdivisions=256, y_subdivisions=256, size=grid_size, location=(0, 0, 0))
terrain = bpy.context.active_object
terrain.name = "Terrain_3D"

for poly in terrain.data.polygons:
    poly.use_smooth = True

height_path = "height_map_u16.png" if os.path.exists("height_map_u16.png") else "../height_map_u16.png"
if not os.path.exists(height_path):
    height_path = "height_map.tiff" if os.path.exists("height_map.tiff") else "../height_map.tiff"

height_path = os.path.abspath(height_path)

if os.path.exists(height_path):
    disp_img = bpy.data.images.load(height_path, check_existing=False)
    disp_img.colorspace_settings.name = "Non-Color"
    disp_tex = bpy.data.textures.new("HeightMapTexture", type='IMAGE')
    disp_tex.image = disp_img
    
    mod = terrain.modifiers.new(name="Displace_Height", type='DISPLACE')
    mod.texture = disp_tex
    mod.texture_coords = 'UV'
    mod.mid_level = 0.0
    mod.strength = 0.72

mat = bpy.data.materials.new(name="Terrain_Material")
nodes = mat.node_tree.nodes
links = mat.node_tree.links
nodes.clear()

node_output = nodes.new(type='ShaderNodeOutputMaterial')
node_output.location = (400, 0)

node_bsdf = nodes.new(type='ShaderNodeBsdfPrincipled')
node_bsdf.location = (100, 0)
links.new(node_bsdf.outputs['BSDF'], node_output.inputs['Surface'])

texture_path = "output_super_res_visual.png" if os.path.exists("output_super_res_visual.png") else "../output_super_res_visual.png"
if not os.path.exists(texture_path):
    texture_path = "texture_preview.png" if os.path.exists("texture_preview.png") else "../texture_preview.png"

texture_path = os.path.abspath(texture_path)

if os.path.exists(texture_path):
    node_tex = nodes.new(type='ShaderNodeTexImage')
    node_tex.location = (-300, 0)
    node_tex.image = bpy.data.images.load(texture_path, check_existing=False)
    links.new(node_tex.outputs['Color'], node_bsdf.inputs['Base Color'])

terrain.data.materials.append(mat)

bpy.ops.object.light_add(type='SUN', location=(5, -5, 10))
sun = bpy.context.active_object
sun.data.energy = 3.0

bpy.ops.object.camera_add(location=(0, -12, 8), rotation=(1.05, 0, 0))
camera = bpy.context.active_object
bpy.context.scene.camera = camera

output_blend = os.path.abspath("terrain.blend")
bpy.ops.wm.save_as_mainfile(filepath=output_blend)
print("SUCCESS: Saved terrain.blend")
"#;

fn find_python() -> PathBuf {
    if Path::new(r".\.venv\Scripts\python.exe").exists() {
        PathBuf::from(r".\.venv\Scripts\python.exe")
    } else if Path::new(r"..\.venv\Scripts\python.exe").exists() {
        PathBuf::from(r"..\.venv\Scripts\python.exe")
    } else {
        PathBuf::from("python")
    }
}

#[tauri::command]
pub fn execute_blender_shit() -> Result<String, String> {
    let py = find_python();
    let norm_script = r#"
import os, numpy as np
try:
    import rasterio
    from PIL import Image
    f = "height_map.tiff" if os.path.exists("height_map.tiff") else "../height_map.tiff"
    if os.path.exists(f):
        with rasterio.open(f) as src:
            d = src.read(1)
            mn, mx = float(d.min()), float(d.max())
            rng = mx - mn if mx > mn else 1.0
            norm = np.clip((d - mn) / rng, 0.0, 1.0)
            u16 = (norm * 65535.0).astype(np.uint16)
            Image.fromarray(u16).save("height_map_u16.png")
            if os.path.exists("../package.json"):
                Image.fromarray(u16).save("../height_map_u16.png")
except Exception as e:
    print(e)
"#;
    let _ = Command::new(&py).arg("-c").arg(norm_script).output();

    let script_filename = "create_terrain.py";
    let _ = fs::write(script_filename, BLENDER_SCRIPT);

    if Path::new("../package.json").exists() {
        let parent_script = Path::new("..").join(script_filename);
        let _ = fs::write(parent_script, BLENDER_SCRIPT);
    }

    let blender_bin = match find_blender() {
        Some(bin) => bin,
        None => {
            return Err("Blender not found. Please install standard Blender or set BLENDER_PATH environment variable.".to_string());
        }
    };

    let output = Command::new(&blender_bin)
        .arg("-b")
        .arg("-P")
        .arg(script_filename)
        .output()
        .map_err(|e| format!("Failed to run Blender: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Blender execution failed: {}", stderr));
    }

    let blend_filename = "terrain.blend";
    if Path::new(blend_filename).exists() {
        if Path::new("../package.json").exists() {
            let _ = fs::copy(blend_filename, Path::new("..").join(blend_filename));
        }
        Ok("Successfully generated 3D terrain object in terrain.blend".to_string())
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(format!("Blender ran successfully: {}", stdout))
    }
}
