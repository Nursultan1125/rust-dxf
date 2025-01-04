use dxf::{Drawing};
use serde::Serialize;
use wasm_bindgen::prelude::*;
use std::io::Cursor;
use xml::EventReader;
use xml::reader::XmlEvent;

#[derive(Serialize, Debug, Clone)]
pub struct Vertex {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Serialize, Debug)]
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
                handle: entity.common.handle.clone().as_string(),
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
                handle: entity.common.handle.clone().as_string(),
                layer: entity.common.layer.clone(),
            }),
            _ => None, // Ignore other types of entities
        }
    }).collect();
    // Convert the entities into a JSON string
    serde_json::to_string(&entities).expect("Failed to serialize to JSON")

}

#[wasm_bindgen]
pub fn sli_to_json(data: &str) -> String {
    let cursor = Cursor::new(data);
    let parser = EventReader::new(cursor);
    let mut points: Vec<Vertex> = Vec::new();
    let mut entities: Vec<SerializableEntity> = Vec::new();
    for e in parser {
        match e {
            Ok(XmlEvent::StartElement { name, attributes,  ..}) => {
                match name.local_name.as_str() {
                    "NodeCoords" => {
                        let vertices: Vertex = Vertex{
                            x: attributes.iter().find(|attr| attr.name.local_name == "NdX").unwrap().value.parse::<f64>().unwrap(),
                            y: attributes.iter().find(|attr| attr.name.local_name == "NdY").unwrap().value.parse::<f64>().unwrap(),
                            z: attributes.iter().find(|attr| attr.name.local_name == "NdZ").unwrap().value.parse::<f64>().unwrap(),
                        };
                        points.push(vertices)
                    },
                    "Element" => {
                        let entity_type = match attributes.iter().find(|attr| attr.name.local_name == "Type").unwrap().value.as_str() {
                            "1" => String::from("LINE"),
                            "2" => String::from("3DFACE"),
                            _ => String::from("UNKNOWN"),
                        };
                        let entity = SerializableEntity{
                            entity_type,
                            vertices: vec![],
                            handle: "".to_string(),
                            layer: "".to_string(),
                        };
                        entities.push(
                            entity
                        )
                    },
                    "Nodes" => {
                        let node_indexes = attributes.iter().map(|attr| attr.value.parse::<usize>().unwrap()).collect::<Vec<usize>>();
                        if let Some(entity) = entities.iter_mut().last() {
                            for index in node_indexes {
                                if let Some(vertex) = points.get(index - 1) {
                                    entity.vertices.push(vertex.clone());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(XmlEvent::EndElement {..}) => {}
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            }
            _ => {}
        }
    }
    serde_json::to_string(&entities).expect("Failed to serialize to JSON")
}
