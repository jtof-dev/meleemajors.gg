from .builders.builder import site_builder


def main() -> None:
    site_builder()


__all__ = ["site_builder", "main"]
