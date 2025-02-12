use std::collections::{HashMap};
use dxf::{Drawing};
use serde::{Serialize, Deserialize};
use std::io::Cursor;
use xml::EventReader;
use xml::reader::XmlEvent;
use wasm_bindgen::prelude::*;
use calamine::{ Data, DataType, Reader, Xlsx};
use serde_json;
use web_sys::console;

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
    pub color_id: i32,
    pub node_id: i32,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
struct RowData {
    id: i32,
    as1: Vec<f64>,
    as2: Vec<f64>,
    as3: Vec<f64>,
    as4: Vec<f64>,
}


#[derive(Serialize, Debug)]
pub struct EntityWithXlsx {
    pub entity_type: String,
    pub vertices: Vec<Vertex>,
    pub row: RowData,
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
                color_id: 0,
                node_id: 0,
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
                color_id: 0,
                node_id: 0,
            }),
            _ => None, // Ignore other types of entities
        }
    }).collect();
    // Convert the entities into a JSON string
    serde_json::to_string(&entities).expect("Failed to serialize to JSON")

}

#[wasm_bindgen]
pub fn sli_to_json(data: &str, tolerance: Option<f64>) -> String {
    let tolerance = tolerance.unwrap_or(0.005);
    let cursor = Cursor::new(data);
    let parser = EventReader::new(cursor);
    let mut points: Vec<Vertex> = Vec::new();
    let mut entities: Vec<SerializableEntity> = Vec::new();
    let mut planes: Vec<f64> = vec![];
    let mut node_id = 0;
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
                        node_id += 1;
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
                            color_id: 0,
                            node_id,
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
                            if is_in_same_plane(&entity.vertices, tolerance) {
                                if let Some(vertex) = entity.vertices.first() {
                                    let plane = (vertex.z / tolerance).round() * tolerance;
                                    if planes.contains(&plane){
                                        if let Some(color_id) = planes.iter().position(|&p| p == plane){
                                            entity.color_id = color_id as i32 + 1;
                                        }
                                    }else {
                                        planes.push(plane);
                                        entity.color_id = planes.len() as i32;
                                    }
                                }

                            }
                            entity.entity_type = match entity.vertices.len() as i32 {
                                2 => String::from("LINE"),
                                3 => String::from("3DFACE_TRIANGLE"),
                                _ => String::from("3DFACE"),
                            };
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
    let mut colored_entities: HashMap<i32, HashMap<&String, Vec<f64>>> = HashMap::new();
    for entity in entities.iter_mut() {
        let mut coords: Vec<f64> = entity.vertices.iter().flat_map(|v| vec![v.x, v.y, v.z]).collect();
        colored_entities.entry(entity.color_id).or_insert(HashMap::new()).entry(&entity.entity_type).or_insert(vec![]).append(&mut coords);
    }
    serde_json::to_string(&colored_entities).expect("Failed to serialize to JSON")
}

fn is_in_same_plane(points: &Vec<Vertex>, tolerance: f64) -> bool {
    if let Some(first) = points.first(){
        points.iter().all(|point| point.z - first.z <= tolerance)
    }else {
        false
    }
}

pub fn parse_xlsx_wasm(data: &[u8]) -> Vec<RowData> {
    match parse_xlsx_from_bytes(data) {
        Ok(parsed) => {
            console::log_1(&format!("✅ Успешный парсинг: {} записей", parsed.len()).into());
            parsed
        }
        Err(err) => {
            console::log_1(&format!("❌ Ошибка парсинга: {}", err).into());
            Vec::new()
        }
    }
}


fn parse_xlsx_from_bytes(data: &[u8]) -> Result<Vec<RowData>, String> {
    let mut workbook: Xlsx<_> = calamine::Reader::new(std::io::Cursor::new(data))
        .map_err(|e| format!("Ошибка загрузки: {}", e))?;

    // Получаем все имена вкладок
    let sheet_names = workbook.sheet_names().to_vec();

    let mut results = Vec::new();
    let mut row_data: Option<RowData> = None;
    let mut row_count = 0;

    // Проходим по всем вкладкам
    for sheet_name in sheet_names {
        let range = workbook.worksheet_range(&sheet_name).map_err(|e| e.to_string())?;

        println!("Обрабатываем вкладку: {}", sheet_name);

        for (index, row) in range.rows().enumerate() {
            let id_cell = row.get(0);

            match id_cell {
                // Если id есть, создаем новый объект
                Some(Data::Float(id)) => {
                    if let Some(prev) = row_data.take() {
                        results.push(prev);
                    }

                    // Читаем первую строку значений
                    row_data = Some(RowData {
                        id: *id as i32,
                        as1: vec![row.get(1).and_then(|d| d.get_float()).unwrap_or(0.0)],
                        as2: vec![row.get(2).and_then(|d| d.get_float()).unwrap_or(0.0)],
                        as3: vec![row.get(3).and_then(|d| d.get_float()).unwrap_or(0.0)],
                        as4: vec![row.get(4).and_then(|d| d.get_float()).unwrap_or(0.0)],
                    });

                    row_count = 1;
                    println!(
                        "✅ Новая строка ID {}: [{}, {}, {}, {}]",
                        id,
                        row_data.as_ref().unwrap().as1[0],
                        row_data.as_ref().unwrap().as2[0],
                        row_data.as_ref().unwrap().as3[0],
                        row_data.as_ref().unwrap().as4[0]
                    );
                }

                // Если id нет (значит, это вторая строка данных)
                None | Some(Data::Empty) | Some(Data::String(_)) => {
                    if let Some(ref mut row_data) = row_data {
                        if row_count == 1 {
                            row_data.as1.push(row.get(1).and_then(|d| d.get_float()).unwrap_or(0.0));
                            row_data.as2.push(row.get(2).and_then(|d| d.get_float()).unwrap_or(0.0));
                            row_data.as3.push(row.get(3).and_then(|d| d.get_float()).unwrap_or(0.0));
                            row_data.as4.push(row.get(4).and_then(|d| d.get_float()).unwrap_or(0.0));
                            row_count += 1;

                            println!("🔹 Добавлено второе значение: [{}, {}, {}, {}]", row_data.as1[1], row_data.as2[1], row_data.as3[1], row_data.as4[1]);
                        } else {
                            println!("⚠️ Пропущена третья строка подряд (ошибка в данных?)");
                        }
                    }

                }

                _ => {
                    println!("❌ Пропущена строка {} (непонятный формат данных)", index + 1);
                }
            }
        }
    }

    // Добавляем последний объект
    if let Some(last) = row_data {
        results.push(last);
    }

    Ok(results)
}


pub fn get_indexes(data: &str) -> Vec<SerializableEntity> {
    let cursor = Cursor::new(data);
    let parser = EventReader::new(cursor);
    let mut points: Vec<Vertex> = Vec::new();
    let mut entities: Vec<SerializableEntity> = Vec::new();
    let mut node_id = 0;
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
                        node_id += 1;
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
                            color_id: 0,
                            node_id,

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
                            entity.entity_type = match entity.vertices.len() as i32 {
                                2 => String::from("LINE"),
                                3 => String::from("3DFACE_TRIANGLE"),
                                _ => String::from("3DFACE"),
                            };
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
    entities
}

// pub fn get_entity_by_index(entities: Vec<SerializableEntity>, index: usize) -> Option<&SerializableEntity> {
//     entities.get(index - 1)
// }

#[wasm_bindgen]
pub fn convert_sli_xsl_to_json(sli_data: &str, data: &[u8]) -> String {
    let entities = get_indexes(sli_data);
    let xlsx = parse_xlsx_wasm(data);
    let mut entities_with_xlsx: Vec<EntityWithXlsx> = Vec::new();
    for row in xlsx {
        if let Some(entity) = entities.get(row.id as usize - 1) {
            entities_with_xlsx.push(EntityWithXlsx{
                entity_type: entity.entity_type.clone(),
                vertices: entity.vertices.clone(),
                row: row,
            })
        }

    }

    serde_json::to_string(&entities_with_xlsx).expect("Failed to serialize to JSON")
}