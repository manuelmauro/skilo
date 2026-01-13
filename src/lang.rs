//! Script language utilities.

use crate::cli::ScriptLang;

impl ScriptLang {
    /// Returns the file extension for this language.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Python => "py",
            Self::Bash => "sh",
            Self::Javascript => "js",
            Self::Typescript => "ts",
        }
    }

    /// Returns the shebang line for this language.
    pub fn shebang(&self) -> &'static str {
        match self {
            Self::Python => "#!/usr/bin/env python3",
            Self::Bash => "#!/usr/bin/env bash",
            Self::Javascript => "#!/usr/bin/env node",
            Self::Typescript => "#!/usr/bin/env -S npx ts-node",
        }
    }

    /// Returns the comment prefix for this language.
    pub fn comment_prefix(&self) -> &'static str {
        match self {
            Self::Python => "#",
            Self::Bash => "#",
            Self::Javascript => "//",
            Self::Typescript => "//",
        }
    }

    /// Returns the file name with the appropriate extension.
    pub fn file_name(&self, name: &str) -> String {
        format!("{}.{}", name, self.extension())
    }
}
