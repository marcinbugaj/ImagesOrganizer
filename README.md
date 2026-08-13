# ImagesOrganizer

Sorts a directory of photographs into folders by **where they were taken**, using the GPS
coordinates stored in their EXIF metadata.

Photographs from a single trip are rarely taken at one point — they are scattered across a
town, a valley or a coastline. Sorting by a fixed radius or by a grid cuts those groups in
the wrong places. This tool clusters the coordinates with HDBSCAN instead, which finds
groups of varying size and density and arranges them into a hierarchy: continent, country,
city, street. You then pick the level of that hierarchy that matches how you want the
folders to look, see the result on a map, and commit it.

The application is a single binary. It starts a local web server, opens a browser, and
serves a React interface with a Leaflet map; the frontend is embedded into the executable at
build time, so there is nothing to install or deploy.

## How it works

1. **Scan** — walks the directory tree and reads EXIF from every `jpg`, `jpeg`, `heif`,
   `heic`, `tiff`, `png` or `raw` file. Files without GPS coordinates are skipped. At least
   five geotagged images are required.
2. **Cluster** — runs HDBSCAN over the coordinates using the **haversine distance in
   metres**, so clustering happens in real ground distance rather than in degrees of
   latitude and longitude, which would distort everything away from the equator.
3. **Build a tree** — the condensed cluster tree produced by HDBSCAN is turned into a tree
   of nested groups; each node carries the convex hull of its descendants and the number of
   photographs beneath it.
4. **Browse** — the tree and the hulls are sent to the frontend, which shows them on a map
   alongside a collapsible tree view. Expanding a node splits a group into its subgroups;
   collapsing merges them back. What is expanded at the moment you commit determines the
   folders you get.
5. **Commit** — creates a `categorized/` directory inside the chosen folder and moves each
   cluster into its own subfolder.

Folders are named from the EXIF capture dates of their contents — `2023-07-14` for a single
day, `2023-07-14_2023-07-19` for a range, `unknown` when no date is readable — followed by
six hex digits derived from the cluster contents, so that two folders with the same date
range never collide. Within a folder, files are sorted by capture time and renamed
`0.jpg`, `1.jpg` and so on.

> ⚠️ **Committing moves the original files.** There is no undo. Run with the dry-run option
> first — it prints every directory it would create and every file it would move, and checks
> write permissions, without touching anything.

## Build

The build is containerised, so nothing beyond Docker is needed on the host. It produces both
a Linux binary and a Windows one, cross-compiled with mingw:

```sh
./build.sh
```

To build on the host directly — requires the Rust toolchain, npm, and
`gcc-mingw-w64-x86-64` plus the `x86_64-pc-windows-gnu` target for the Windows artefact:

```sh
./build_local.sh
```

The frontend must be built before the Rust crate, because `rust-embed` reads
`src/ui/build` at compile time. Both scripts already do this in the right order.

## Run

```sh
./ImagesOrganizer
```

The server binds to `127.0.0.1:3000` and a browser window opens automatically. Paste the
absolute path of the directory you want to organise into the interface.

## Layout

| Path | Contents |
|---|---|
| `src/extract_filepath_location.rs` | EXIF GPS extraction, degrees/minutes/seconds → decimal |
| `src/haversine_metric.rs` | Haversine distance as a `petal_neighbors` metric |
| `src/launch_pipeline_for_directory.rs` | The scan → cluster → tree pipeline |
| `src/build_tree.rs` | HDBSCAN condensed tree → nested groups |
| `src/convert_tree.rs` | Convex hull per node |
| `src/to_serde_tree.rs`, `src/types.rs` | Wire format for the frontend |
| `src/reorganize.rs` | Folder naming, dry run, moving files |
| `src/static_assets.rs` | Frontend embedded via `rust-embed` |
| `src/ui/` | React + TypeScript frontend — MUI, Leaflet map, tree view |
| `docker/` | Build image: Rust, npm, mingw |

## Notes

Clustering uses a [fork of `petal-clustering`](https://github.com/marcinbugaj/petal-clustering)
pinned by revision. The upstream crate returns flat clusters; the fork exposes the
**condensed tree**, which is what makes the hierarchy — and therefore the choice of
granularity — possible at all.

HDBSCAN runs with `min_samples = 2` and `min_cluster_size = 4`. Small groups of photographs
taken close together are common and worth keeping, so the thresholds are deliberately low.
