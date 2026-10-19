#!/usr/bin/env python3
"""Publish only verified native artifacts; retain immutable version assets."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import xml.etree.ElementTree as ET
from urllib.error import HTTPError
from urllib.request import Request, urlopen

REPOSITORY = "tangxiangong/serial-port"
API = f"https://api.github.com/repos/{REPOSITORY}"


def api(path):
    request = Request(API + path, headers={"Accept": "application/vnd.github+json", "Authorization": f"Bearer {os.environ['GH_TOKEN']}", "User-Agent": "SerialStudio-release"})
    try:
        with urlopen(request, timeout=60) as response:
            return json.load(response)
    except HTTPError as error:
        if error.code == 404:
            return None
        raise


def gh(*arguments):
    subprocess.run(["gh", *arguments, "--repo", REPOSITORY], check=True)


def cargo(*arguments):
    subprocess.run(["cargo", "run", "--locked", "-p", "xtask", "--", *map(str, arguments)], check=True)


def download(asset, destination):
    # Release assets are public. Never forward the API token to a CDN redirect.
    url = asset["browser_download_url"]
    if not url.startswith(f"https://github.com/{REPOSITORY}/releases/download/"):
        raise ValueError("Unexpected release asset URL")
    with urlopen(url, timeout=120) as response, destination.open("wb") as output:
        while block := response.read(1024 * 1024):
            output.write(block)


def checksum(path):
    with path.open("rb") as stream:
        digest = hashlib.sha256()
        while block := stream.read(1024 * 1024):
            digest.update(block)
        return digest.hexdigest()


def upload(tag, path, existing, mutable=False):
    previous = existing.get(path.name)
    if previous is not None:
        digest = previous.get("digest") or ""
        if not digest.startswith("sha256:"):
            with tempfile.TemporaryDirectory() as temporary:
                local = Path(temporary) / "asset"
                download(previous, local)
                digest = "sha256:" + checksum(local)
        if digest == "sha256:" + checksum(path):
            return
        if not mutable:
            raise ValueError(f"Refusing to overwrite immutable release asset: {path.name}")
    arguments = ["release", "upload", tag, str(path)]
    if previous is not None:
        arguments.append("--clobber")
    gh(*arguments)


def main():
    tag, directory = sys.argv[1:]
    if not re.fullmatch(r"v(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)(?:-(?:alpha|beta)\.(?:0|[1-9][0-9]*))?", tag):
        raise ValueError("Unsupported release tag")
    directory = Path(directory).resolve()
    version = tag[1:]
    root = Path(__file__).resolve().parent.parent
    metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--locked", "--no-deps", "--format-version", "1"], cwd=root))
    if any(package["version"] != version for package in metadata["packages"]):
        raise ValueError("Release tag differs from the workspace version")
    project_version = ET.parse(root / "windows/SerialStudio/SerialStudio.csproj").findtext("PropertyGroup/Version")
    mac_version = re.search(r'MARKETING_VERSION:\s*"([^"]+)"', (root / "macos/project.yml").read_text())
    if project_version != version or mac_version is None or mac_version[1] != version:
        raise ValueError("Native project versions differ from the release tag")
    source = root / "release-notes" / version
    # Finish note and signature verification before the first remote write.
    cargo("release-notes", source, directory)
    cargo("sign-windows", directory, tag)
    release = api(f"/releases/tags/{tag}")
    feed_release = api("/releases/tags/update-feed")
    with tempfile.TemporaryDirectory() as temporary:
        feeds = Path(temporary)
        old = {asset["name"]: asset for asset in (feed_release or {}).get("assets", [])}
        metadata = {name: value for name, value in old.items() if name == "channels.json" or name.startswith("appcast-") or name.startswith("releases.win-")}
        if metadata and "channels.json" not in metadata:
            raise ValueError("Existing update feed has no channel history; refusing to reset it")
        for name, asset in metadata.items():
            if Path(name).name != name or "\\" in name:
                raise ValueError("Invalid previous metadata filename")
            download(asset, feeds / name)
        cargo("feeds", directory, feeds, tag)
        assets = sorted(path for path in directory.iterdir() if path.is_file() and path.name != "release-body.md")
        sums = directory / "SHA256SUMS"
        sums.write_text("".join(f"{checksum(path)}  {path.name}\n" for path in assets if path.name != "SHA256SUMS"))
        if sums not in assets:
            assets.append(sums)
        if release is None:
            arguments = ["release", "create", tag, "--title", f"Serial Studio {version}", "--notes-file", str(directory / "release-body.md"), "--draft"]
            if "-" in version:
                arguments.append("--prerelease")
            gh(*arguments)
            release = api(f"/releases/tags/{tag}")
        existing = {asset["name"]: asset for asset in release["assets"]}
        for path in assets:
            upload(tag, path, existing)
        if release.get("draft", False):
            gh("release", "edit", tag, "--draft=false")
        if feed_release is None:
            gh("release", "create", "update-feed", "--title", "Serial Studio update feed", "--notes", "Signed native update metadata. Installers are available in versioned releases.", "--latest=false")
        # Keep payloads available before advancing any client-visible metadata.
        for path in sorted(directory.glob("*-full.nupkg")):
            upload("update-feed", path, old)
        for path in sorted(feeds.iterdir()):
            if path.is_file() and path.name != "channels.json":
                upload("update-feed", path, old, mutable=True)
        upload("update-feed", feeds / "channels.json", old, mutable=True)


if __name__ == "__main__":
    main()
