//! The arboard shim's backend: Android's clipboard, through MainActivity.

use serde_json::json;

use crate::protocol::{Clip, decode_image, parse_clip};

pub struct SystemClipboard;

impl arboard::Backend for SystemClipboard {
    fn get(&self) -> Option<arboard::Content> {
        let answer = super::bridge::call("clip_get", "", None)?;
        match parse_clip(&answer) {
            Clip::None => None,
            Clip::Text(text) => Some(arboard::Content::Text(text)),
            Clip::Html { html, text } => Some(arboard::Content::Html { html, alt: Some(text) }),
            Clip::Image => {
                let (width, height, bytes) = decode_image(&super::bridge::call("clip_image", "", None)?)?;
                Some(arboard::Content::Image { width, height, bytes })
            }
        }
    }

    fn set(&self, content: arboard::Content) -> Result<(), String> {
        if !super::bridge::available() {
            return Err("the clipboard is not available yet".into());
        }
        match content {
            arboard::Content::Text(text) => super::bridge::call("clip_text", &json!({ "text": text }).to_string(), None),
            arboard::Content::Html { html, alt } => {
                super::bridge::call("clip_html", &json!({ "html": html, "text": alt.unwrap_or_default() }).to_string(), None)
            }
            arboard::Content::Image { width, height, bytes } => {
                super::bridge::call("clip_set_image", &json!({ "width": width, "height": height }).to_string(), Some(&bytes))
            }
            arboard::Content::Files(paths) => {
                let paths: Vec<String> = paths.iter().map(|p| p.to_string_lossy().into_owned()).collect();
                super::bridge::call("clip_files", &json!({ "paths": paths }).to_string(), None)
            }
        };
        Ok(())
    }
}
