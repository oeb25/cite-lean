import Lake
open Lake DSL

package «Theory» where

@[default_target]
lean_lib «Theory» where

meta if get_config? env = some "dev" then -- dev is so not everyone has to build it
require «doc-gen4» from git "https://github.com/leanprover/doc-gen4" @ "main"
