
<p align="center">
<img src="arts/main.png" width="400px">
</p>


![GitHub Release](https://img.shields.io/github/v/release/funcieqDEV/Atra)
![GitHub Commits](https://img.shields.io/github/commit-activity/m/funcieqDEV/Atra)
![GitHub Stars](https://img.shields.io/github/stars/funcieqDEV/Atra?style=social)
![License](https://img.shields.io/github/license/funcieqDEV/Atra)
![Build Status](https://img.shields.io/github/actions/workflow/status/funcieqDEV/Atra/rust.yml)

Atra is a modern, efficient static website generator designed for creating fast, component-based websites. With its intuitive syntax and powerful component system, Atra makes building static sites both productive and enjoyable.

## ✨ Features

- **Component-based architecture** - Reusable components with `.atrac` files
- **Modern syntax** - Clean, readable markup language
- **Fast compilation** - Built with Rust for maximum performance
- **Hot reloading** - Watch mode for instant development feedback
- **CSS-in-JS style** - Inline styling with familiar CSS syntax
- **Static output** - Generates optimized HTML files

## 🚀 Quick Start

### Installation

#### From Releases (Recommended)
1. Download the latest release from [GitHub Releases](https://github.com/funcieqDEV/Atra/releases)
2. Extract the binary to a directory in your PATH

#### Building from Source
```bash
git clone https://github.com/funcieqDEV/Atra.git
cd Atra
cargo build --release
```

### Adding to System PATH

#### Windows
1. Copy `atra.exe` to `C:\Program Files\Atra\`
2. Add `C:\Program Files\Atra\` to your system PATH
3. Open a new terminal and verify with `atra --help`

#### Linux/macOS
```bash
# Copy binary to /usr/local/bin
sudo cp target/release/atra /usr/local/bin/

# Or add to your shell profile
echo 'export PATH=$PATH:/path/to/atra' >> ~/.bashrc
source ~/.bashrc
```

## 📖 Usage

### Basic Commands

```bash
# Build your project
atra build config.json

# Watch for changes and rebuild automatically
atra watch config.json
```

### Configuration

Create a `config.json` file:

```json
{
  "source_folder": "src",
  "output_folder": "dist"
}
```

### Basic Example

Create an `index.atra` file:

```atra
text("<!DOCTYPE html");
html(lang="en"){
    head(){
        title(){
            text("Atra!")
        }
        meta(charset="utf");
    }
    body()
        p(){
            text("Hello Atra!")
        }
    }
}
```

This generates clean HTML even with indentations

### Components

Create reusable components with `.atrac` files:

**components/button.atrac**
```atra
$Button(text, color) {
    button()[
        background-color: {color};
        padding: 10px 20px;
        border: none;
        border-radius: 5px;
        cursor: pointer;
    ] {
        text("{text}");
    }
}
```

**index.atra**
```atra
div() {
    $Button("Click me!", "#007bff");
    $Button("Secondary", "#6c757d");
}
```


#### Loops
```atra
ul() {
    %loop(13){
        li(){
            text("element");
        }
    }
}
```

## 🏗️ Project Structure

```
my-atra-project/
├── src/
│   ├── index.atra
│   └── components/
├── dist/           # Generated HTML files
└── config.json
```
> I recommend keeping the components in a separate folder

## 🤝 Contributing

We welcome contributions! Here's how you can help:

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/amazing-feature`
3. **Commit your changes**: `git commit -m 'Add amazing feature'`
4. **Push to the branch**: `git push origin feature/amazing-feature`
5. **Open a Pull Request**

### Development Setup

```bash
git clone https://github.com/funcieqDEV/Atra.git
cd Atra
cargo build
cargo test
```

### Code Style
- Follow Rust standard formatting with `cargo fmt`
- Ensure all tests pass with `cargo test`
- Add tests for new features

## 📝 Examples

Check out the `test/` directory for working examples, or visit our [Example webistes](https://github.com/funcieqDEV/Atra/tree/main/examples/) for more complex projects.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🌟 Support

If you find Atra useful, please consider:
- ⭐ Starring the repository
- 🐛 Reporting bugs
- 💡 Suggesting new features
- 📖 Improving documentation

## 📞 Contact

- GitHub Issues: [Report bugs or request features](https://github.com/funcieqDEV/Atra/issues)
- Discussions: [Community discussions](https://github.com/funcieqDEV/Atra/discussions)
- discord: funcieq

---

Built with ❤️ in Rust
