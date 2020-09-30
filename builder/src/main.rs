use std::collections::HashMap;
use std::error::Error;
use std::fs::*;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let _ = remove_dir_all("output/");
    create_dir("output/")?;
    copy_dir("static/", "output/")?;

    let header = read_to_string("templates/header.html")?;
    let footer = read_to_string("templates/footer.html")?;

    {
        let mut vars = HashMap::new();
        vars.insert("title".to_string(), "404".to_string());

        let mut error_404 = String::new();
        error_404.push_str(&header);
        error_404.push_str(&read_to_string("404.html")?);
        error_404.push_str(&footer);

        write("output/404.html", substitute(&error_404, &vars))?;
    }

    let post_header = read_to_string("templates/post-header.html")?;

    let mut posts = Vec::new();
    create_dir("output/posts/")?;
    for post in read_dir("posts/")? {
        let post = post?;
        let mut url = "posts/".to_string();
        url.push_str(&post.file_name().into_string().map_err(|_| "")?[9..].to_string());

        let mut info_path = post.path().to_path_buf();
        info_path.push("info.txt");
        let info = read_to_string(info_path)?;
        let info_lines: Vec<&str> = info.lines().collect();

        let title = info_lines[0].to_string();
        let date = info_lines[1].to_string();
        posts.push((url.clone(), title.clone(), date.clone()));

        let mut out_dir = PathBuf::from("output/");
        out_dir.push(&url);

        create_dir(&out_dir)?;

        for file in read_dir(post.path())? {
            let file = file?;
            let file_name = file.file_name();
            if file_name != "index.html" && file_name != "info.txt" {
                let mut out_path = out_dir.clone();
                out_path.push(file_name);
                copy(file.path(), out_path)?;
            }
        }

        let mut contents_path = post.path().to_path_buf();
        contents_path.push("index.html");

        let contents = render_katex(&read_to_string(contents_path)?)?;

        let mut vars = HashMap::new();
        vars.insert("date".to_string(), date);
        vars.insert("title".to_string(), title);

        let mut post = String::new();
        post.push_str(&header);
        post.push_str(&post_header);
        post.push_str(&contents);
        post.push_str(&footer);

        let mut post_dir = out_dir.clone();
        post_dir.push("index.html");
        write(post_dir, substitute(&post, &vars))?;
    }

    {
        let mut vars = HashMap::new();
        vars.insert("title".to_string(), "home".to_string());

        let mut index = String::new();
        index.push_str(&header);
        index.push_str(&read_to_string("templates/post-list-begin.html")?);

        let post_list_item = read_to_string("templates/post-list-item.html")?;
        for (url, title, date) in posts {
            let mut vars = HashMap::new();
            vars.insert("url".to_string(), url);
            vars.insert("date".to_string(), date);
            vars.insert("title".to_string(), title);
            index.push_str(&substitute(&post_list_item, &vars));
        }

        index.push_str(&read_to_string("templates/post-list-end.html")?);
        index.push_str(&footer);

        write("output/index.html", substitute(&index, &vars))?;

        copy("output/index.html", "output/posts/index.html")?;
    }    

    Ok(())
}

fn copy_dir<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<(), Box<dyn Error>> {
    let _ = create_dir(&to);

    for entry in read_dir(from)? {
        let entry = entry?;
        let mut out_path = to.as_ref().to_path_buf();
        out_path.push(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(entry.path(), out_path)?;
        } else {
            copy(entry.path(), out_path)?;
        }
    }

    Ok(())
}

fn substitute(input: &str, vars: &HashMap<String, String>) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars();
    while let Some(c) = chars.next() {
        if c == '$' {
            if let Some(c) = chars.next() {
                if c == '{' {
                    let mut name = String::new();
                    while let Some(c) = chars.next() {
                        if c == '}' {
                            break;
                        }
                        name.push(c);
                    }

                    if let Some(value) = vars.get(&name) {
                        output.push_str(value);
                    }
                } else {
                    output.push('$');
                    output.push(c);
                }
            } else {
                output.push('$');
            }
        } else {
            output.push(c);
        }
    }

    output
}

fn render_katex(input: &str) -> Result<String, Box<dyn Error>> {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(c) = chars.next() {
                if c == '(' {
                    let mut string = String::new();
                    while let Some(c) = chars.next() {
                        if c == '\\' {
                            if let Some(c) = chars.next() {
                                if c == ')' {
                                    break;
                                } else {
                                    string.push('\\');
                                    string.push(c);
                                }
                            } else {
                                string.push(c);
                            }
                        } else {
                            string.push(c);
                        }
                    }

                    output.push_str(&katex::render(&string)?);
                } else if c == '[' {
                    let mut string = String::new();
                    while let Some(c) = chars.next() {
                        if c == '\\' {
                            if let Some(c) = chars.next() {
                                if c == ']' {
                                    break;
                                } else {
                                    string.push('\\');
                                    string.push(c);
                                }
                            } else {
                                string.push(c);
                            }
                        } else {
                            string.push(c);
                        }
                    }

                    let opts = katex::Opts::builder().display_mode(true).build()?;
                    output.push_str(&katex::render_with_opts(&string, opts)?);
                } else {
                    output.push('\\');
                    output.push(c);
                }
            } else {
                output.push('\\');
            }
        } else {
            output.push(c);
        }
    }

    Ok(output)
}