//! Reading the text in an image with Apple's Vision framework.
//! It runs on the device: no internet and no AI model involved. macOS only.
use std::path::Path;

/// All the text Vision finds in the image, one line per detected line of text.
/// Returns an error when the image has no readable text

#[cfg(target_os = "macos")]
pub fn read_text(path: &Path) -> Result<String, String> {
    let text = mac::read_lines(path)?.join("\n");
    if text.trim().is_empty() {
        return Err("No readable text found in image".to_string());
    }
    Ok(text)
}

#[cfg(not(target_os = "macos"))]
pub fn read_text(_image_path: &Path) -> Result<String, String> {
    Err("Reading text from images only works on macOS for now.".to_string())
}

#[cfg(target_os = "macos")]
mod mac {
    use std::path::Path;

    use objc2::rc::{autoreleasepool, Retained};
    use objc2::AnyThread;
    use objc2_foundation::{NSArray, NSDictionary, NSString, NSURL};
    use objc2_vision::{
        VNImageRequestHandler, VNRecognizeTextRequest, VNRecognizedTextObservation, VNRequest,
        VNRequestTextRecognitionLevel,
    };

    /// Runs Vision on the file and returns the best guess for each line of text(AI_WRITTEN)
    pub fn read_lines(image_path: &Path) -> Result<Vec<String>, String> {
        // Vision creates temporary Objective-C objects; the pool frees them when we are done.
        autoreleasepool(|_| {
            let url = NSURL::fileURLWithPath(&NSString::from_str(&image_path.to_string_lossy()));
            let options = NSDictionary::new();
            let handler = unsafe {
                VNImageRequestHandler::initWithURL_options(
                    VNImageRequestHandler::alloc(),
                    &url,
                    &options,
                )
            };

            // "Accurate" is slower than "Fast" but much better on small code text.
            let request = VNRecognizeTextRequest::new();
            request.setRecognitionLevel(VNRequestTextRecognitionLevel::Accurate);

            // `performRequests` wants a list of the general `VNRequest` type.
            let as_request: Retained<VNRequest> =
                Retained::into_super(Retained::into_super(request.clone()));
            handler
                .performRequests_error(&NSArray::from_retained_slice(&[as_request]))
                .map_err(|e| format!("text recognition failed: {}", e.localizedDescription()))?;

            let observations: Retained<NSArray<VNRecognizedTextObservation>> =
                request.results().unwrap_or_default();

            let mut lines = Vec::new();
            for observation in observations.iter() {
                // Vision offers several guesses per line, best first; keep the best.
                if let Some(best) = observation.topCandidates(1).firstObject() {
                    lines.push(best.string().to_string());
                }
            }
            Ok(lines)
        })
    }
}
