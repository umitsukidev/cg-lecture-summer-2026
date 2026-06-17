use opencv::prelude::*;

// OpenCVのMat型に RgbaImage への変換機能を追加する拡張トレイト
pub trait MatExt {
    fn to_rgba_image(&self) -> opencv::Result<nannou::image::RgbaImage>;
}

impl MatExt for opencv::core::Mat {
    fn to_rgba_image(&self) -> opencv::Result<nannou::image::RgbaImage> {
        let mut rgba_mat = opencv::core::Mat::default();

        // チャンネル数に応じて適切な変換コードを選択
        let code = match self.channels() {
            3 => opencv::imgproc::COLOR_BGR2RGBA,
            4 => opencv::imgproc::COLOR_BGRA2RGBA,
            _ => {
                return Err(opencv::Error::new(
                    opencv::core::StsError,
                    "Unsupported channel count (must be BGR 3 channels or BGRA 4 channels)",
                ));
            }
        };

        // カラーフォーマットを変換
        opencv::imgproc::cvt_color(
            self,
            &mut rgba_mat,
            code,
            0,
            opencv::core::AlgorithmHint::ALGO_HINT_DEFAULT,
        )?;

        // メモリ領域が連続（continuous）していることを保証する
        let continuous_mat = if rgba_mat.is_continuous() {
            rgba_mat
        } else {
            rgba_mat.clone()
        };

        // rawバイト列の取得
        let bytes = continuous_mat.data_bytes()?;
        let size = continuous_mat.size()?;
        let width = size.width as u32;
        let height = size.height as u32;

        // RgbaImageの生成
        let image_buffer = nannou::image::RgbaImage::from_raw(width, height, bytes.to_vec())
            .ok_or_else(|| {
                opencv::Error::new(
                    opencv::core::StsError,
                    "Failed to create RgbaImage from raw bytes",
                )
            })?;

        Ok(image_buffer)
    }
}
