# vedtoob

## Installation

```sh
cargo install --git https://github.com/theodore-s-beers/vedtoob
```

Caveat: [`pandoc`](https://github.com/jgm/pandoc) is a runtime dependency of the `show` subcommand, i.e., it needs to be in your `PATH`.

## Usage

List the titles and slugs of all [Boot.dev](https://www.boot.dev/) courses:

```sh
vedtoob list-courses
```

List the chapters of a given course:

```sh
vedtoob list-chapters --course learn-sql
```

List the lessons in a given course chapter (there are also short forms of these flags):

```sh
vedtoob list-lessons --course learn-sql --chapter 9
```

Show the readme for a given lesson:

```sh
vedtoob show --course learn-sql --chapter 9 --lesson 1
```
