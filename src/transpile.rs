use std::path::Path;
use std::vec::Vec;
use error::IndentexError;


const LINESEP: &'static str = "\n";


// Indentation processing
#[inline]
fn count_left_indent<T: AsRef<str>>(line: T) -> Option<usize> {
    if line.as_ref().is_empty() {
        None
    } else {
        Some(line.as_ref().chars().count() - line.as_ref().trim_left().chars().count())
    }
}

fn scan_indents<T: AsRef<str>>(lines: &[T]) -> Vec<usize> {
    let raw_indents = lines.iter().map(count_left_indent).collect::<Vec<_>>();

    let mut adjusted_indents: Vec<usize> = Vec::with_capacity(raw_indents.len() + 1);
    let mut last_indent: usize = 0;

    for current_indent in raw_indents.iter().rev() {
        adjusted_indents.push(match *current_indent {
            None => last_indent,
            Some(ind) => {
                last_indent = ind;
                ind
            }
        });
    }

    adjusted_indents.reverse();
    adjusted_indents.push(0);

    adjusted_indents
}


// Transpilation
fn transpile<T: AsRef<str>>(lines: &[T]) -> String {
    use parsers::Environment;
    use parsers::Hashline::{PlainLine, OpenEnv};
    use parsers::process_line;

    // The number of environments is not known beforehand
    let mut env_stack: Vec<Environment> = Vec::new();
    // TODO: Estimate the required string capacity
    let mut transpiled = String::new();

    let adjusted_indents = scan_indents(lines);

    for (line_num, line) in lines.iter().enumerate() {
        let list_like_active = match env_stack.last() {
            None => false, // No environment is active at all
            Some(d) => d.is_list_like(),
        };

        let tl = match process_line(line.as_ref(), list_like_active) {
            PlainLine(l) => l,
            OpenEnv(e) => {
                let tag_begin = e.latex_begin();
                env_stack.push(e);
                tag_begin
            }
        };
        transpiled.push_str(&tl);
        transpiled.push_str(LINESEP);

        // Check if we are in an environment and close as many as needed
        while match env_stack.last() {
            None => false,
            Some(d) => d.indent_depth() >= adjusted_indents[line_num + 1],
        } {
            // `unwrap()` is safe here since we have already checked if the stack is empty
            let tag_end = env_stack.pop().unwrap().latex_end();
            transpiled.push_str(&tag_end);
            transpiled.push_str(LINESEP);
        }
    }

    transpiled
}

pub fn transpile_file<T: AsRef<Path>>(path: T) -> Result<(), IndentexError> {
    use file_utils::{read_and_trim_lines, rename_indentex_file, write_to_file};

    let lines = read_and_trim_lines(path.as_ref())?;
    let transpiled_text = transpile(&lines);
    let path_out = rename_indentex_file(path)?;
    write_to_file(path_out, &transpiled_text)?;

    Ok(())
}
