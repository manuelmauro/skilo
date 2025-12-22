use crate::cli::ScriptLang;

impl ScriptLang {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Python => "py",
            Self::Bash => "sh",
            Self::Javascript => "js",
            Self::Typescript => "ts",
        }
    }

    pub fn shebang(&self) -> &'static str {
        match self {
            Self::Python => "#!/usr/bin/env python3",
            Self::Bash => "#!/usr/bin/env bash",
            Self::Javascript => "#!/usr/bin/env node",
            Self::Typescript => "#!/usr/bin/env -S npx ts-node",
        }
    }

    pub fn comment_prefix(&self) -> &'static str {
        match self {
            Self::Python => "#",
            Self::Bash => "#",
            Self::Javascript => "//",
            Self::Typescript => "//",
        }
    }

    pub fn file_name(&self, name: &str) -> String {
        format!("{}.{}", name, self.extension())
    }
}
