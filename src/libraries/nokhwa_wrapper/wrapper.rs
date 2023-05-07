use std::io::{Cursor};
use image::{ImageOutputFormat};
use nokhwa::{{Camera, query}, Buffer, pixel_format::RgbAFormat, utils::{ApiBackend, CameraControl, CameraFormat, CameraIndex, CameraInfo, FrameFormat, RequestedFormat, RequestedFormatType, Resolution}};

/// Takes a screenshot using the given `camera` and returns the resulting image data as a vector of bytes in JPEG format.
///
/// # Arguments
///
/// * `camera` - A `Camera` object to capture the screenshot from.
///
/// # Errors
///
/// This function may return a `Box<dyn std::error::Error>` if there is an error capturing the screenshot or encoding it into JPEG format.
///
/// # Examples
///
/// ```
/// # use wrapper::Camera;
/// # let camera = Camera::new();
/// let screenshot_bytes = match wrapper::screenshot(camera) {
///     Ok(bytes) => bytes,
///     Err(e) => {
///         eprintln!("Error capturing screenshot: {}", e);
///         return;
///     }
/// };
///
/// // Do something with the screenshot bytes...
/// ```
pub fn snapshot(mut camera: Camera) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut buffers: Vec<Buffer> = Vec::new();

    // Takes about 5 frames to normalize colors for some reason.
    for _ in 0..5 {
        let frame = camera.frame()?;
        buffers.push(frame);
    }

    // Decode the image into an ImageBuffer
    let image = &buffers[4].decode_image::<RgbAFormat>().unwrap();

    let mut jpeg_bytes = Vec::new(); // Create a vector to store the JPEG bytes
    let mut cursor = Cursor::new(&mut jpeg_bytes); // Create a cursor for the byte buffer
    image.write_to(&mut cursor, ImageOutputFormat::Jpeg(100))?; // Encode the image into JPEG format with a quality of 100

    Ok(jpeg_bytes)

}

/// Returns a list of available cameras and their information.
///
/// This function queries the system for available cameras and returns their information in a vector of `CameraInfo` objects. The `CameraInfo` struct contains fields for the camera's human-readable name, description, miscellaneous information, and index.
///
/// # Errors
///
/// This function may return a `Box<dyn std::error::Error>` if there is an error querying the system for available cameras.
///
/// # Examples
///
/// ```
/// use wrapper::list_devices;
///
/// match list_devices() {
///     Ok(devices) => {
///         for device in devices {
///             println!("Camera: {} ({})", device.human_name, device.description);
///         }
///     }
///     Err(e) => {
///         eprintln!("Error listing devices: {}", e);
///         return;
///     }
/// }
/// ```
pub fn list_devices() ->  Result<Vec<CameraInfo>, Box<dyn std::error::Error>> {
    let devices = query(ApiBackend::Auto)?;
    Ok(devices)
}

pub fn init_static_cam(
    index: CameraIndex,
) -> Result<Camera, Box<dyn std::error::Error>> {
    // Find the best possible format for our use case
    let (resolution, frame_format, frame_rate) = (
        Resolution::new(1920, 1080),
        FrameFormat::MJPEG,
        30,
    );
    let requested = RequestedFormat::new::<RgbAFormat>(RequestedFormatType::Closest(CameraFormat::new(
        resolution, frame_format, frame_rate,
    )));

    Ok(Camera::new(index,requested)?)
}

//TODO: Implement this function in the front-end
pub fn _supported_controls(camera: &Camera) -> Result<Vec<CameraControl>, Box<dyn std::error::Error>> {

    // Get the supported controls for the provided camera and return a vector of said controls.
    let camera_controls = camera.camera_controls().unwrap(); //TODO: add error handling
    let mut controls = Vec::with_capacity(camera_controls.len());

    // Extends a collection with the provided camera controls
    controls.extend(camera_controls.iter().cloned());

    Ok(controls)
}