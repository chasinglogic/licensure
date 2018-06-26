use comments;
use licenses;
use std::fs::File;
use std::io::prelude::*;
use std::io::Error;

pub fn license_file(
    filename: &str,
    ident: &str,
    author: &str,
    email: &str,
    year: &str,
) -> Result<(), Error> {
    let mut f = File::open(filename);
    let mut content = String::new();

    if f.is_err() {
        return Err(f.err().unwrap());
    }

    let read_op = f.unwrap().read_to_string(&mut content);
    if read_op.is_err() {
        return Err(read_op.err().unwrap());
    }

    let filetype = comments::get_filetype(&filename);
    let commenter = comments::get_commenter(&filetype);
    let mut header = licenses::render_template(ident, author, email, year);
    header = commenter.comment(&header);
    header.push_str(&content);

    f = File::create(filename);
    if f.is_err() {
        return Err(f.err().unwrap());
    }

    f.unwrap().write_all(header.as_bytes()).map(|_x| ())
}
