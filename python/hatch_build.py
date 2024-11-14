# pyright: basic
from pathlib import Path

from grpc_tools import protoc
from hatchling.builders.hooks.plugin.interface import BuildHookInterface


class CustomBuildHook(BuildHookInterface):
    def initialize(self, version, build_data):
        if self.target_name == "sdist":
            return

        args = [
            "grpc_tools.protoc",
            "-Ifricon/_proto=./proto",
            "--python_out=src",
            "--pyi_out=src",
            "--grpc_python_out=src",
        ] + list(map(str, Path("proto").glob("**/*.proto")))
        proto_include = protoc._get_resource_file_name("grpc_tools", "_proto")
        protoc.main(args + ["-I{}".format(proto_include)])

        build_data["artifacts"].append("_proto")
