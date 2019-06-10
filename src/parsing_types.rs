use crate::parsers::list_env_parser;

pub struct RawHashlineParseData {
    indent_depth: usize,
    name: String,
    opts: String,
    args: String,
    comment: String,
}

#[derive(Debug, PartialEq)]
pub enum Hashline {
    OpenEnv(Environment),
    PlainLine(String),
}

#[derive(Debug, PartialEq)]
pub struct Environment {
    indent_depth: usize,
    name: String,
    opts: String,
    comment: String,
    is_list_like: bool,
}

impl RawHashlineParseData {
    pub fn new(
        indent_depth: usize,
        name: String,
        opts: String,
        args: String,
        comment: String,
    ) -> Self {
        RawHashlineParseData {
            indent_depth,
            name,
            opts,
            args,
            comment,
        }
    }
}

impl From<RawHashlineParseData> for Hashline {
    fn from(raw_hashline: RawHashlineParseData) -> Self {
        // FIXME: Trimming should not be a part of data conversion

        if raw_hashline.args.trim().is_empty() {
            // If no args are given, it's an environment
            Hashline::OpenEnv(Environment {
                indent_depth: raw_hashline.indent_depth,
                name: raw_hashline.name.trim().to_string(), // FIXME: Avoid reallocation here
                opts: raw_hashline.opts.trim().to_string(), // FIXME: Avoid reallocation here
                comment: raw_hashline.comment.trim().to_string(), // FIXME: Avoid reallocation here
                is_list_like: list_env_parser(raw_hashline.name.as_ref()).is_ok(),
            })
        } else {
            // If there are some args, it's a single-line command
            Hashline::PlainLine(format!(
                r"{dummy:ind$}\{name}{opts}{{{args}}}{comment_sep}{comment}",
                dummy = "",
                ind = raw_hashline.indent_depth,
                name = raw_hashline.name.trim(),
                opts = raw_hashline.opts.trim(),
                args = raw_hashline.args.trim(),
                comment_sep = if raw_hashline.comment.trim().is_empty() {
                    ""
                } else {
                    " "
                },
                comment = raw_hashline.comment.trim(),
            ))
        }
    }
}

impl Environment {
    pub fn latex_begin(&self) -> String {
        format!(
            r"{dummy:ind$}\begin{{{name}}}{opts}{comment_sep}{comment}",
            name = self.name,
            opts = self.opts,
            comment = self.comment,
            dummy = "",
            ind = self.indent_depth,
            comment_sep = if self.comment.is_empty() { "" } else { " " },
        )
    }

    pub fn latex_end(&self) -> String {
        format!(
            r"{dummy:ind$}\end{{{name}}}",
            name = self.name,
            dummy = "",
            ind = self.indent_depth,
        )
    }

    pub fn indent_depth(&self) -> usize {
        self.indent_depth
    }

    pub fn is_list_like(&self) -> bool {
        self.is_list_like
    }
}

#[cfg(test)]
mod tests {
    #[cfg(test)]
    mod raw_hashline_parser_data_into_hashline {
        use super::super::{Hashline, RawHashlineParseData};

        #[test]
        fn plain_lines() {
            assert_eq!(
                Hashline::from(RawHashlineParseData {
                    indent_depth: 0,
                    name: "foo".to_string(),
                    opts: "".to_string(),
                    args: "bar".to_string(),
                    comment: "".to_string()
                }),
                Hashline::PlainLine("\\foo{bar}".to_string())
            );
            assert_eq!(
                Hashline::from(RawHashlineParseData {
                    indent_depth: 2,
                    name: "foo".to_string(),
                    opts: "".to_string(),
                    args: "bar".to_string(),
                    comment: "qux".to_string()
                }),
                Hashline::PlainLine("  \\foo{bar} qux".to_string())
            );
            assert_eq!(
                Hashline::from(RawHashlineParseData {
                    indent_depth: 4,
                    name: "foo".to_string(),
                    opts: "bar".to_string(),
                    args: "qux".to_string(),
                    comment: "".to_string()
                }),
                Hashline::PlainLine("    \\foobar{qux}".to_string())
            );
        }

        #[test]
        fn environments() {
            use super::super::Environment;

            assert_eq!(
                Hashline::from(RawHashlineParseData {
                    indent_depth: 0,
                    name: "foo".to_string(),
                    opts: "bar".to_string(),
                    args: "".to_string(),
                    comment: "".to_string()
                }),
                Hashline::OpenEnv(Environment {
                    indent_depth: 0,
                    name: "foo".to_string(),
                    opts: "bar".to_string(),
                    comment: "".to_string(),
                    is_list_like: false,
                })
            );
            assert_eq!(
                Hashline::from(RawHashlineParseData {
                    indent_depth: 2,
                    name: "foo".to_string(),
                    opts: "".to_string(),
                    args: "".to_string(),
                    comment: "bar".to_string()
                }),
                Hashline::OpenEnv(Environment {
                    indent_depth: 2,
                    name: "foo".to_string(),
                    opts: "".to_string(),
                    comment: "bar".to_string(),
                    is_list_like: false,
                })
            );
            assert_eq!(
                Hashline::from(RawHashlineParseData {
                    indent_depth: 4,
                    name: "foo".to_string(),
                    opts: "bar".to_string(),
                    args: "".to_string(),
                    comment: "qux".to_string()
                }),
                Hashline::OpenEnv(Environment {
                    indent_depth: 4,
                    name: "foo".to_string(),
                    opts: "bar".to_string(),
                    comment: "qux".to_string(),
                    is_list_like: false,
                })
            );
            assert_eq!(
                Hashline::from(RawHashlineParseData {
                    indent_depth: 0,
                    name: "itemize".to_string(),
                    opts: "bar".to_string(),
                    args: "".to_string(),
                    comment: "qux".to_string()
                }),
                Hashline::OpenEnv(Environment {
                    indent_depth: 0,
                    name: "itemize".to_string(),
                    opts: "bar".to_string(),
                    comment: "qux".to_string(),
                    is_list_like: true,
                })
            );
        }
    }

    #[test]
    fn environment_methods() {
        use super::Environment;

        let env_1 = Environment {
            indent_depth: 0,
            name: "foo".to_string(),
            opts: "bar".to_string(),
            comment: "% baz".to_string(),
            is_list_like: true,
        };

        assert_eq!(env_1.latex_begin(), "\\begin{foo}bar % baz");
        assert_eq!(env_1.latex_end(), "\\end{foo}");
        assert_eq!(env_1.is_list_like(), true);
        assert_eq!(env_1.indent_depth(), 0);

        let env_2 = Environment {
            indent_depth: 2,
            name: "abc".to_string(),
            opts: "def".to_string(),
            comment: "".to_string(),
            is_list_like: false,
        };

        assert_eq!(env_2.latex_begin(), "  \\begin{abc}def");
        assert_eq!(env_2.latex_end(), "  \\end{abc}");
        assert_eq!(env_2.is_list_like(), false);
        assert_eq!(env_2.indent_depth(), 2);
    }
}
