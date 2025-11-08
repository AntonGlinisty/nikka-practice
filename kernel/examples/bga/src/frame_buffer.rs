use core::mem::{
    self,
    MaybeUninit,
};

use embedded_graphics_core::{
    Pixel,
    draw_target::DrawTarget,
    geometry::{
        Dimensions,
        OriginDimensions,
        Point,
        Size,
    },
    pixelcolor::PixelColor,
    primitives::rectangle::Rectangle,
};

use ku::{
    error::{
        Error,
        Result,
    },
    memory::{
        Block,
        Page,
        Phys,
        size,
    },
    time,
};

use kernel::{
    log::debug,
    memory::{
        BASE_ADDRESS_SPACE,
        FrameGuard,
        KERNEL_MMIO,
        KERNEL_RW,
    },
};

/// Управляет содержимым экрана через его
/// [видеобуфер](https://en.wikipedia.org/wiki/Framebuffer).
/// Поддерживает
/// [двойную буферизацию](https://en.wikipedia.org/wiki/Multiple_buffering).
pub struct FrameBuffer<Color: PixelColor + 'static> {
    /// Вторичный буфер (back buffer)
    /// [двойной буферизации](https://en.wikipedia.org/wiki/Multiple_buffering).
    /// Информация, записываемая в него, не отображается до тех пор, пока не вызван метод
    /// [`FrameBuffer::flush()`].
    back_buffer: &'static mut [Color],

    /// [Видеобуфер](https://en.wikipedia.org/wiki/Framebuffer)
    /// экрана.
    /// Является первичным буфером (front buffer)
    /// [двойной буферизации](https://en.wikipedia.org/wiki/Multiple_buffering).
    /// Информация, записываемая в него сразу отображается на экране.
    front_buffer: &'static mut [Color],

    /// Предыдущий вторичный буфер [`Framebuffer::back_buffer`], записанный в
    /// [`FrameBuffer::front_buffer`] предыдущим вызовом [`FrameBuffer::flush()`].
    prev_back_buffer: &'static mut [Color],

    /// Разрешение экрана.
    resolution: Size,

    /// Разность между индексами пикселей, соседних по вертикальной координате.
    /// В общем случае не равна `self.resolution.width`.
    stride: usize,
}

impl<Color: PixelColor> FrameBuffer<Color> {
    /// Создаёт видеобуфер по адресу `frame_buffer`
    /// с разрешением `resolution` и глубиной цвета, задаваемой `Color`.
    /// Очищает экран, заполняя его цветом `color`.
    pub fn new(
        frame_buffer: Phys,
        resolution: Size,
        color: Color,
    ) -> Result<Self> {
        let stride = size::from(resolution.width);
        let pixel_count = stride * size::from(resolution.height);
        let frame_buffer_size = pixel_count * mem::size_of::<Color>();

        let (front_buffer, back_buffer, prev_back_buffer) = Self::map_buffers(
            Block::new(frame_buffer, (frame_buffer + frame_buffer_size)?)?,
            pixel_count,
            color,
        )?;

        Ok(Self {
            back_buffer,
            front_buffer,
            prev_back_buffer,
            resolution,
            stride,
        })
    }

    /// Копирует содержимое вторичного буфера, накопившего изображение,
    /// в первичный.
    /// Это приводит к обновлению содержимого экрана.
    /// Не ждёт вертикальной синхронизации.
    pub fn flush(&mut self) {
        let timer = time::timer();

        for (pixel_index, pixel) in self.front_buffer.iter_mut().enumerate() {
            if self.back_buffer[pixel_index] != self.prev_back_buffer[pixel_index] {
                *pixel = self.back_buffer[pixel_index];
            }
        }

        mem::swap(&mut self.back_buffer, &mut self.prev_back_buffer);

        debug!(duration = %timer.elapsed(), "flush the frame buffer");
    }

    /// Записывает в заданный пиксель заданный цвет, если `pixel` находится внутри экрана.
    #[inline(always)]
    fn set_pixel(
        &mut self,
        pixel: Pixel<Color>,
    ) -> Result<()> {
        let Pixel::<Color>(point, color) = pixel;

        if self.bounding_box().contains(point) {
            self.back_buffer[self.index(point)?] = color;
        }

        Ok(())
    }

    /// Возвращает смещение пикселя с заданными координатами в
    /// [видеобуфере](https://en.wikipedia.org/wiki/Framebuffer)
    /// экрана.
    /// Не проверяет, что `point` находится внутри экрана.
    #[inline(always)]
    fn index(
        &self,
        point: Point,
    ) -> Result<usize> {
        Ok(size::from(u32::try_from(point.x)?) + size::from(u32::try_from(point.y)?) * self.stride)
    }

    /// Отображает `frame_buffer` в виртуальную память в качестве первичного буфера.
    /// И создаёт вторичные буферы такого же размера.
    fn map_buffers(
        frame_buffer: Block<Phys>,
        pixel_count: usize,
        color: Color,
    ) -> Result<(
        &'static mut [Color],
        &'static mut [Color],
        &'static mut [Color],
    )> {
        let back_buffer_flags = KERNEL_RW;
        let front_buffer_flags = KERNEL_MMIO;

        let mut address_space = BASE_ADDRESS_SPACE.lock();

        let front_buffer =
            address_space.allocate(Page::layout(frame_buffer.size())?, front_buffer_flags)?;

        for (frame, page) in frame_buffer.enclosing().into_iter().zip(front_buffer.into_iter()) {
            let frame = FrameGuard::reference(frame);
            unsafe {
                address_space.map_page_to_frame(page, frame, front_buffer_flags)?;
            }
        }

        let front_buffer = unsafe { front_buffer.try_into_mut_slice::<MaybeUninit<Color>>()? };
        let front_buffer = &mut front_buffer[.. pixel_count];
        let front_buffer = front_buffer.write_with(|_| color);

        let mut back_buffer = || {
            address_space.map_slice(pixel_count, back_buffer_flags, || color).map(|x| &mut x[.. pixel_count])
        };

        Ok((front_buffer, back_buffer()?, back_buffer()?))
    }
}

impl<Color: PixelColor> Drop for FrameBuffer<Color> {
    fn drop(&mut self) {
        let message = "failed to unmap the frame buffer";
        let mut address_space = BASE_ADDRESS_SPACE.lock();
        unsafe {
            address_space.unmap_slice(self.back_buffer).expect(message);
            address_space.unmap_slice(self.front_buffer).expect(message);
            address_space.unmap_slice(self.prev_back_buffer).expect(message);
        }
    }
}

impl<Color: PixelColor> OriginDimensions for FrameBuffer<Color> {
    fn size(&self) -> Size {
        self.resolution
    }
}

impl<Color: PixelColor> DrawTarget for FrameBuffer<Color> {
    type Color = Color;
    type Error = Error;

    fn draw_iter<I>(
        &mut self,
        pixels: I,
    ) -> Result<()>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels.into_iter() {
            self.set_pixel(pixel)?;
        }

        Ok(())
    }

    fn fill_contiguous<I>(
        &mut self,
        area: &Rectangle,
        colors: I,
    ) -> Result<()>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        let area = area.intersection(&self.bounding_box());
        let mut colors = colors.into_iter();
        let mut start = self.index(area.top_left)?;

        for _ in 0 .. area.size.height {
            let end = start + size::from(area.size.width);

            for (pixel, color) in self.back_buffer[start .. end].iter_mut().zip(&mut colors) {
                *pixel = color;
            }

            start += self.stride;
        }

        Ok(())
    }

    fn fill_solid(
        &mut self,
        area: &Rectangle,
        color: Self::Color,
    ) -> Result<()> {
        let area = area.intersection(&self.bounding_box());
        let mut start = self.index(area.top_left)?;

        for _ in 0 .. area.size.height {
            let end = start + size::from(area.size.width);

            self.back_buffer[start .. end].fill(color);

            start += self.stride;
        }

        Ok(())
    }

    fn clear(
        &mut self,
        color: Self::Color,
    ) -> Result<()> {
        self.back_buffer.fill(color);

        Ok(())
    }
}
