# CLI Calendar

Just a minimalist calendar manager CLI app. It uses `confy` to manage
the configuration files. The paths are all fully managed through the
configuration file.

You can find a matching schema [in this Gist][link-schema] to make
writing them easier. I purposefully did not code a way to edit them in
the app.

The program only reads YAML files for now. You can enable the schema in
YAML like this
```yml
# yaml-language-server: $schema=../events-schema.json
# ...rest of your YAML file
```

## Commands

The program is called `clindar` which comes from cli + calendar and
because it's weird enough to avoid collisions with other CLI apps. 

You can call help to find out how to use it. The arguments are managed
by clap.

> Note about the `--all` flag vs full filters. To make `calendar` and
> `upcoming` easier to type I simply hardcoded that NOT using the
> `--all` flag filters IN only all the entries which have a flag
> "public". The rest of commands have normal filters.

[link-schema]: https://gist.github.com/SlimRunner/89e9e9c0cc333b0bff3cac8f6c6d1823
