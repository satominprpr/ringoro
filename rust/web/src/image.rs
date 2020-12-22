use image::{imageops::FilterType, io::Reader, DynamicImage, ImageFormat, ImageOutputFormat};
use std::io::Cursor;
use svgtypes::ViewBox;
use xmltree::Element;

use crate::utils::{
    result::Result,
    serde::{Deserialize, Serialize},
    simple_error,
};

const CAPACITY: usize = 100 * 1024;

pub enum ImageInput {
    Bin(Vec<u8>),
    Text(String),
}

#[derive(Serialize, Deserialize)]
pub struct BinImage {
    data: Vec<u8>,
    #[serde(rename = "type")]
    type_: String,
}

impl BinImage {
    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }
    pub fn type_(&self) -> &String {
        &self.type_
    }
}

#[derive(Serialize, Deserialize)]
pub struct TextImage {
    data: String,
    #[serde(rename = "type")]
    type_: String,
}

impl TextImage {
    pub fn data(&self) -> &String {
        &self.data
    }
    pub fn type_(&self) -> &String {
        &self.type_
    }
}

pub struct ResizedBin {
    orig: BinImage,
    large: BinImage,
    small: BinImage,
}

impl ResizedBin {
    pub fn orig(&self) -> &BinImage {
        &self.orig
    }
    pub fn large(&self) -> &BinImage {
        &self.small
    }
    pub fn small(&self) -> &BinImage {
        &self.large
    }
}

pub enum ResizedImage {
    Bin(ResizedBin),
    Text(TextImage),
}

impl ResizedImage {
    pub fn resize(
        type_: impl AsRef<str>,
        input: ImageInput,
        w: f64,
        h: f64,
        shink_rate: f64,
        delta: f64,
    ) -> Result<Self> {
        if !(0.0..1.0).contains(&shink_rate) {
            return Err(simple_error!("rate range error"));
        }
        if !(0.0..1.0).contains(&delta) {
            return Err(simple_error!("delta range error"));
        }
        let image = decode_image(type_, input)?;
        image.check_size(w, h, delta)?;
        image.resize(w, h, shink_rate)
    }
}

enum BinImageType {
    Jpeg(DynamicImage),
    Png(DynamicImage),
}

enum TextImageType {
    Svg(Element, f64, f64),
}

enum ImageType {
    Bin(BinImageType),
    Text(TextImageType),
}

impl ImageType {
    fn calc<FBin, FText, T>(&self, fbin: FBin, ftext: FText) -> T
    where
        FBin: FnOnce(&DynamicImage) -> T,
        FText: FnOnce(&Element, f64, f64) -> T,
    {
        match self {
            ImageType::Bin(BinImageType::Jpeg(img)) | ImageType::Bin(BinImageType::Png(img)) => {
                fbin(img)
            }
            ImageType::Text(TextImageType::Svg(elem, w, h)) => ftext(elem, *w, *h),
        }
    }

    fn size(&self) -> (f64, f64) {
        use image::GenericImageView;
        self.calc(
            |img| {
                let (w, h) = img.dimensions();
                (w as f64, h as f64)
            },
            |_, w, h| (w, h),
        )
    }

    fn check_size(&self, w: f64, h: f64, delta: f64) -> Result<()> {
        let (rw, rh) = self.size();
        let rate = rw / rh;
        let excepted_rate = w / h;
        if !(0.0..delta).contains(&(rate - excepted_rate).abs()) {
            return Err(simple_error!("delta range error"));
        }
        Ok(())
    }

    fn resize(&self, w: f64, h: f64, shink_rate: f64) -> Result<ResizedImage> {
        match self {
            ImageType::Bin(BinImageType::Jpeg(img)) => resize_img(
                &img,
                w,
                h,
                shink_rate,
                &String::from("imate/jpeg"),
                &ImageOutputFormat::Jpeg(85),
            ),
            ImageType::Bin(BinImageType::Png(img)) => resize_img(
                &img,
                w,
                h,
                shink_rate,
                &String::from("imate/png"),
                &ImageOutputFormat::Png,
            ),
            ImageType::Text(TextImageType::Svg(elem, _, _)) => resize_svg(&elem, w, h),
        }
    }
}

fn decode_image(type_: impl AsRef<str>, data: ImageInput) -> Result<ImageType> {
    match (type_.as_ref(), data) {
        ("image/png", ImageInput::Bin(bin)) => {
            Ok(ImageType::Bin(BinImageType::Png(decode_png(bin)?)))
        }
        ("image/jpeg", ImageInput::Bin(bin)) => {
            Ok(ImageType::Bin(BinImageType::Jpeg(decode_jpeg(bin)?)))
        }
        ("image/svg+xml", ImageInput::Text(text)) => {
            let (elem, x, y) = parse_svg(text)?;
            Ok(ImageType::Text(TextImageType::Svg(elem, x, y)))
        }
        _ => Err(simple_error!("invalid image type")),
    }
}

fn decode(buf: impl AsRef<[u8]>, f: ImageFormat) -> Result<DynamicImage> {
    let cur = Cursor::new(buf);
    Ok(Reader::with_format(cur, f).decode()?)
}

fn decode_jpeg(buf: impl AsRef<[u8]>) -> Result<DynamicImage> {
    decode(buf, ImageFormat::Jpeg)
}

fn decode_png(buf: impl AsRef<[u8]>) -> Result<DynamicImage> {
    decode(buf, ImageFormat::Png)
}

fn resize_img(
    orig_img: &DynamicImage,
    w: f64,
    h: f64,
    shink_rate: f64,
    type_: &str,
    f: &ImageOutputFormat,
) -> Result<ResizedImage> {
    let large_img = orig_img.resize_exact(w as u32, h as u32, FilterType::CatmullRom);
    let small_img = orig_img.resize_exact(
        (w * shink_rate) as u32,
        (h * shink_rate) as u32,
        FilterType::CatmullRom,
    );
    Ok(ResizedImage::Bin(ResizedBin {
        small: save_img(&small_img, type_.to_string(), f.clone())?,
        large: save_img(&large_img, type_.to_string(), f.clone())?,
        orig: save_img(orig_img, type_.to_string(), f.clone())?,
    }))
}

fn save_img(img: &DynamicImage, type_: String, f: ImageOutputFormat) -> Result<BinImage> {
    let mut data = Vec::<u8>::with_capacity(CAPACITY);
    img.write_to(&mut data, f)?;
    Ok(BinImage { data, type_ })
}

fn parse_svg(text: String) -> Result<(Element, f64, f64)> {
    let elem = Element::parse(text.as_bytes())?;
    is_svg(&elem)?;
    let (x, y) = svg_size(&elem)?;
    Ok((elem, x, y))
}

fn is_svg(elem: &Element) -> Result<()> {
    match (&elem.namespace, &*elem.name) {
        (Some(ns), "svg") if &*ns == "http://www.w3.org/2000/svg" => Ok(()),
        _ => Err(simple_error!("This is not SVG.")),
    }
}

fn svg_size(elem: &Element) -> Result<(f64, f64)> {
    use std::str::FromStr;
    let attr = elem
        .attributes
        .get("viewBox")
        .ok_or_else(|| simple_error!("viewBox is not found."))?;
    let viewbox = ViewBox::from_str(&attr)?;
    Ok((viewbox.w, viewbox.h))
}

fn resize_svg(orig: &Element, w: f64, h: f64) -> Result<ResizedImage> {
    let mut out = orig.clone();
    out.attributes
        .insert(String::from("width"), format!("{}px", w));
    out.attributes
        .insert(String::from("height"), format!("{}px", h));
    out.attributes
        .insert(String::from("preserveAspectRatio"), String::from("none"));
    let mut buf = Vec::with_capacity(CAPACITY);
    orig.write(&mut buf)?;
    let data = String::from_utf8(buf)?;
    Ok(ResizedImage::Text(TextImage {
        data,
        type_: String::from("image/svg+xml"),
    }))
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;

    fn load_image(filename: &str) -> Vec<u8> {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("test_files");
        d.push(filename);
        std::fs::read(d).unwrap()
    }

    #[test]
    fn test_png() {
        let file = load_image("sample.png");
        let input = ImageInput::Bin(file);
        let _result = ResizedImage::resize("image/png", input, 854f64, 560f64, 0.5, 0.03).unwrap();
    }

    #[test]
    fn test_jpeg() {
        let file = load_image("sample.jpg");
        let input = ImageInput::Bin(file);
        let _result = ResizedImage::resize("image/jpeg", input, 854f64, 560f64, 0.5, 0.03).unwrap();
    }

    #[test]
    fn test_svg() {
        let file = load_image("sample.svg");
        let input = ImageInput::Text(String::from_utf8(file).unwrap());
        let _result =
            ResizedImage::resize("image/svg+xml", input, 721f64, 545f64, 0.5, 0.03).unwrap();
    }
}
