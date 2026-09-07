import json
import shutil
import subprocess
import tempfile
from pathlib import Path

from jinja2 import Environment, PackageLoader, Template

# using .resolve() forces an absolute path to where this script is located. using that, we can safely write relative paths
BUILDERS_DIR = Path(__file__).resolve().parent  # ./builders/
DATA_FILE = (
    BUILDERS_DIR.parents[1] / "data" / "tournaments.json"
)  # ../../data/tournaments.json
PROJECT_ROOT = BUILDERS_DIR.parents[2]  # ./meleemajors.gg/
DESTINATION_FILE = PROJECT_ROOT / "site" / "index.html"


def site_builder():

    env = Environment(loader=PackageLoader("ssg", "templates"))
    template = env.get_template("template.html")

    tournamentsJson = json.loads(DATA_FILE.read_text(encoding="utf-8"))

    # print(template.render(**tournamentsJson))
    generatedHtml = template.render(**tournamentsJson)

    lint_and_deploy(generatedHtml)


def lint_and_deploy(generatedHtml):

    # print("BUILDERS_DIR:", BUILDERS_DIR)
    # print("DATA_FILE:", DATA_FILE, "exists:", DATA_FILE.exists())
    # print(
    #     "DESTINATION parent:",
    #     DESTINATION_FILE.parent,
    #     "exists:",
    #     DESTINATION_FILE.parent.exists(),
    # )

    # makes a tempfolder in host os's tmp/ dir, so that we can use any file name safely
    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp) / "index.html"
        tmp_path.write_text(generatedHtml, encoding="utf-8")

        subprocess.run(
            [
                "uv",
                "run",
                "--",
                "djlint",
                "--reformat",
                str(tmp_path),
            ],
            capture_output=True,
            text=True,
        )

        shutil.move(str(tmp_path), DESTINATION_FILE)


if __name__ == "__main__":
    site_builder()
