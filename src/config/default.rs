// Simply contains the default YAML config for generation and consumption
pub const DEFAULT_CONFIG: &str = r##"
# Regexes which if matched by a file path will always be excluded from
# getting a license header
excludes:
  - \.gitignore
  - .*lock
  - \.git/.*
  - \.licensure\.yml
  - README.*
  - LICENSE.*
  - .*\.(md|rst|txt)
# Definition of the licenses used on this project and to what files
# they should apply.
#
# No default license configuration is provided. This section must be
# configured by the user.
licenses:
  # Either a regex or the string "any" to determine to what files this
  # license should apply. It is common for projects to have files
  # under multiple licenses or with multiple copyright holders. This
  # provides the ability to automatically license files correctly
  # based on their file paths.
  #
  # If "any" is provided all files will match this license.
  # - files: any
  #
  #   The license identifier, a list of common identifiers can be
  #   found at: https://spdx.org/licenses/ but existence of the ident
  #   in this list it is not enforced unless auto_template is set to
  #   true.
  #   ident: MIT
  #
  #   A list of authors who hold copyright over these files
  #   authors:
  #       Provide either your full name or company name for copyright purposes
  #     - name: Your Name Here
  #       Optionally provide email for copyright purposes
  #       email: you@yourdomain.com
  # 
  #   The template that will be rendered to generate the header before
  #   comment characters are applied. Available variables are:
  #    - [year]: substituted with the current year.
  #    - [name of author]: Substituted with name of the author and email
  #      if provided. If email is provided the output appears as Full
  #      Name <email@example.com>. If multiple authors are provided the
  #      list is concatenated together with commas.
  #   template: |
  #     Copyright [year] [name of author]. All rights reserved. Use of
  #     this source code is governed by the [ident] license that can be
  #     found in the LICENSE file.
  #
  #   If auto_template is true then template is ignored and the SPDX
  #   API will be queried with the ident value to automatically
  #   determine the license header template. auto_template works best
  #   with licenses that have a standardLicenseHeader field defined in
  #   their license info JSON, if it is not then we will use the full
  #   licenseText to generate the header which works fine for short
  #   licenses like MIT but can be quite lengthy for other licenses
  #   like BSD-4-Clause. The above default template is valid for most
  #   licenses and is recommended for MIT, and BSD licenses. Common
  #   licenses that work well with the auto_template feature are GPL
  #   variants, and the Apache 2.0 license.
  #
  #   Important Note: this means the ident must be a valid SPDX identifier
  #   auto_template: true
  # 
  #   Try to detect the text wrapping of the template, and unwrap it
  #   unwrap_text: true

# Define type of comment characters to apply based on file extensions.
comments:
  # The extensions (or singular extension) field defines which file
  # extensions to apply the commenter to.
  - extensions:
      - js
      - rs
      - go
    # The commenter field defines the kind of commenter to
    # generate. There are two types of commenters: line and block.
    #
    # This demonstrates a line commenter configuration. A line
    # commenter type will apply the comment_char to the beginning of
    # each line in the license header. It will then apply a number of
    # empty newlines to the end of the header equal to trailing_lines.
    #
    # If trailing_lines is omitted it is assumed to be 0.
    commenter:
      type: line
      comment_char: "//"
      trailing_lines: 0
  - extensions:
      - css
      - cpp
      - c
    # This demonstrates a block commenter configuration. A block
    # commenter type will add start_block_char as the first character
    # in the license header and add end_block_char as the last character
    # in the license header. If per_line_char is provided each line of
    # the header between the block start and end characters will be
    # line commented with the per_line_char
    #
    # trailing_lines works the same for both block and line commenter
    # types
    commenter:
      type: block
      start_block_char: "/*\n"
      end_block_char: "*/"
      per_line_char: "*"
      trailing_lines: 0
  # In this case extension is singular and a single string extension is provided.
  - extension: html
    commenter:
      type: block
      start_block_char: "<!--\n"
      end_block_char: "-->"
  - extensions:
      - el
      - lisp
    commenter:
      type: line
      comment_char: ";;;"
      trailing_lines: 0
  # The extension string "any" is special and so will match any file
  # extensions. Commenter configurations are always checked in the
  # order they are defined, so if any is used it should be the last
  # commenter configuration or else it will override all others.
  #
  # In this configuration if we can't match the file extension we fall
  # back to the popular "#" line comment used in most scripting
  # languages.
  - extension: any
    commenter:
      type: line
      comment_char: "#"
      trailing_lines: 0
    
"##;
