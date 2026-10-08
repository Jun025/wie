use alloc::{borrow::Cow, boxed::Box, format, vec, vec::Vec};
use core::marker::PhantomData;

use bytemuck::{Zeroable, cast_vec};

use jvm::{
    Array, ArrayRawBufferMut, ClassInstanceRef, Jvm, Result as JvmResult,
    runtime::{JavaIoInputStream, JavaLangClassLoader, JavaLangString},
};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_backend::canvas::{
    ArgbPixel, Canvas, Color, Image as BackendImage, ImageBuffer, ImageBufferCanvas, PixelType, Rgb332Pixel, Rgb565Pixel, decode_image,
};
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::lcdui::{Graphics, graphics::transformed_region};

// class javax.microedition.lcdui.Image
pub struct Image;

impl Image {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/Image",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(II[BI)V", Self::init, MethodAccessFlags::empty()),
                // SKT titles call `new Image()` (the platform they were built against had one): 5 of
                // them died on NoSuchMethodError at boot in the 2026-09-27 census.
                JavaMethodProto::new("<init>", "()V", Self::init_empty, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getWidth", "()I", Self::get_width, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "getGraphics",
                    "()Ljavax/microedition/lcdui/Graphics;",
                    Self::get_graphics,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "createImage",
                    "(II)Ljavax/microedition/lcdui/Image;",
                    Self::create_image,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "createImage",
                    "([BII)Ljavax/microedition/lcdui/Image;",
                    Self::create_image_from_data,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "createImage",
                    "(Ljava/lang/String;)Ljavax/microedition/lcdui/Image;",
                    Self::create_image_from_name,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "createImage",
                    "(Ljavax/microedition/lcdui/Image;)Ljavax/microedition/lcdui/Image;",
                    Self::create_image_from_image,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                // MIDP 2.0 — the rest of the creation surface. A general MIDP game reaches for these where the
                // carrier titles never did (2026-10-08 open-source corpus round).
                JavaMethodProto::new(
                    "createImage",
                    "(Ljava/io/InputStream;)Ljavax/microedition/lcdui/Image;",
                    Self::create_image_from_stream,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "createImage",
                    "(Ljavax/microedition/lcdui/Image;IIIII)Ljavax/microedition/lcdui/Image;",
                    Self::create_image_from_region,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "createRGBImage",
                    "([IIIZ)Ljavax/microedition/lcdui/Image;",
                    Self::create_rgb_image,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new("getRGB", "([IIIIIII)V", Self::get_rgb, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("isMutable", "()Z", Self::is_mutable, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("w", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("h", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("imgData", "[B", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("bpl", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("mutable", "Z", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        width: i32,
        height: i32,
        img_data: ClassInstanceRef<Array<i8>>,
        bpl: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Image::<init>({this:?}, {width}, {height}, {img_data:?}, {bpl})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        jvm.put_field(&mut this, "w", "I", width).await?;
        jvm.put_field(&mut this, "h", "I", height).await?;
        jvm.put_field(&mut this, "imgData", "[B", img_data).await?;
        jvm.put_field(&mut this, "bpl", "I", bpl).await?;

        Ok(())
    }

    async fn init_empty(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        // 1x1, not 0x0: every reader derives bytes-per-pixel as bpl / width.
        let img_data = jvm.instantiate_array("B", 4).await?;
        Self::init(jvm, context, this, 1, 1, img_data.into(), 4).await
    }

    async fn create_image(jvm: &Jvm, _: &mut WieJvmContext, width: i32, height: i32) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("javax.microedition.lcdui.Image::createImage({width}, {height})");

        let bytes_per_pixel = 4;

        let mut image = Self::create_image_instance(
            jvm,
            width as _,
            height as _,
            &vec![0; (width * height * bytes_per_pixel) as usize],
            bytes_per_pixel as _,
        )
        .await?;
        jvm.put_field(&mut image, "mutable", "Z", true).await?;

        Ok(image)
    }

    async fn create_image_from_name(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("javax.microedition.lcdui.Image::createImage({name:?})");

        let name = JavaLangString::to_rust_string(jvm, &name).await?;

        let class_loader = JavaLangClassLoader::get_system_class_loader(jvm).await?;
        let stream = JavaLangClassLoader::get_resource_as_stream(jvm, &class_loader, &name).await?;
        let Some(stream) = stream else {
            let exception = jvm
                .exception("java/io/FileNotFoundException", &format!("Resource not found: {name}"))
                .await;
            return Err(exception);
        };

        let image_data = JavaIoInputStream::read_until_end(jvm, &stream).await?;
        let image_data_len = image_data.len() as i32;

        let mut image_array = jvm.instantiate_array("B", image_data_len as _).await?;
        jvm.array_raw_buffer_mut(&mut image_array).await?.write(0, &image_data)?;

        jvm.invoke_static(
            "javax/microedition/lcdui/Image",
            "createImage",
            "([BII)Ljavax/microedition/lcdui/Image;",
            (image_array, 0, image_data_len),
        )
        .await
    }

    async fn create_image_from_data(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        data: ClassInstanceRef<Array<i8>>,
        image_offset: i32,
        image_length: i32,
    ) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("javax.microedition.lcdui.Image::createImage({data:?}, {image_offset}, {image_length})");

        let mut image_data = vec![0; image_length as usize];
        jvm.array_raw_buffer(&data).await?.read(image_offset as _, &mut image_data)?;

        let image = {
            let result = decode_image(&cast_vec(image_data));
            if let Ok(image) = result {
                image
            } else {
                tracing::error!("Failed to decode image: {:?}", result.err());
                let exception = jvm.exception("java/lang/IllegalArgumentException", "Failed to decode image").await;

                return Err(exception);
            }
        };

        Self::create_image_instance(jvm, image.width(), image.height(), &image.raw(), image.bytes_per_pixel()).await
    }

    async fn create_image_from_image(jvm: &Jvm, _: &mut WieJvmContext, image: ClassInstanceRef<Image>) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("javax.microedition.lcdui.Image::createImage({image:?})");

        let src_image = Image::image(jvm, &image).await?;

        Self::create_image_instance(jvm, src_image.width(), src_image.height(), &src_image.raw(), src_image.bytes_per_pixel()).await
    }

    async fn create_image_from_stream(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        stream: ClassInstanceRef<JavaIoInputStream>,
    ) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("javax.microedition.lcdui.Image::createImage({stream:?})");

        if stream.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "stream is null").await);
        }
        let data = JavaIoInputStream::read_until_end(jvm, &stream).await?;
        let mut array = jvm.instantiate_array("B", data.len()).await?;
        jvm.array_raw_buffer_mut(&mut array).await?.write(0, &data)?;

        jvm.invoke_static(
            "javax/microedition/lcdui/Image",
            "createImage",
            "([BII)Ljavax/microedition/lcdui/Image;",
            (array, 0, data.len() as i32),
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn create_image_from_region(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        image: ClassInstanceRef<Image>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        transform: i32,
    ) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("javax.microedition.lcdui.Image::createImage({image:?}, {x}, {y}, {width}, {height}, {transform})");

        if image.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "image is null").await);
        }
        let src = Image::image(jvm, &image).await?;
        let inside = x >= 0 && y >= 0 && width > 0 && height > 0 && (x + width) as u32 <= src.width() && (y + height) as u32 <= src.height();
        if !inside || !(0..=7).contains(&transform) {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "invalid region or transform").await);
        }
        let region = transformed_region(&*src, x, y, width, height, transform);

        Self::create_image_instance(jvm, region.width(), region.height(), &region.raw(), 4).await
    }

    async fn create_rgb_image(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        rgb: ClassInstanceRef<Array<i32>>,
        width: i32,
        height: i32,
        process_alpha: bool,
    ) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("javax.microedition.lcdui.Image::createRGBImage({rgb:?}, {width}, {height}, {process_alpha})");

        if rgb.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "rgb is null").await);
        }
        let count = (width as i64) * (height as i64);
        if width <= 0 || height <= 0 || count > jvm.array_length(&rgb).await? as i64 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "invalid size").await);
        }
        let pixels: Vec<i32> = jvm.load_array(&rgb, 0, count as _).await?;
        let opaque = if process_alpha { 0 } else { 0xff00_0000u32 };
        let raw: Vec<u32> = pixels.into_iter().map(|p| p as u32 | opaque).collect();

        Self::create_image_instance(jvm, width as _, height as _, bytemuck::cast_slice(&raw), 4).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn get_rgb(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        mut rgb: ClassInstanceRef<Array<i32>>,
        offset: i32,
        scan_length: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Image::getRGB({this:?}, {offset}, {scan_length}, {x}, {y}, {width}, {height})");

        if rgb.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "rgb is null").await);
        }
        let image = Image::image(jvm, &this).await?;
        let inside = x >= 0 && y >= 0 && width >= 0 && height >= 0 && (x + width) as u32 <= image.width() && (y + height) as u32 <= image.height();
        if !inside {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "region outside the image").await);
        }
        let length = jvm.array_length(&rgb).await? as i64;
        for row in 0..height {
            let start = offset as i64 + row as i64 * scan_length as i64;
            if start < 0 || start + width as i64 > length {
                return Err(jvm.exception("java/lang/ArrayIndexOutOfBoundsException", "rgbData too small").await);
            }
            let line: Vec<i32> = (0..width)
                .map(|col| ArgbPixel::from_color(image.get_pixel(x + col, y + row)) as i32)
                .collect();
            jvm.store_array(&mut rgb, start as _, line).await?;
        }

        Ok(())
    }

    // Only createImage(int, int) makes a mutable image; every other factory decodes or copies into an immutable one.
    async fn is_mutable(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        jvm.get_field(&this, "mutable", "Z").await
    }

    async fn get_graphics(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Graphics>> {
        tracing::debug!("javax.microedition.lcdui.Image::getGraphics({this:?})");

        let instance = jvm
            .new_class(
                "javax/microedition/lcdui/Graphics",
                "(Ljavax/microedition/lcdui/Image;)V",
                (this.clone(),),
            )
            .await?;

        Ok(instance.into())
    }

    async fn get_width(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Image::getWidth({this:?})");

        jvm.get_field(&this, "w", "I").await
    }

    async fn get_height(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Image::getHeight({this:?})");

        jvm.get_field(&this, "h", "I").await
    }

    pub async fn image(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<Box<dyn BackendImage>> {
        let width: i32 = jvm.get_field(this, "w", "I").await?;
        let bpl: i32 = jvm.get_field(this, "bpl", "I").await?;

        let bytes_per_pixel = bpl / width;

        Ok(match bytes_per_pixel {
            1 => Box::new(JavaImageBuffer::<Rgb332Pixel>::new(jvm, this).await?) as _,
            2 => Box::new(JavaImageBuffer::<Rgb565Pixel>::new(jvm, this).await?) as _,
            4 => Box::new(JavaImageBuffer::<ArgbPixel>::new(jvm, this).await?) as _,
            _ => unimplemented!("Unsupported pixel format: {bytes_per_pixel}"),
        })
    }

    pub async fn image_buffer(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<Box<dyn ImageBuffer>> {
        let width: i32 = jvm.get_field(this, "w", "I").await?;
        let bpl: i32 = jvm.get_field(this, "bpl", "I").await?;

        let bytes_per_pixel = bpl / width;

        Ok(match bytes_per_pixel {
            1 => Box::new(JavaImageBuffer::<Rgb332Pixel>::new(jvm, this).await?) as _,
            2 => Box::new(JavaImageBuffer::<Rgb565Pixel>::new(jvm, this).await?) as _,
            4 => Box::new(JavaImageBuffer::<ArgbPixel>::new(jvm, this).await?) as _,
            _ => unimplemented!("Unsupported pixel format: {bytes_per_pixel}"),
        })
    }

    pub async fn canvas(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<Box<dyn Canvas>> {
        let width: i32 = jvm.get_field(this, "w", "I").await?;
        let bpl: i32 = jvm.get_field(this, "bpl", "I").await?;

        let bytes_per_pixel = bpl / width;

        Ok(match bytes_per_pixel {
            1 => Box::new(ImageBufferCanvas::new(JavaImageBuffer::<Rgb332Pixel>::new(jvm, this).await?)) as _,
            2 => Box::new(ImageBufferCanvas::new(JavaImageBuffer::<Rgb565Pixel>::new(jvm, this).await?)) as _,
            4 => Box::new(ImageBufferCanvas::new(JavaImageBuffer::<ArgbPixel>::new(jvm, this).await?)) as _,
            _ => unimplemented!("Unsupported pixel format: {bytes_per_pixel}"),
        })
    }

    async fn create_image_instance(jvm: &Jvm, width: u32, height: u32, data: &[u8], bytes_per_pixel: u32) -> JvmResult<ClassInstanceRef<Image>> {
        let mut data_array = jvm.instantiate_array("B", data.len() as _).await?;
        jvm.array_raw_buffer_mut(&mut data_array).await?.write(0, data)?;

        Ok(jvm
            .new_class(
                "javax/microedition/lcdui/Image",
                "(II[BI)V",
                (width as i32, height as i32, data_array, (width * bytes_per_pixel) as i32),
            )
            .await?
            .into())
    }
}

struct JavaImageBuffer<T>
where
    T: PixelType,
{
    width: i32,
    height: i32,
    raw_buffer: Box<dyn ArrayRawBufferMut>,
    _phantom: PhantomData<T>,
}

impl<T> JavaImageBuffer<T>
where
    T: PixelType,
{
    pub async fn new(jvm: &Jvm, this: &ClassInstanceRef<Image>) -> JvmResult<Self> {
        let mut java_img_data = jvm.get_field(this, "imgData", "[B").await?;
        let raw_buffer = jvm.array_raw_buffer_mut(&mut java_img_data).await?;

        let width: i32 = jvm.get_field(this, "w", "I").await?;
        let height: i32 = jvm.get_field(this, "h", "I").await?;

        Ok(Self {
            width,
            height,
            raw_buffer,
            _phantom: PhantomData,
        })
    }
}

impl<T> BackendImage for JavaImageBuffer<T>
where
    T: PixelType,
{
    fn width(&self) -> u32 {
        self.width as _
    }

    fn height(&self) -> u32 {
        self.height as _
    }

    fn bytes_per_pixel(&self) -> u32 {
        size_of::<T::DataType>() as _
    }

    fn get_pixel(&self, x: i32, y: i32) -> Color {
        let offset = (((y as u32) * self.width() + (x as u32)) * self.bytes_per_pixel()) as usize;

        let mut raw = T::DataType::zeroed();
        self.raw_buffer.read(offset, bytemuck::bytes_of_mut(&mut raw)).unwrap();

        T::to_color(raw)
    }

    fn raw(&self) -> Cow<'_, [u8]> {
        let size = self.width() * self.height() * self.bytes_per_pixel();
        let mut buffer = vec![0; size as usize];
        self.raw_buffer.read(0, &mut buffer).unwrap();

        Cow::Owned(buffer)
    }

    fn colors(&self) -> Vec<Color> {
        let buffer = self.raw();

        let data: Vec<T::DataType> = bytemuck::pod_collect_to_vec(&buffer);

        data.into_iter().map(T::to_color).collect()
    }
}

impl<T> ImageBuffer for JavaImageBuffer<T>
where
    T: PixelType,
{
    fn put_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x > self.width || y > self.height || x < 0 || y < 0 {
            return;
        }

        let offset = (((y as u32) * self.width() + (x as u32)) * self.bytes_per_pixel()) as usize;

        let raw = T::from_color(color);
        let raw_bytes = bytemuck::bytes_of(&raw);

        self.raw_buffer.write(offset as _, raw_bytes).unwrap();
    }

    fn put_pixels(&mut self, x: i32, y: i32, _width: u32, colors: &[Color]) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return;
        }
        // a span past the row end would spill into the next row, or past the last one into a panic
        let colors = &colors[..colors.len().min((self.width - x) as usize)];

        let offset = (((y as u32) * self.width() + (x as u32)) * self.bytes_per_pixel()) as usize;

        let raw_bytes = colors
            .iter()
            .flat_map(|color| bytemuck::bytes_of(&T::from_color(*color)).to_vec())
            .collect::<Vec<_>>();

        self.raw_buffer.write(offset as _, &raw_bytes).unwrap();
    }

    fn xor_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return;
        }

        let offset = (((y as u32) * self.width() + (x as u32)) * self.bytes_per_pixel()) as usize;

        let mut raw = T::DataType::zeroed();
        self.raw_buffer.read(offset, bytemuck::bytes_of_mut(&mut raw)).unwrap();

        let raw = T::xor_color(raw, color);
        self.raw_buffer.write(offset as _, bytemuck::bytes_of(&raw)).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn check_pixel_buffer<T: PixelType>(jvm: &Jvm, pixels: [T::DataType; 2]) -> JvmResult<()> {
        let size = size_of::<T::DataType>();
        let image = Image::create_image_instance(jvm, 2, 1, bytemuck::cast_slice(&pixels), size as u32).await?;
        let mut buffer = JavaImageBuffer::<T>::new(jvm, &image).await?;
        let actual = buffer.get_pixel(1, 0);
        let expected = T::to_color(pixels[1]);
        assert_eq!((actual.a, actual.r, actual.g, actual.b), (expected.a, expected.r, expected.g, expected.b));

        let color = Color {
            a: 255,
            r: 127,
            g: 63,
            b: 31,
        };
        buffer.xor_pixel(1, 0, color);
        let expected = [pixels[0], T::xor_color(pixels[1], color)];
        assert_eq!(&*buffer.raw(), bytemuck::cast_slice(&expected));
        Ok(())
    }

    // SKT titles construct `new Image()`; readers divide bpl by width, so it must not be 0x0.
    #[test]
    fn no_arg_image_is_one_pixel() -> wie_util::Result<()> {
        test_utils::run_jvm_test(Box::new([crate::get_protos().into()]), |jvm| async move {
            let image: ClassInstanceRef<Image> = jvm.new_class("javax/microedition/lcdui/Image", "()V", ()).await?.into();
            let width: i32 = jvm
                .invoke_virtual(&image, "javax/microedition/lcdui/Image", "getWidth", "()I", ())
                .await?;
            let height: i32 = jvm
                .invoke_virtual(&image, "javax/microedition/lcdui/Image", "getHeight", "()I", ())
                .await?;
            assert_eq!((width, height), (1, 1));
            let mut buffer = JavaImageBuffer::<ArgbPixel>::new(&jvm, &image).await?;
            let _ = buffer.get_pixel(0, 0);
            Ok(())
        })
    }

    #[test]
    fn java_pixel_buffers_preserve_native_formats() -> wie_util::Result<()> {
        test_utils::run_jvm_test(Box::new([crate::get_protos().into()]), |jvm| async move {
            check_pixel_buffer::<Rgb332Pixel>(&jvm, [0x13, 0xe7]).await?;
            check_pixel_buffer::<Rgb565Pixel>(&jvm, [0x1234, 0xabcd]).await?;
            check_pixel_buffer::<ArgbPixel>(&jvm, [0x10203040, 0x80abcdef]).await?;
            Ok(())
        })
    }

    // createRGBImage → createImage(region, TRANS_MIRROR) → getRGB: a 2x1 strip comes back reversed, alpha kept.
    #[test]
    fn rgb_images_round_trip_through_a_mirrored_region() -> wie_util::Result<()> {
        test_utils::run_jvm_test(Box::new([crate::get_protos().into()]), |jvm| async move {
            let mut rgb = jvm.instantiate_array("I", 2).await?;
            jvm.store_array(&mut rgb, 0, vec![0x80ff0000u32 as i32, 0xff00ff00u32 as i32]).await?;
            let image: ClassInstanceRef<Image> = jvm
                .invoke_static(
                    "javax/microedition/lcdui/Image",
                    "createRGBImage",
                    "([IIIZ)Ljavax/microedition/lcdui/Image;",
                    (rgb, 2, 1, true),
                )
                .await?;
            let mirrored: ClassInstanceRef<Image> = jvm
                .invoke_static(
                    "javax/microedition/lcdui/Image",
                    "createImage",
                    "(Ljavax/microedition/lcdui/Image;IIIII)Ljavax/microedition/lcdui/Image;",
                    (image.clone(), 0, 0, 2, 1, 2),
                )
                .await?;

            let out = jvm.instantiate_array("I", 2).await?;
            let _: () = jvm
                .invoke_virtual(
                    &mirrored,
                    "javax/microedition/lcdui/Image",
                    "getRGB",
                    "([IIIIIII)V",
                    (out.clone(), 0, 2, 0, 0, 2, 1),
                )
                .await?;
            let out: Vec<i32> = jvm.load_array(&out, 0, 2).await?;
            assert_eq!(out, [0xff00ff00u32 as i32, 0x80ff0000u32 as i32]);

            let mutable: bool = jvm
                .invoke_virtual(&mirrored, "javax/microedition/lcdui/Image", "isMutable", "()Z", ())
                .await?;
            let blank: ClassInstanceRef<Image> = jvm
                .invoke_static(
                    "javax/microedition/lcdui/Image",
                    "createImage",
                    "(II)Ljavax/microedition/lcdui/Image;",
                    (1, 1),
                )
                .await?;
            let blank_mutable: bool = jvm
                .invoke_virtual(&blank, "javax/microedition/lcdui/Image", "isMutable", "()Z", ())
                .await?;
            assert_eq!((mutable, blank_mutable), (false, true));
            Ok(())
        })
    }
}
