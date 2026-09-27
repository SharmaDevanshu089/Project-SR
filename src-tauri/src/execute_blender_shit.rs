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

bpy.ops.mesh.primitive_grid_add(x_subdivisions=256, y_subdivisions=256, size=10.0, location=(0, 0, 0))
terrain = bpy.context.active_object
terrain.name = "Terrain_3D"

for poly in terrain.data.polygons:
    poly.use_smooth = True

height_map_path = os.path.abspath("height_map.tiff")
if not os.path.exists(height_map_path) and os.path.exists("../height_map.tiff"):
    height_map_path = os.path.abspath("../height_map.tiff")

texture_path = os.path.abspath("output_super_res_visual.png")
if not os.path.exists(texture_path) and os.path.exists("../output_super_res_visual.png"):
    texture_path = os.path.abspath("../output_super_res_visual.png")

if os.path.exists(height_map_path):
    disp_tex = bpy.data.textures.new("HeightMapTexture", type='IMAGE')
    disp_img = bpy.data.images.load(height_map_path)
    disp_tex.image = disp_img
    
    mod = terrain.modifiers.new(name="Displace_Height", type='DISPLACE')
    mod.texture = disp_tex
    mod.texture_coords = 'UV'
    mod.mid_level = 0.0
    mod.strength = 1.5

mat = bpy.data.materials.new(name="Terrain_Material")
mat.use_nodes = True
nodes = mat.node_tree.nodes
links = mat.node_tree.links
nodes.clear()

node_output = nodes.new(type='ShaderNodeOutputMaterial')
node_output.location = (400, 0)

node_bsdf = nodes.new(type='ShaderNodeBsdfPrincipled')
node_bsdf.location = (100, 0)
links.new(node_bsdf.outputs['BSDF'], node_output.inputs['Surface'])

if os.path.exists(texture_path):
    node_tex = nodes.new(type='ShaderNodeTexImage')
    node_tex.location = (-300, 100)
    node_tex.image = bpy.data.images.load(texture_path)
    links.new(node_tex.outputs['Color'], node_bsdf.inputs['Base Color'])

if os.path.exists(height_map_path):
    node_disp_img = nodes.new(type='ShaderNodeTexImage')
    node_disp_img.location = (-300, -200)
    node_disp_img.image = bpy.data.images.load(height_map_path)
    
    node_disp = nodes.new(type='ShaderNodeDisplacement')
    node_disp.location = (100, -200)
    node_disp.inputs['Scale'].default_value = 1.5
    links.new(node_disp_img.outputs['Color'], node_disp.inputs['Height'])
    links.new(node_disp.outputs['Displacement'], node_output.inputs['Displacement'])

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

#[tauri::command]
pub fn execute_blender_shit() -> Result<String, String> {
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
    if Path::new(blend_filename).exists() || Path::new("..").join(blend_filename).exists() {
        Ok("Successfully generated 3D terrain object in terrain.blend".to_string())
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(format!("Blender ran successfully: {}", stdout))
    }
}
