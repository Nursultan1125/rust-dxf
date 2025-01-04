use dxf::Drawing;
use dxf::entities::{Entity, Line, Face3D};  // Assuming you are using the dxf crate to get these types
use serde::Serialize;
use wasm_bindgen::prelude::*;
use std::io::Cursor;

// Define a struct for the vertex (a 3D point)
#[derive(Serialize)]
pub struct Vertex {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

// Define the structure for serializing the entity data
#[derive(Serialize)]
pub struct SerializableEntity {
    pub entity_type: String,
    pub vertices: Vec<Vertex>,
    pub handle: String,
    pub layer: String,
}

#[wasm_bindgen]
pub fn dxf_to_json(dxf_data: &str) -> String {
    let mut cursor = Cursor::new(dxf_data);
    let drawing = Drawing::load(&mut cursor).expect("Failed to parse DXF data");

    // Collecting all entities
    let entities: Vec<SerializableEntity> = drawing.entities().filter_map(|entity| {
        match entity.specific {
            // Handle LINE entities
            dxf::entities::EntityType::Line(ref line) => Some(SerializableEntity {
                entity_type: "LINE".to_string(),
                vertices: vec![
                    Vertex {
                        x: line.p1.x,
                        y: line.p1.y,
                        z: line.p1.z,
                    },
                    Vertex {
                        x: line.p2.x,
                        y: line.p2.y,
                        z: line.p2.z,
                    },
                ],
                handle: entity.common.handle.clone(),
                layer: entity.common.layer.clone(),
            }),
            // Handle 3DFACE entities
            dxf::entities::EntityType::Face3D(ref face3d) => Some(SerializableEntity {
                entity_type: "3DFACE".to_string(),
                vertices: vec![
                    Vertex {
                        x: face3d.first_corner.x,
                        y: face3d.first_corner.y,
                        z: face3d.first_corner.z,
                    },
                    Vertex {
                        x: face3d.second_corner.x,
                        y: face3d.second_corner.y,
                        z: face3d.second_corner.z,
                    },
                    Vertex {
                        x: face3d.third_corner.x,
                        y: face3d.third_corner.y,
                        z: face3d.third_corner.z,
                    },
                    Vertex {
                        x: face3d.fourth_corner.x,
                        y: face3d.fourth_corner.y,
                        z: face3d.fourth_corner.z,
                    },
                ],
                handle: entity.common.handle.clone(),
                layer: entity.common.layer.clone(),
            }),
            _ => None, // Ignore other types of entities
        }
    }).collect();

    // Convert the entities into a JSON string
    serde_json::to_string(&entities).expect("Failed to serialize to JSON")
}
