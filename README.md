# cite-lean

Create links to Lean 4 documentation.

## Installation

You can either build it from source using `cargo build`, or download one of our generated binaries from [our releases](https://github.com/oeb25/cite-lean/releases/latest).

We also have installers for most platforms:

```bash
# linux/macos
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/oeb25/cite-lean/releases/download/v0.1.1/cite-lean-installer.sh | sh

# windows
powershell -ExecutionPolicy Bypass -c "irm https://github.com/oeb25/cite-lean/releases/download/v0.1.1/cite-lean-installer.ps1 | iex"
```

## Usage

### Building documentation

Documentation can be built using [doc-gen4](https://github.com/leanprover/doc-gen4). You need to add the following lines to your `lakefile.lean`:

```lean
meta if get_config? env = some "dev" then -- dev is so not everyone has to build it
require «doc-gen4» from git "https://github.com/leanprover/doc-gen4" @ "main"
```

Next, you need to build it like so:

```bash
lake -R -Kenv=dev build Theory:docs
```

And then host it some where.

If you are using GitHub, we recommend setting up GitHub Pages with the generated documentation. To get a starting point, `cite-lean` includes a template

```bash
mkdir -p .github/workflows
# replace Theory with the name of your library
cite-lean asset github-doc-gen4-ci Theory > .github/workflows/docs.yml
```

### Specifying citations

By putting `% cite-lean(Theory.my-theorem)` where `Theory.my-theorem` refers to the Lean path of you theorem, that line will be replaced by a link to the documentation.

```tex
% cite-lean(Theory.my-theorem)

% ... becomes ...
\citeLean{Theory/Basic.html\#Theory.my-theorem} % cite-lean(Theory.my-theorem)
```

> [!WARNING]
> Anything before the `%` will be overwritten by the `\citeLean` macro, so be careful what you put before it!

### Setting up macros

`cite-lean` provides some macros to get started.

```bash
# print out the tex macros
cite-lean asset tex-macros

# write out the Lean logo PDF used for buttons
cite-lean asset lean-pdf > lean-logo.pdf
```

### Updating citations

Every time you update your documentation, you need to fetch a new cache:

```bash
# download the latest documentation
cite-lean download --doc-url https://oeb25.github.io/cite-lean/
```

After that, call the following, where `tex/` is the root of your LaTeX files. All `.tex` files in that directory well be updated as needed.

```bash
# update all citations
cite-lean cite --write tex/
```
