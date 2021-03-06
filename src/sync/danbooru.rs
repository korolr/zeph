extern crate hyper;

use self::hyper::client::Client;

use std::thread;
use std::time::Duration;

use super::{Image,process_downloads,req_and_parse};

use std::sync::mpsc::Receiver;

pub fn main(rc: &Receiver<()>) {
    let client = Client::new();
    let mut url_string = "http://danbooru.donmai.us/posts.json?limit=100".to_string();
    let mut page = 1;

    loop {
        let res = match req_and_parse(&client, &url_string) {
            Ok(x) => x,
            Err(_) => {
                thread::sleep(Duration::new(3,0));
                continue
            }
        };

        let images = res.as_array().unwrap();
        if images.is_empty() { break }

        let images = images.iter().fold(Vec::new(), |mut acc, x| {
            let image = x.as_object().unwrap();
            let tags = image["tag_string"].as_str().unwrap().split_whitespace().map(String::from).collect::<Vec<_>>();
            let rating = image["rating"].as_str().unwrap().chars().nth(0).unwrap();

            if let Some(ext) = image.get("file_ext") {
                let ext = ext.as_str().unwrap();

                if ext != "webm" && ext != "swf" && ext != "mp4" {
                    let url = format!("http://danbooru.donmai.us{}", image["file_url"].as_str().unwrap().to_string());
                    let id = image["id"].as_i64().unwrap();
                    let name = format!("danbooru_{}.{}", id, ext);
                    let score = image["score"].as_i64().unwrap();

                    acc.push(Image{
                        name: name.to_string(),
                        got_from: "danbooru".to_string(),
                        url: url,
                        tags: tags,
                        rating: rating,
                        post_url: format!("http://danbooru.donmai.us/posts/{}", id),
                        score: score as i32
                    });
                }
            }
            acc
        });

        if process_downloads(&client, &images, rc).is_err() { break }

        page += 1;

        url_string = format!("http://danbooru.donmai.us/posts.json?page={}&limit=100", page);
    }
}
