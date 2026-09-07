import json
from pathlib import Path

from jinja2 import Environment, PackageLoader, Template


def site_builder():

    env = Environment(loader=PackageLoader("ssg", "templates"))
    template = env.get_template("template.html")

    DATA_FILE = Path(__file__).resolve().parents[2] / "data" / "tournaments.json"

    tournamentsJson = json.loads(DATA_FILE.read_text(encoding="utf-8"))

    print(template.render(**tournamentsJson))


if __name__ == "__main__":
    site_builder()
