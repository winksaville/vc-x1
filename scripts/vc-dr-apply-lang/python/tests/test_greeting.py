from {module} import greeting


def test_greeting() -> None:
    assert greeting() == "Hello, world!"
