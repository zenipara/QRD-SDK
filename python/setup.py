from setuptools import setup
from setuptools_rust import Binding, RustExtension

setup(
    name="qrd",
    version="0.1.0",
    rust_extensions=[RustExtension("qrd.qrd", binding=Binding.PyO3)],
    packages=["qrd"],
    zip_safe=False,
)