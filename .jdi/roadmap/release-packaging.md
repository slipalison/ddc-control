---
order: 5.6
name: Release packaging
---
- **Slug:** release-packaging
- **Goal:** bundles Tauri (MSI/NSIS no Windows; deb/rpm/AppImage no Linux incluindo regra udev) + binários do `ddc-cli`, release automático (versão semântica pelos commits, tag e GitHub Release com os pacotes anexados para download e notas do CHANGELOG) via template reutilizável em `slipalison/github-workflows`
