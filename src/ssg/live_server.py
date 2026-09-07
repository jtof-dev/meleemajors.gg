from livereload import Server

from ssg import site_builder


def main():
    server = Server()

    server.watch("../../site/*.html")
    server.watch("../../site/*.js")
    server.watch("../../site/*.css")

    server.watch("./templates/template.html", site_builder)

    server.serve(root="../../site/", port=8080)
