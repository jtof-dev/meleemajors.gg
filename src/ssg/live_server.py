from pathlib import Path

from livereload import Server

from ssg import site_builder

THIS_DIR = Path(__file__).resolve().parent  # ./ssg/
SITE_DIR = THIS_DIR.parents[1] / "site"  # ../../site/
TEMPLATES_DIR = THIS_DIR / "templates"


def main():
    # print("THIS_DIR:", THIS_DIR)
    # print("other:", SITE_DIR, "exists:", SITE_DIR.exists())

    server = Server()

    server.watch(str(SITE_DIR / "*.html"))
    server.watch(str(SITE_DIR / "*.js"))
    server.watch(str(SITE_DIR / "*.css"))

    server.watch(str(TEMPLATES_DIR / "template.html"), site_builder)

    server.serve(root=str(SITE_DIR), port=8080)
