import nox

@nox.session
def full(session):
    """Run the full test suite."""
    session.run("cargo", "test", external=True)

@nox.session
def validate_spec(session):
    """Validate product spec schema."""
    session.install("PyYAML")
    session.run("python3", "-c", "import yaml; yaml.safe_load(open('products/costpilot/product.yml'))")