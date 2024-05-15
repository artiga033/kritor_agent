use std::collections::HashMap;
use std::collections::VecDeque;

use super::Error;

use base64::prelude::*;
use kritor::common::element::Data as KritorElementData;
use kritor::common::element::ElementType as KritorElementType;
use kritor::common::image_element as kritor_image;
use kritor::common::Element as KritorElement;

#[derive(Debug)]
pub enum Element {
    Tag(Box<TagElement>),
    Plain(String),
}
#[derive(Debug)]
pub struct TagElement {
    pub name: String,
    pub attributes: Option<HashMap<String, String>>,
    pub children: Option<Vec<Element>>,
}

impl TagElement {
    pub fn children_mut(&mut self) -> &mut Vec<Element> {
        self.children.get_or_insert_with(Vec::new)
    }
    pub fn attributes_mut(&mut self) -> &mut HashMap<String, String> {
        self.attributes.get_or_insert_with(HashMap::new)
    }
}

impl Element {
    pub fn empty_tag() -> Self {
        Self::with_tag_name(String::new())
    }
    pub fn with_tag_name(name: String) -> Self {
        Self::Tag(Box::new(TagElement {
            name,
            attributes: None,
            children: None,
        }))
    }
    pub fn new_plain<S: ToString>(text: S) -> Self {
        Self::Plain(text.to_string())
    }
    pub fn is_plain(&self) -> bool {
        matches!(self, Self::Plain(_))
    }
    pub fn text_mut(&mut self) -> Option<&mut String> {
        if let Self::Plain(s) = self {
            Some(s)
        } else {
            None
        }
    }
    pub fn text(&self) -> Option<&String> {
        if let Self::Plain(s) = self {
            Some(s)
        } else {
            None
        }
    }
    pub fn tag_mut(&mut self) -> Option<&mut TagElement> {
        if let Self::Tag(e) = self {
            Some(e)
        } else {
            None
        }
    }
    pub fn tag(&self) -> Option<&TagElement> {
        if let Self::Tag(e) = self {
            Some(e)
        } else {
            None
        }
    }
}

impl From<TagElement> for Element {
    #[inline]
    fn from(value: TagElement) -> Self {
        Self::Tag(Box::new(value))
    }
}
impl From<Box<TagElement>> for Element {
    fn from(value: Box<TagElement>) -> Self {
        Self::Tag(value)
    }
}

#[derive(Debug)]
pub struct Root {
    pub root_element: Element,
}

impl Root {
    pub fn try_into_kritor_elements(self) -> Result<Vec<KritorElement>, Error> {
        match self.root_element {
            Element::Plain(text) => Ok(vec![KritorElement {
                r#type: KritorElementType::Text.into(),
                data: Some(KritorElementData::Text(kritor::common::TextElement {
                    text,
                })),
            }]),
            Element::Tag(tag) => {
                let mut result = Vec::new();
                let mut queue = tag.children.map(VecDeque::from).unwrap_or_default();
                while let Some(mut tag) = queue.pop_front() {
                    if let Some(ref mut children) = tag.tag_mut().and_then(|x| x.children.as_mut())
                    {
                        for child in children.drain(..) {
                            queue.push_back(child);
                        }
                    }
                    result.push(tag.try_into()?);
                }
                Ok(result)
            }
        }
    }
    pub fn try_from_kritor_elements(value: Vec<KritorElement>) -> Result<Self, Error> {
        let root_element = Element::from(TagElement {
            name: "".into(),
            attributes: None,
            children: {
                let mut result = Vec::new();
                for element in value.into_iter() {
                    result.push(element.try_into()?)
                }
                Some(result)
            },
        });
        Ok(Root { root_element })
    }
}

impl Element {
    /// Serialize a element into the satori message representation.
    pub fn serialize(&self) -> String {
        match self {
            Element::Plain(text) => text.clone(),
            Element::Tag(tag) => tag.serialize(),
        }
    }
}

impl TagElement {
    pub fn serialize(&self) -> String {
        let has_tag_name = !self.name.is_empty();
        let mut result = String::new();
        if has_tag_name {
            result.push_str(&format!("<{}", self.name));
        }
        if let Some(attributes) = &self.attributes {
            for (k, v) in attributes {
                result.push_str(&format!(" {}=\"{}\"", k, v));
            }
        }
        if let Some(children) = &self.children {
            if has_tag_name {
                result.push('>');
            }
            for child in children {
                result.push_str(&child.serialize());
            }
            if has_tag_name {
                result.push_str(&format!("</{}>", self.name));
            }
        } else if has_tag_name {
            result.push_str("/>");
        }
        result
    }
}

impl TryFrom<Element> for kritor::common::Element {
    type Error = Error;

    /// Note that the children will be lost.
    fn try_from(value: Element) -> Result<Self, Self::Error> {
        match value {
            Element::Plain(text) => Ok(KritorElement {
                r#type: KritorElementType::Text.into(),
                data: Some(KritorElementData::Text(kritor::common::TextElement {
                    text,
                })),
            }),
            Element::Tag(tag) => Ok(match tag.name.as_str() {
                "at" => KritorElement {
                    r#type: KritorElementType::At.into(),
                    data: tag.attributes.map(|mut x| {
                        KritorElementData::At(kritor::common::AtElement {
                            uin: x.get("id").and_then(|x| x.parse().ok()),
                            uid: x
                                .remove("type")
                                .filter(|x| x.as_str() == "all")
                                .unwrap_or_default(),
                        })
                    }),
                },
                "face" => KritorElement {
                    r#type: KritorElementType::Face.into(),
                    data: tag.attributes.and_then(|x| {
                        Some(KritorElementData::Face(kritor::common::FaceElement {
                            id: x.get("id").and_then(|x| x.parse::<u32>().ok())?,
                            ..Default::default()
                        }))
                    }),
                },
                "img" => KritorElement {
                    r#type: KritorElementType::Image.into(),
                    data: tag.attributes.and_then(|mut x| {
                        Some(KritorElementData::Image(kritor::common::ImageElement {
                            r#type: Some(kritor_image::ImageType::Common.into()),
                            data: Some(kritor_image::Data::FileUrl(x.remove("src")?)),
                            file_md5: None,
                            sub_type: None,
                        }))
                    }),
                },
                "audio" => KritorElement {
                    r#type: KritorElementType::Voice.into(),
                    data: tag.attributes.and_then(|mut x| {
                        Some(KritorElementData::Voice(kritor::common::VoiceElement {
                            data: Some(kritor::common::voice_element::Data::FileUrl(
                                x.remove("src")?,
                            )),
                            file_md5: None,
                            magic: None,
                        }))
                    }),
                },
                "video" => KritorElement {
                    r#type: KritorElementType::Video.into(),
                    data: tag.attributes.and_then(|mut x| {
                        Some(KritorElementData::Video(kritor::common::VideoElement {
                            data: Some(kritor::common::video_element::Data::FileUrl(
                                x.remove("src")?,
                            )),
                            file_md5: None,
                        }))
                    }),
                },
                "file" => KritorElement {
                    r#type: KritorElementType::File.into(),
                    data: tag.attributes.map(|mut x| {
                        KritorElementData::File(kritor::common::FileElement {
                            url: x.remove("src"),
                            name: x.remove("title"),
                            ..Default::default()
                        })
                    }),
                },
                "message"
                    if tag
                        .attributes
                        .as_ref()
                        .is_some_and(|x| x.get("forward").is_some_and(|x| x == "true")) =>
                {
                    KritorElement {
                        r#type: KritorElementType::Forward.into(),
                        data: tag.attributes.and_then(|mut x| {
                            Some(KritorElementData::Forward(kritor::common::ForwardElement {
                                uniseq: x.remove("id")?,
                                ..Default::default()
                            }))
                        }),
                    }
                }
                _ => KritorElement {
                    r#type: KritorElementType::Text.into(),
                    data: Some(KritorElementData::Text(kritor::common::TextElement {
                        text: tag.serialize(),
                    })),
                },
            }),
        }
    }
}

impl TryFrom<KritorElement> for Element {
    type Error = Error;

    fn try_from(value: KritorElement) -> Result<Self, Self::Error> {
        let r#type = KritorElementType::try_from(value.r#type).map_err(|_| "Invalid type")?;
        Ok(match r#type {
            KritorElementType::Text => {
                let data = value.data.ok_or("Missing data")?;
                let text = match data {
                    KritorElementData::Text(x) => x.text,
                    _ => return Err("Invalid data".into()),
                };
                Element::new_plain(text)
            }
            KritorElementType::At => {
                let data = value.data.ok_or("Missing data")?;
                let data = match data {
                    KritorElementData::At(x) => x,
                    _ => return Err("Invalid data".into()),
                };
                Element::from(TagElement {
                    name: "at".to_string(),
                    attributes: Some({
                        let mut map = HashMap::new();
                        if data.uid == "all" {
                            map.insert("type".to_string(), "all".to_string());
                        } else {
                            map.insert(
                                "id".to_string(),
                                data.uin.ok_or("Missing uin")?.to_string(),
                            );
                        }
                        map
                    }),
                    children: None,
                })
            }
            KritorElementType::Face => {
                let data = value.data.ok_or("Missing data")?;
                let data = match data {
                    KritorElementData::Face(x) => x,
                    _ => return Err("Invalid data".into()),
                };
                Element::from(TagElement {
                    name: "face".to_string(),
                    attributes: Some({
                        let mut map = HashMap::new();
                        map.insert("id".to_string(), data.id.to_string());
                        map
                    }),
                    children: None,
                })
            }
            KritorElementType::Image => {
                let data = value.data.ok_or("Missing data")?;
                let data = match data {
                    KritorElementData::Image(x) => x,
                    _ => return Err("Invalid data".into()),
                };
                Element::from(TagElement {
                    name: "img".to_string(),
                    attributes: Some({
                        let mut map = HashMap::new();
                        map.insert(
                            "src".to_string(),
                            match data.data.ok_or("Missing data")? {
                                kritor_image::Data::FileUrl(x) => x,
                                kritor_image::Data::FilePath(x) => format!("file://{}", x),
                                kritor_image::Data::File(x) => encode_data_url(x.as_slice()),
                                _ => {
                                    return Err(
                                        "satori does not support this type of resource".into()
                                    )
                                }
                            },
                        );
                        map
                    }),
                    children: None,
                })
            }
            KritorElementType::Voice => {
                let data = value.data.ok_or("Missing data")?;
                let data = match data {
                    KritorElementData::Voice(x) => x,
                    _ => return Err("Invalid data".into()),
                };
                Element::from(TagElement {
                    name: "audio".to_string(),
                    attributes: Some({
                        let mut map = HashMap::new();
                        map.insert(
                            "src".to_string(),
                            match data.data.ok_or("Missing data")? {
                                kritor::common::voice_element::Data::FileUrl(x) => x,
                                kritor::common::voice_element::Data::FilePath(x) => {
                                    format!("file://{}", x)
                                }
                                kritor::common::voice_element::Data::File(x) => {
                                    encode_data_url(x.as_slice())
                                }
                                _ => {
                                    return Err(
                                        "satori does not support this type of resource".into()
                                    )
                                }
                            },
                        );
                        map
                    }),
                    children: None,
                })
            }
            KritorElementType::Video => {
                let data = value.data.ok_or("Missing data")?;
                let data = match data {
                    KritorElementData::Video(x) => x,
                    _ => return Err("Invalid data".into()),
                };
                Element::from(TagElement {
                    name: "video".to_string(),
                    attributes: Some({
                        let mut map = HashMap::new();
                        map.insert(
                            "src".to_string(),
                            match data.data.ok_or("Missing data")? {
                                kritor::common::video_element::Data::FileUrl(x) => x,
                                kritor::common::video_element::Data::FilePath(x) => {
                                    format!("file://{}", x)
                                }
                                kritor::common::video_element::Data::File(x) => {
                                    encode_data_url(x.as_slice())
                                }
                                _ => {
                                    return Err(
                                        "satori does not support this type of resource".into()
                                    )
                                }
                            },
                        );
                        map
                    }),
                    children: None,
                })
            }
            _ => {
                return Err("including elements unsupported by satori.".into());
            }
        })
    }
}

fn encode_data_url(data: &[u8]) -> String {
    let mime = infer::get(data)
        .map(|x| x.mime_type())
        .unwrap_or("application/octet-stream");
    let mut data_url = format!("data:{};base64,", mime);
    BASE64_STANDARD.encode_string(data, &mut data_url);
    data_url
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() {
        let element = Element::Tag(Box::new(TagElement {
            name: "img".to_string(),
            attributes: Some({
                let mut map = HashMap::new();
                map.insert(
                    "src".to_string(),
                    "https://example.com/image.png".to_string(),
                );
                map
            }),
            children: None,
        }));
        assert_eq!(
            element.serialize(),
            "<img src=\"https://example.com/image.png\"/>"
        );
    }

    #[test]
    fn test_from_to_kritor() {
        let msg = super::super::Parser::new(
            r#"Hello <img src="https://s.a33.su/avatar.png"/> <at id="10086"></at>"#,
        )
        .parse()
        .unwrap();
        dbg!(&msg.root_element);
        let kritor_element: Vec<KritorElement> = msg.try_into_kritor_elements().unwrap();
        dbg!(&kritor_element);
    }

    #[test]
    fn test_encode_data_url() {
        let data = b"Hello, world!";
        let data_url = encode_data_url(data);
        assert_eq!(
            data_url,
            "data:application/octet-stream;base64,SGVsbG8sIHdvcmxkIQ=="
        );
        let data = &[0xFF, 0xD8, 0xFF, 0xAA, 0x39, 0x39];
        let data_url = encode_data_url(data);
        assert_eq!(data_url, "data:image/jpeg;base64,/9j/qjk5");
    }
}
