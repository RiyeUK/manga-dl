# manga-dl (Working Title)

This WIP Rust project downloads manga from mangadex in full quality. In a folder structure that is ready to be converted
into epub format for reading using e-readers.

# Disclaimer

Robust rate-limiting has not been handled. This means although some effort has been put into not overloading mangadex's
servers that it is possible to be temporarlly IP banned from Mangadex if using this to download large mangas.

# Usage

The general usage of manga-dl is as follows:

```
manga-dl [OPTIONS] <OUTPUT>
```

## Arguments

- `<OUTPUT>` This is the folder where the downloaded mnaga will be saved. You can use `title` within the folder path,
and it will be replaced with the manga title.

## Options

- `--anilist-id <ANILIST_ID>`: An optional AniList ID to use in conjunction with the manga title, to confirm you get the
correct result when manga-dl searches. This isn't used if an `--id` is provided.

- `-c --chapters <CHAPTERS>`: A range of chapters to download. The format for specifying a range is 1..3 for a range of
chapters from 1 to 3, or 1..=3 for an inclusive range from 1 to 3. You can also just specify a number if you only want
one.

- `-v --volumes <VOLUMES>`: A range of volumes to download. The format for specifying a range is the same as for
chapters. Both Volumes and Chapters are taken into account and are logicaly ORed together.

- `--cover-language <COVER_LANGUAGE>`: The language for the manga covers. Defaults to Japanese.

- `-t, --title <TITLE>:` The title of the manga. This option is required if the ID is not supplied and is used to search
the mangadex api.

- `--translated-language <TRANSLATED_LANGUAGE>:` The language into which the manga should be translated. The default is
English (en).

- `--download-covers:` An optional flag to download manga covers. If specified, manga covers will be downloaded along
with the chapters.

- `--verbose:` An optional flag to enable verbose output for better visibility of the download process. (Currently does
nothing)

- `-h, --help:` Print the help message, displaying the available options and usage information.

- `-V, --version:` Print the version of manga-dl.## Features

# Examples

1. Search for and download a manga by title:

```
manga-dl -t "My Manga" /path/to/save/{title}/
```

2. Download a manga by ID:

```
manga-dl -i 12345678-abcd-9876-wxyz /path/to/save/{title}/
```

3. Download volumes 1 to 5 inclusivly using an anilist-id:

```
manga-dl -t "My Manga" --anilist-id 123456 --volumes 1..=5 /path/to/save/{title}/
```

4. Download the first 10 chapters and download covers:

```
manga-dl -t "My Manga" --chapters ..10 --download-covers /path/to/save/{title}/
```

# Acknowledgments

The manga-dl program uses the MangaDex API to fetch manga data. Many thanks to the MangaDex team for providing this API.

## Proposed features

* Using Anilist.co Manga Reading list
* Better error handling
* Retry Logic
