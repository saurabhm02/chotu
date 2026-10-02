use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = r#"
    export function copyTextToClipboard(text) {
        if (navigator.clipboard && navigator.clipboard.writeText) {
            navigator.clipboard.writeText(text);
        } else {
            const textarea = document.createElement('textarea');
            textarea.value = text;
            textarea.style.position = 'fixed';
            textarea.style.opacity = '0';
            document.body.appendChild(textarea);
            textarea.select();
            document.execCommand('copy');
            document.body.removeChild(textarea);
        }
    }
"#)]
extern "C" {
    fn copyTextToClipboard(text: &str);
}

pub fn copy_to_clipboard(text: &str) {
    copyTextToClipboard(text);
}
