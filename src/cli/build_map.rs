use std::collections::VecDeque;
use std::fs::{self, copy, File};
use std::io::{stdout, BufReader};
// use std::thread;
// use std::t&ime::Duration;
use draw::{render, Canvas, Color, Drawing, Shape, Style, SvgRenderer};
use image::{DynamicImage, ImageBuffer, Rgb, RgbImage};
use rbx_dom_weak::WeakDom;
use rbx_types::{Matrix3, Ref, Variant, Vector3};
use std::path::{Path, PathBuf};
use std::string;

use serde::Deserialize;
// use indicatif::{ProgressBar, ProgressStyle};
use anyhow::Context;
use structopt::StructOpt;
use tiny_skia::{Paint, PathBuilder, Pixmap, Transform};

use crate::math_lib;

const UNKNOWN_FILE_KIND_ERROR: &str = "Could not detect what kind of file to read. \
										Expected file to end in .rbxlx or .rbxl.";

#[derive(Deserialize)]
struct ConfigFileType {
    draw_everything: bool,
    world_files: Vec<ObjectFileType>,
}

#[derive(Deserialize)]
struct ObjectFileType {
    color: Vec<u8>,
    dir: Vec<String>,
    part_name: String,
}

/// Generates an image file representing a game map
#[derive(Debug, StructOpt)]
pub struct BuildMapCommand {
    /// Path to the place file
    ///
    /// Should end in .rbxl or .rbxlx
    #[structopt(long, short)]
    pub placefile: PathBuf,

    /// The Z height of the image in studs
    #[structopt(long = "height")]
    pub height: i32,

    /// The X width of the image in studs
    #[structopt(long = "width")]
    pub width: i32,

    /// The X Center of the image in world space
    #[structopt(long = "center_x")]
    pub center_x: f32,

    /// The Z Center of the image in world space
    #[structopt(long = "center_z")]
    pub center_z: f32,

    /// scale
    #[structopt(long = "scale")]
    pub scale: f32,

    /// config
    #[structopt(long = "config")]
    pub config: PathBuf,
}

impl BuildMapCommand {
    pub fn run(self) -> anyhow::Result<()> {
        log::trace!("Determining file type");
        println!("building..");
        let file_type = detect_file_kind(&self.placefile).context(UNKNOWN_FILE_KIND_ERROR)?;

        let file_reader = BufReader::new(File::open(&self.placefile)?);

        let config_raw_data = fs::read_to_string(self.config).expect("Invalid config file path");
        let config_data: ConfigFileType =
            serde_json::from_str(&config_raw_data).expect("JSON formatting issue with config file");

        //let mut world_pixel_data: Vec<Vec<i32>> = vec![vec![Default::default(); self.height.try_into().unwrap()]; self.width.try_into().unwrap()];

        //let mut one_dimensional_pixel_data: Vec<i32> = Vec::with_capacity((self.height * self.width).try_into().unwrap());

        match file_type {
            OutputKind::Rbxl => {
                let dom = rbx_binary::from_reader(file_reader)?;
                // get binary working first, then xml.
                // debug:
                println!("Root instances in file:");
                let root = dom.root();

                let mut img: RgbImage = ImageBuffer::new(self.width as u32, self.height as u32);

                let mut pixmap = Pixmap::new(self.width as u32, self.height as u32).unwrap();
                if config_data.draw_everything == true {
                    // draw everything :)
                    let workspace = root
                        .children()
                        .iter()
                        .find(|&&x| dom.get_by_ref(x).unwrap().name == "Workspace")
                        .context("Could not find workspace from file.")?;
                    let descendants = get_descendants(&dom, workspace)?;

                    for iref in descendants.iter() {
                        let part = dom.get_by_ref(*iref).unwrap();
                        if part.class == "Part" {
                            let cf = match part.properties.get("CFrame") {
                                Some(Variant::CFrame(v)) => v,
                                _ => panic!("Part does not have a cframe"),
                            };

                            let object_size = match part.properties.get("Size") {
                                Some(Variant::Vector3(v)) => v,
                                _ => panic!("Part does not have a size"),
                            };

                            let object_color = match part.properties.get("Color") {
                                Some(Variant::Color3uint8(v)) => v,
                                _ => panic!("Part does not have a color"),
                            };

                            let object_transparency = match part.properties.get("Transparency") {
                                Some(Variant::Float32(v)) => v,
                                _ => panic!("Part does not have transparency"),
                            };

                            let object_position =
                                Vector3::new(cf.position.x, cf.position.y, cf.position.z); //cf.position + Vector3::new(0f32, 0f32, 5000f32);
                            let object_orientation = cf.orientation;

                            let mut color: Vec<u8> = Vec::new();
                            color.push(object_color.r);
                            color.push(object_color.g);
                            color.push(object_color.b);
                            color.push(((1f32 - object_transparency) * 255f32).round() as u8);

                            //let mut c: [u8; 3] = [object_color.r, object_color.g, object_color.b];

                            let r_p = Vector3::new(
                                object_position.x * self.scale + self.center_x,
                                object_position.y * self.scale,
                                object_position.z * self.scale + self.center_z,
                            );
                            let s = Vector3::new(
                                object_size.x * self.scale,
                                object_size.y * self.scale,
                                object_size.z * self.scale,
                            );
                            //draw_part_to_imgbuf(&mut img, r_p, s, object_orientation, &c);
                            draw_part_on_pixmap(&mut pixmap, r_p, s, object_orientation, &color);
                        }
                    }
                } else {
                    // get world files and iterate through.
                    let world_data_files = config_data.world_files;
                    for object_data_file in world_data_files.iter() {
                        let object_part_name = &object_data_file.part_name;
                        let dir_path = &object_data_file.dir;
                        let mut inst = dom.root();
                        let mut inst_ref: &Ref = &Ref::new();
                        let mut stack = VecDeque::from_iter(dir_path.iter());
                        while let Some(cur_path) = stack.pop_front() {
                            inst_ref = inst
                                .children()
                                .iter()
                                .find(|&&x| dom.get_by_ref(x).unwrap().name == *cur_path)
                                .expect(format!("Unable to find instance {}", cur_path).as_str());
                            inst = dom.get_by_ref(*inst_ref).unwrap();
                        }
                        let descendants = get_descendants(&dom, inst_ref)?;

                        for iref in descendants.iter() {
                            let part = dom.get_by_ref(*iref).unwrap();
                            //println!("{}", part.name);
                            if &part.name == object_part_name {
                                //println!("did part");
                                let cf = match part.properties.get("CFrame") {
                                    Some(Variant::CFrame(v)) => v,
                                    _ => panic!("Part does not have a cframe"),
                                };

                                let object_size = match part.properties.get("Size") {
                                    Some(Variant::Vector3(v)) => v,
                                    _ => panic!("Part does not have a size"),
                                };

                                let object_position =
                                    Vector3::new(cf.position.x, cf.position.y, cf.position.z); //cf.position + Vector3::new(0f32, 0f32, 5000f32);
                                let object_orientation = cf.orientation;

                                let r_p = Vector3::new(
                                    object_position.x * self.scale + self.center_x,
                                    object_position.y * self.scale,
                                    object_position.z * self.scale + self.center_z,
                                );
                                let s = Vector3::new(
                                    object_size.x * self.scale,
                                    object_size.y * self.scale,
                                    object_size.z * self.scale,
                                );
                                draw_part_on_pixmap(
                                    &mut pixmap,
                                    r_p,
                                    s,
                                    object_orientation,
                                    &object_data_file.color,
                                );
                            }
                        }

                        println!("Should do {}", inst.name);
                    }
                }
                // let workspace = root.children().iter()
                // 					.find(|&&x| dom.get_by_ref(x).unwrap().name == "Workspace")
                // 					.context("Could not find workspace from file.")?;
                // println!("Found Workspace: {}", dom.get_by_ref(*workspace).unwrap().name);
                // // Attempt to find the map contents
                // let map_contents = dom
                // 					.get_by_ref(*workspace)
                // 					.unwrap().children().iter()
                // 					.find(|&&x| dom.get_by_ref(x).unwrap().name == "Map")
                // 					.context("Could not find map folder in workspace.")?;
                // println!("Found map folder in workspace. {}", dom.get_by_ref(*map_contents).unwrap().name);
                // let road_folder = dom
                // 					.get_by_ref(*map_contents)
                // 					.unwrap().children().iter()
                // 					.find(|&&x| dom.get_by_ref(x).unwrap().name == "Roads")
                // 					.context("Could not find road folder in map.")?;
                // println!("Found road folder in map. {}", dom.get_by_ref(*road_folder).unwrap().name);

                // //let mut document = Document::new().set("viewBox", (0, 0, self.width, self.height));

                // // let terrain_ref = dom
                // // 					.get_by_ref(*workspace)
                // // 					.unwrap().children().iter()
                // // 					.find(|&&x| dom.get_by_ref(x).unwrap().name == "Terrain")
                // // 					.context("Could not find terrain in workspace.")?;
                // // //println!("{:?}", dom.get_by_ref(*terrain_ref).unwrap().properties.get("SmoothGrid"));
                // // let terrain = dom.get_by_ref(*terrain_ref).unwrap();
                // // let grid = match terrain.properties.get("SmoothGrid") {
                // // 	Some(Variant::BinaryString(v)) => v,
                // // 	_ => panic!("No grid?"),
                // // };

                // for &referent in dom.get_by_ref(*road_folder).unwrap().children() {
                // 	let road_model = dom.get_by_ref(referent).unwrap();
                // 	// now we need to create an SVG shape for every part named 'base'
                // 	for &iref in road_model.children() {
                // 		let instance = dom.get_by_ref(iref).unwrap();
                // 		if instance.name == "Base" {
                // 			// we care about this, draw it.
                // 			// okay so how tf do we get instance parameters from this shit.
                // 			//println!("{:?}", instance.properties.keys());
                // 			let cf = match instance.properties.get("CFrame") {
                // 				Some(Variant::CFrame(v)) => v,
                // 				_ => panic!("Part does not have a cframe"),
                // 			};

                // 			let object_size = match instance.properties.get("Size") {
                // 				Some(Variant::Vector3(v)) => v,
                // 				_ => panic!("Part does not have a size"),
                // 			};

                // 			let object_position = Vector3::new(cf.position.x, cf.position.y, cf.position.z);//cf.position + Vector3::new(0f32, 0f32, 5000f32);
                // 			let object_orientation = cf.orientation;

                // 			let r_p = Vector3::new(object_position.x * self.scale + self.center_x, object_position.y * self.scale, object_position.z * self.scale + self.center_z);
                // 			let s = Vector3::new(object_size.x * self.scale, object_size.y * self.scale, object_size.z * self.scale);
                // 			draw_part_on_pixmap(&mut pixmap, r_p, s, object_orientation);
                // 		}
                // 	}
                // }

                //render::save(&canvas, "test.svg", SvgRenderer::new()).expect("Failed to save.");
                // let mut output_file = File::create("test2.svg")?;
                // output_file.write_all(&document.to_string().into_bytes());

                println!("Saving..");
                pixmap.save_png("output.png").unwrap();
                img.save("test.png").unwrap();

                println!("Success.");
                // for &referent in dom.root().children() {
                // 	let instance = dom.get_by_ref(referent).unwrap();
                // 	println!("- {}", instance.name);
                // }
            }
            OutputKind::Rbxlx => {
                //let dom = rbx_xml::from_reader_default(file_reader)?;
            }
        }
        // println!("Rendering map");
        // let pb = ProgressBar::new(200);
        // for _ in 0..200 {
        // 	pb.inc(1);
        // 	thread::sleep(Duration::from_millis(5));
        // }
        // pb.finish_with_message("done");
        Ok(())
    }
}

/// The different file types we support
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputKind {
    /// An XML place file
    Rbxlx,
    /// A binary place file
    Rbxl,
}

fn detect_file_kind(output: &Path) -> Option<OutputKind> {
    let extension = output.extension()?.to_str()?;

    match extension {
        "rbxlx" => Some(OutputKind::Rbxlx),
        "rbxl" => Some(OutputKind::Rbxl),
        _ => None,
    }
}

fn draw_part_to_imgbuf_experimental(
    imgbuf: &mut RgbImage,
    pos: Vector3,
    size: Vector3,
    rot: Matrix3,
    color: &[u8; 3],
) {
    let aa = math_lib::axis_angle_conversion::matrix3_to_axis_angle(rot);
    let t = aa.y;
    // 1 1
    let r1_x = size.x * 0.5f32 * t.cos() - size.z * 0.5f32 * t.sin() + pos.x;
    let r1_z = size.x * 0.5f32 * t.sin() + size.z * 0.5f32 * t.cos() + pos.z;
    // -1 1
    let r2_x = -size.x * 0.5f32 * t.cos() - size.z * 0.5f32 * t.sin() + pos.x;
    let r2_z = -size.x * 0.5f32 * t.sin() + size.z * 0.5f32 * t.cos() + pos.z;
    // 1 -1
    let r3_x = -size.x * 0.5f32 * t.cos() + size.z * 0.5f32 * t.sin() + pos.x;
    let r3_z = -size.x * 0.5f32 * t.sin() - size.z * 0.5f32 * t.cos() + pos.z;
    // -1 -1
    let r4_x = size.x * 0.5f32 * t.cos() + size.z * 0.5f32 * t.sin() + pos.x;
    let r4_z = size.x * 0.5f32 * t.sin() - size.z * 0.5f32 * t.cos() + pos.z;

    imgbuf.get_pixel_mut(r1_x as u32, r1_z as u32).0 = *color;
    imgbuf.get_pixel_mut(r2_x as u32, r2_z as u32).0 = *color;
    imgbuf.get_pixel_mut(r3_x as u32, r3_z as u32).0 = *color;
    imgbuf.get_pixel_mut(r4_x as u32, r4_z as u32).0 = *color;

    let filler_scale = 2u32;

    let img_size_x = (r3_x - r1_x).round() as u32 * filler_scale;
    let img_size_z = (r3_z - r1_z).round() as u32 * filler_scale;
    for x_s in 0u32..img_size_x {
        let x = lerp(r1_x, r3_x, (x_s / img_size_x) as f32) as u32;
        for z_s in 0u32..img_size_z {
            let z = lerp(r1_z, r3_z, (z_s / img_size_z) as f32) as u32;
            imgbuf.get_pixel_mut(x, z).0 = *color;
        }
    }
}

// actual rendering code
fn draw_part_on_pixmap(
    map: &mut Pixmap,
    pos: Vector3,
    size: Vector3,
    rot: Matrix3,
    color: &Vec<u8>,
) {
    let aa = math_lib::axis_angle_conversion::matrix3_to_axis_angle(rot);
    let t = aa.y;
    // 1 1
    let r1_x = size.x * 0.5f32 * t.cos() - size.z * 0.5f32 * t.sin() + pos.x;
    let r1_z = size.x * 0.5f32 * t.sin() + size.z * 0.5f32 * t.cos() + pos.z;
    // -1 1
    let r2_x = -size.x * 0.5f32 * t.cos() - size.z * 0.5f32 * t.sin() + pos.x;
    let r2_z = -size.x * 0.5f32 * t.sin() + size.z * 0.5f32 * t.cos() + pos.z;
    // 1 -1
    let r3_x = -size.x * 0.5f32 * t.cos() + size.z * 0.5f32 * t.sin() + pos.x;
    let r3_z = -size.x * 0.5f32 * t.sin() - size.z * 0.5f32 * t.cos() + pos.z;
    // -1 -1
    let r4_x = size.x * 0.5f32 * t.cos() + size.z * 0.5f32 * t.sin() + pos.x;
    let r4_z = size.x * 0.5f32 * t.sin() - size.z * 0.5f32 * t.cos() + pos.z;

    let path = {
        let mut pb = PathBuilder::new();
        pb.move_to(r1_x, r1_z);
        pb.line_to(r2_x, r2_z);
        pb.line_to(r3_x, r3_z);
        pb.line_to(r4_x, r4_z);
        //pb.line_to(r1_x, r1_z);
        pb.close();
        pb.finish().unwrap()
    };

    let mut paint = Paint::default();
    paint.set_color_rgba8(color[0], color[1], color[2], color[3]);
    paint.anti_alias = true;

    map.fill_path(
        &path,
        &paint,
        tiny_skia::FillRule::Winding,
        Transform::identity(),
        None,
    );
}

fn get_descendants(dom: &WeakDom, inst_ref: &Ref) -> anyhow::Result<Vec<Ref>> {
    let instance = dom
        .get_by_ref(*inst_ref)
        .expect("received invalid child in tree when recursing through descendants");

    let mut descendants: Vec<Ref> = Vec::new();
    let mut stack = VecDeque::from_iter(instance.children().into_iter());

    while let Some(current) = stack.pop_front() {
        descendants.push(*current);

        let current_instance = dom
            .get_by_ref(*current)
            .expect("received invalid child in tree when recursing through descendants");

        for child in current_instance.children().iter().rev() {
            stack.push_front(child);
        }
    }

    Ok(descendants)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
