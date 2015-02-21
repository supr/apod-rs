#![feature(plugin)]
#![plugin(regex_macros)]
#![plugin(docopt_macros)]

extern crate regex;

extern crate "rustc-serialize" as rustc_serialize;

extern crate docopt;

extern crate curl;

use curl::http;
//use regex::Regex;
use std::str;
use std::string::String;
use std::process::Command;
use std::fs::File;
use std::os;
use std::cell::RefCell;
use std::io::Write;

static APOD_BASE_URL: &'static str = "http://apod.nasa.gov/apod/";

docopt!(Args derive Show, "
Usage: apod-rs [options] [-d DIR]
       apod-rs --help

Options:
       -d DIR                Download location.
       -h, --help            Show this message.
       -v, --verbose         Verbose.
");

struct MemoryPage
{
    code: u32,
    body: Vec<u8> 
}

struct Apod {
    handle: RefCell<http::Handle>
}


impl Apod {

    fn new(verbose: bool) -> Apod {
        if verbose {
            Apod { handle: RefCell::new(http::handle().verbose()) }
        } else {
            Apod { handle: RefCell::new(http::handle()) }
        }
    }

    fn get_page(&self, url: &str) -> MemoryPage {
        let resp = self.handle.borrow_mut().get(url)
                    .header("User-Agent", "apod-rs/0.1 github.com/supr/apod-rs")
                    .exec().unwrap();
        MemoryPage { code: resp.get_code(), body: resp.move_body() }
    }

    fn get_image_url<'a>(&self, page: &'a MemoryPage) -> Option<&'a str> {
        let rex = regex!("<a href=\"(image.*)\"");
        let body = str::from_utf8((&*page.body)).unwrap();
        match rex.is_match(body) {
            true => {
                rex.captures(body).unwrap().at(1)
            },
            false => None
        }
    }

    fn download_page(&self, to: &str, url: String) -> Option<String> {
        let page = self.get_page(&*url);
        let web_path = Path::new(url);
        let file_name = web_path.filename().unwrap();

        let mut fileObj = File::create(&Path::new(format!("{}/{}", to, str::from_utf8(file_name).unwrap())));
        if let Ok(mut file) = fileObj {
                match file.write(&page.body) {
                        Err(_) => None,
                        Ok(_) => Some(format!("{}/{}", to, str::from_utf8(file_name).unwrap()))
                }
        } else {
                None
        }
    }

    fn set_wallpaper(&self, file: String) {
        let file_path = format!("file://{}", file);
        let args = vec!["set", "org.gnome.desktop.background", "picture-uri", &*file_path];
        let _ = Command::new("gsettings").args(&*args).spawn();
    }
}

fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    let apod = Apod::new(args.flag_verbose);
    let page = apod.get_page(APOD_BASE_URL);

    match page.code {
        200...399 => {
            if let Some(url) = apod.get_image_url(&page) {
                let download_dir = if args.flag_d.len() > 0 {
                    args.flag_d
                } else {
                    format!("{}/Pictures", os::homedir().unwrap().display())
                };

                if let Some(downloaded_file) = apod.download_page(&*download_dir, format!("{}{}", APOD_BASE_URL, url)) {
                    apod.set_wallpaper(downloaded_file);
                } else {
                    println!("Unable to download wallpaper to: {}", download_dir);
                    os::set_exit_status(1);
                }
            }
        },
        _ => { println!("Unable to get APOD Page: {} Status Code: {}", APOD_BASE_URL, page.code); os::set_exit_status(1);  }
    }
}
