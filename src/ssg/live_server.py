from pathlib import Path

from livereload import Server

from ssg import generate_site, print_gradient

THIS_DIR = Path(__file__).resolve().parent  # ./ssg/
SITE_DIR = THIS_DIR.parents[1] / "site"  # ../../site/
TEMPLATES_DIR = THIS_DIR / "templates"


def main():

    server = Server()
    print_gradient("liveserver running at https://127.0.0.1:8080!")

    server.watch(str(SITE_DIR / "*.html"))
    server.watch(str(SITE_DIR / "*.js"))
    server.watch(str(SITE_DIR / "*.css"))

    server.watch(str(TEMPLATES_DIR / "template.html"), generate_site)

    server.serve(root=str(SITE_DIR), port=8080)


if __name__ == "__main__":
    main()
