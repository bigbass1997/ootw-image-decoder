use std::ffi::OsStr;
use std::path::PathBuf;
use clap::{AppSettings, Arg, Command};
use image::{Rgb, RgbImage};
use image::imageops::{crop, flip_horizontal_in_place};

pub const TWIDTH: usize = 8;
pub const THEIGHT: usize = 8;

pub struct Tile {
    /// Upper-left x position of the tile within the `TiledImage`
    pub offx: usize,
    /// Upper-left y position of the tile within the `TiledImage`
    pub offy: usize,
    /// 2D grid of pixel data for this tile
    pub pixels: [[u32; THEIGHT]; TWIDTH],
}

pub struct TiledImage {
    pub tiles: Vec<Tile>,
}
impl TiledImage {
    pub fn new(pixels: Vec<u32>, width: usize, height: usize) -> Self {
        let mut tiles = vec![];
        
        let mut i = 0;
        for ty in 0..(height / THEIGHT) {
            for tx in 0..(width / TWIDTH) {
                let mut tile = Tile {
                    offx: tx * TWIDTH,
                    offy: ty * THEIGHT,
                    pixels: [[0u32; THEIGHT]; TWIDTH],
                };
                
                // Convert line of pixel data into a 8x8 grid
                for col in 0..TWIDTH {
                    for row in 0..THEIGHT {
                        tile.pixels[col][row] = pixels[i];
                        i += 1;
                    }
                }
                
                // Rotate each 2x2 grid sub-section 90 degrees counter-clockwise
                for col in (0..TWIDTH).step_by(2) {
                    for row in (0..THEIGHT).step_by(2) {
                        let ul = tile.pixels[col][row];
                        let ur = tile.pixels[col + 1][row];
                        let dr = tile.pixels[col + 1][row + 1];
                        let dl = tile.pixels[col][row + 1];
                        
                        tile.pixels[col][row] = ur;
                        tile.pixels[col + 1][row] = dr;
                        tile.pixels[col + 1][row + 1] = dl;
                        tile.pixels[col][row + 1] = ul;
                    }
                }
                
                // Swap upper-right 4x4 quadrant with lower-left 4x4 quadrant
                for col in (TWIDTH / 2)..TWIDTH {
                    for row in 0..(THEIGHT / 2) {
                        let other_col = col - (TWIDTH / 2);
                        let other_row = row + (THEIGHT / 2);
                        
                        let tmp = tile.pixels[col][row];
                        tile.pixels[col][row] = tile.pixels[other_col][other_row];
                        tile.pixels[other_col][other_row] = tmp;
                    }
                }
                
                // Rearrange middle four pixels/columns (2, 3, 4, and 5; 0-indexed) in each row
                for row in 0..THEIGHT {
                    let p2 = tile.pixels[2][row];
                    let p3 = tile.pixels[3][row];
                    let p4 = tile.pixels[4][row];
                    let p5 = tile.pixels[5][row];
                    
                    tile.pixels[2][row] = p4;
                    tile.pixels[3][row] = p5;
                    tile.pixels[4][row] = p2;
                    tile.pixels[5][row] = p3;
                }
                
                // Rearrange the upper 4 rows, and the lower 4 rows
                for col in 0..TWIDTH {
                    let row0 = tile.pixels[col][0];
                    let row1 = tile.pixels[col][1];
                    let row2 = tile.pixels[col][2];
                    let row3 = tile.pixels[col][3];
                    
                    tile.pixels[col][0] = row1;
                    tile.pixels[col][1] = row3;
                    tile.pixels[col][2] = row0;
                    tile.pixels[col][3] = row2;
                    
                    
                    let row4 = tile.pixels[col][4];
                    let row5 = tile.pixels[col][5];
                    let row6 = tile.pixels[col][6];
                    let row7 = tile.pixels[col][7];
                    
                    tile.pixels[col][4] = row5;
                    tile.pixels[col][5] = row7;
                    tile.pixels[col][6] = row4;
                    tile.pixels[col][7] = row6;
                }
                
                tiles.push(tile);
            }
        }
        
        Self {
            tiles,
        }
    }
}


fn main() {
    // Handle command-line arguments
    let matches = Command::new("Out of the World - Image Decoder")
        .version(clap::crate_version!())
        .arg(Arg::new("input")
            .takes_value(true)
            .required(true)
            .help("Path to input binary file. No wildcards, nor multiple files."))
        .next_line_help(true)
        .arg_required_else_help(true)
        .setting(AppSettings::DeriveDisplayOrder)
        .get_matches();
    let mut path = PathBuf::from(matches.value_of("input").unwrap());
    
    
    // Get binary data from file
    let mut data = std::fs::read(&path).unwrap();
    let footer = &data[(data.len()-12)..];
    let width = u16::from_le_bytes([footer[4], footer[5]]);
    let height = u16::from_le_bytes([footer[6], footer[7]]);
    let logical_width = u16::from_le_bytes([footer[8], footer[9]]);
    let logical_height = u16::from_le_bytes([footer[10], footer[11]]);
    
    data.resize(data.len() - 12, 0);
    
    // Parse pixels from file (RGB in reverse order)
    let mut pixels = vec![];
    for i in (0..data.len()).rev().step_by(3) {
        let r = data[i];
        let g = data[i - 1];
        let b = data[i - 2];
        let color = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
        
        pixels.push(color);
    }
    
    // Create/calculate tiled image from raw pixel data
    let tiled = TiledImage::new(pixels, width as usize, height as usize);
    
    // Move tiled pixel data into an image so that it can be saved as a file
    let mut img = RgbImage::new(width as u32, height as u32);
    for tile in &tiled.tiles {
        for col in tile.offx..(tile.offx + TWIDTH) {
            for row in tile.offy..(tile.offy + THEIGHT) {
                let pixel = tile.pixels[col % TWIDTH][row % THEIGHT];
                
                (*img.get_pixel_mut(col as u32, row as u32)) = to_rgb(pixel);
            }
        }
    }
    
    // Flip image horizontally
    flip_horizontal_in_place(&mut img);
    
    // Save full image
    let stem = path.file_stem().unwrap_or(OsStr::new("output")).to_string_lossy().to_string();
    path.set_file_name(format!("{}-full.png", stem));
    img.save(&path).unwrap();
    
    // Save "logical" image based on binary footer data
    path.set_file_name(format!("{}-logical.png", stem));
    let img = crop(&mut img, 0, 0, logical_width as u32, logical_height as u32).to_image();
    img.save(&path).unwrap();
}

/// Converts 0RGB to RGB component colors
fn to_rgb(color: u32) -> Rgb<u8> {
    Rgb([((color & 0x00FF0000) >> 16) as u8, ((color & 0x0000FF00) >> 8) as u8, (color & 0x000000FF) as u8])
}