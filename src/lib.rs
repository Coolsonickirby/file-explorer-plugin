#![feature(proc_macro_hygiene)]
extern crate tinytemplate;

use skyline_web::Webpage;
use tinytemplate::TinyTemplate;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use percent_encoding::percent_decode_str;

const LOCALHOST: &str = "http://localhost/";
const STARTING_FOLDER: &str = "sd:/";

static HTML_TEXT: &str = include_str!("resources/index.html");
static CSS_TEXT: &str = include_str!("resources/style.css");
static FILE_ICON: &[u8] = include_bytes!("resources/file.png");
static FOLDER_ICON: &[u8] = include_bytes!("resources/folder.png");

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct FileType{
    name: String,
    icon: String,
    is_directory: bool,
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
struct Context{
    current_dir: String,
    is_root: bool,
    listings: Vec<FileType>
}

pub fn get_directory_results(location: &str) -> Vec<FileType>{
    let mut directory_listing: Vec<FileType> = Vec::new();
    let paths = fs::read_dir(location).unwrap();

    for path in paths {
        let name = format!("{}", path.unwrap().path().display());
        let path_string = format!("{}/{}", location, name);
        let is_directory = PathBuf::from(&path_string).is_dir();
        let new_path_struct = FileType{
            name: String::from(name),
            icon: if is_directory {"folder.png".to_string()} else {"file.png".to_string()},
            is_directory
        };

        directory_listing.push(new_path_struct);
    }

    // Sort it alphabetically
    directory_listing.sort_by(|a, b| a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase()));

    // Sort it by bool
    directory_listing.sort_by(|a, b| b.is_directory.cmp(&a.is_directory));

    directory_listing
}

fn to_html(context: &Context) -> String {
    let mut tpl = TinyTemplate::new();
    tpl.add_template("page_listing", HTML_TEXT).unwrap();
    tpl.render("page_listing", &context).unwrap()
}

fn go_up(path: String) -> String{
    let split = path.split("/");
    let mut result: Vec<&str> = split.collect();
    let amount_to_remove: usize = if path.chars().last().unwrap() == '/' {2} else {1};
    
    result.truncate(result.len() - amount_to_remove);
    
    result.push("");
    
    result.join("/")
}

fn show_menu(context: Context) -> String {
    let response = Webpage::new()
        .file("index.html", &to_html(&context))
        .file("style.css", CSS_TEXT)
        .file("folder.png", FOLDER_ICON)
        .file("file.png", FILE_ICON)
        .background(skyline_web::Background::Default)
        .boot_display(skyline_web::BootDisplay::Black)
        .open()
        .unwrap();

    match response.get_last_url().unwrap(){
        "http://localhost/go_up" => go_up(context.current_dir),
        "" => "".to_string(),
        url => {
            if context.current_dir.chars().last().unwrap() == '/'{
                format!("{}{}", context.current_dir, percent_decode_str(&url[LOCALHOST.len()..]).decode_utf8_lossy().into_owned().to_string())
            }else{
                format!("{}/{}", context.current_dir, percent_decode_str(&url[LOCALHOST.len()..]).decode_utf8_lossy().into_owned().to_string())
            }
        }
    }
}

pub fn show_explorer(mut path: String) -> String{
    loop {
        let results: Vec<FileType> = get_directory_results(&path);
        
        let context = Context{
            current_dir: path.to_string(),
            is_root: path.matches("/").count() == 1,
            listings: results
        };

        let response_path = show_menu(context);
        
        
        if response_path != ""{
            path = format!("{}", response_path);
            if PathBuf::from(&path).is_dir(){
                if path.chars().last().unwrap() != '/'{
                    // Add slash if path is directory without / at the end
                    path = format!("{}/", path);
                }
            }else{
                // If file is selected, then return file path
                return path.to_string();
            }
        }else{
            // If invalid directory, then return last path
            return path.to_string();
        }
    }
}

#[skyline::main(name = "file-explorer")]
pub fn main() {
    
    let starting_path;

    if STARTING_FOLDER.chars().last().unwrap() != '/'{
        starting_path = format!("{}/", STARTING_FOLDER);
    }else{
        starting_path = STARTING_FOLDER.to_string();
    }

    let selected_file = show_explorer(starting_path);

    println!("Selected File Path: {}", selected_file);

}