#!/bin/bash

uv run builder.py > ../../site/index.html

uv run -- djlint --profile=jinja --reformat ../../site/index.html

