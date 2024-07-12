use std::fmt::Debug;
use serde::{Serialize, Deserialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "t", content = "d")]
pub enum Message {
    #[serde(rename = "d")]
    DimensionsUpdate(DimensionsData),

    #[serde(rename = "tu")]
    TouchUpdate(Vec<Touch>),

    #[serde(rename = "te")]
    TouchEnd(Vec<i32>)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DimensionsData {
    pub width: i32,
    pub height: i32,
    pub resolution: i32
}

#[derive(Copy, Clone, Debug, Default, Deserialize, Serialize)]
pub struct Touch {
    pub id: i32,
    pub x: f64,
    pub y: f64,
    #[serde(rename = "rx")]
    pub radius_x: f32,
    #[serde(rename = "ry")]
    pub radius_y: f32,
    #[serde(rename = "ra")]
    pub rotation_angle: f32,
    #[serde(rename = "p")]
    pub pressure: f32
}

