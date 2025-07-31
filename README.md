# TaskTaskRevolution

A command-line static site generator for managing your projects, tasks, and resources through simple YAML files.

## Core Concept

TaskTaskRevolution (ttr) transforms a structured directory of YAML files into a clean, modern, and readable static HTML website. Instead of complex commands to manage your project, you manage it by creating and editing simple text files. This approach makes your project data transparent, version-controllable, and easy to manage with your favorite text editor.

## How It Works

1.  **Initialize:** Run `ttr init` in a new directory to create a global `config.yaml`.
2.  **Define:** Create directories for your projects. Inside each, you define your project (`project.yaml`), your resources (`resources/*.yaml`), and your tasks (`tasks/*.yaml`).
3.  **Build:** Run `ttr build` from the root directory. `ttr` will discover all your projects and generate a complete HTML static site in the `public` directory, with separate pages for each project.

## Installation

```bash
# Clone the repository
git clone https://gitlab.com/flaviogranato/tasktaskrevolution.git

# Enter the directory
cd tasktaskrevolution

# Compile the project
cargo build --release

# Optional: Add the binary to your PATH
# e.g., sudo cp target/release/ttr /usr/local/bin/
```

## Usage

### Initialize a Repository

To start, create a root directory for all your projects and run the `init` command. This will create a `config.yaml` file to hold global settings.

```bash
mkdir my-projects
cd my-projects
ttr init --manager-name "Your Name" --manager-email "your@email.com"
```

### Define Your Project

Inside your repository, create a directory for each project. At a minimum, each project needs a `project.yaml` file.

**Example Directory Structure:**

```
my-projects/
├── config.yaml
└── my-first-project/
    ├── project.yaml
    ├── resources/
    │   └── john_doe.yaml
    └── tasks/
        └── TSK-01.yaml
```

**`project.yaml` Example:**

```yaml
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Project
metadata:
  name: "My First Project"
  description: "A simple project to get started."
spec:
  status: InProgress
```

### Build the Static Site

From the root directory (`my-projects` in this example), run the `build` command.

```bash
ttr build
```

This will generate the static site in a `public` directory. The output will contain a subdirectory for each project found.

## Development

This project follows the principles of Clean Architecture and Domain-Driven Design (DDD), organizing the code into well-defined layers:

-   **Domain**: Contains the core business logic and entities.
-   **Application**: Implements the application's use cases.
-   **Infrastructure**: Provides concrete implementations for persistence, etc.
-   **Interface**: Manages user interaction (CLI).

### Setting Up the Environment

1.  Clone the repository.
2.  Install dependencies: `cargo build`
3.  Run the tests: `cargo test`

## Contributing

This is a personal project under active development. For this reason, external contributions are not being accepted at this time.

## License

This project is licensed under the [Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International License](https://creativecommons.org/licenses/by-nc-sa/4.0/).

See the [LICENSE](LICENSE) file for more details.