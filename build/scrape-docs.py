#!/usr/bin/env python
import pandas as pd
import textwrap
from typing import Set, Dict
from utils import *
import json
import aiohttp
import asyncio

DOC_PATH = Path(__file__).parent / "docstrings"
MD_WORD_WRAP = 120

class DocumentationFiles:
    def __init__(self, name: str):
        self.body = DOC_PATH / "body" / f"{name}.md"
        self.metadata = DOC_PATH / "metadata" / f"{name}.json"

    def all_exist(self):
        return self.body.exists() and self.metadata.exists()

def _postprocess_doc_paragraph(params: dict, s: str) -> str:
    words = s.split()

    for i, w in enumerate(words):
        if w in params:
            words[i] = f"`{w}`"

    return " ".join(words)


def create_body_file(path: Path, parameters: Dict[str, str], pdata: dict):
    paragraphs = (
        "\n".join(textwrap.wrap(_postprocess_doc_paragraph(parameters, para), MD_WORD_WRAP))
        for para in pdata['doc']
    )
    body = "\n\n".join(paragraphs)
    with open(path, 'w') as fp:
        fp.write(body)
    print("wrote", path)


def create_metadata_file(path: Path, data: dict):
    data = data.copy()
    del data['doc']
    del data['cl_only']
    del data['ty']
    with open(path, 'w') as fp:
        json.dump(data, fp, indent='  ')
    print("wrote", path)

async def create_documentation(session: aiohttp.ClientSession, kind, full_list: Dict[str, str], name: str, args=None):
    if kind == "attribute":
        files = DocumentationFiles(name + "_attr")
    elif kind == "parameter":
        files = DocumentationFiles(name + "_param")
    else:
        raise ValueError("kind must be attribute or parameter")

    write_body = (args is not None and args.overwrite_body) or not files.body.exists()
    write_meta = (args is not None and args.overwrite_meta) or not files.metadata.exists()

    if not (write_body or write_meta):
        return

    if name not in full_list:
        return

    if kind == "attribute":
        data = await fetch_attribute_data(session, name, full_list[name])
    else:
        data = await fetch_parameter_data(session, name, full_list[name])

    if write_body:
        create_body_file(files.body, full_list, data)

    if write_meta:
        create_metadata_file(files.metadata, data)

async def main(args):
    (DOC_PATH / "body").mkdir(exist_ok=True)
    (DOC_PATH / "metadata").mkdir(exist_ok=True)
    param_csv = pd.read_csv("params.csv")
    attr_csv = pd.read_csv("attrs.csv")

    async with http_session() as session:
        attributes = await fetch_attribute_list(session)
        await asyncio.gather(*(
            create_documentation(session, "attribute", attributes, a, args) for a in attr_csv['attr']
        ))
        parameters = await fetch_parameter_list(session)
        await asyncio.gather(*(
            create_documentation(session, "parameter", parameters, p, args) for p in param_csv['param']
        ))


if __name__ == '__main__':
    import argparse
    p = argparse.ArgumentParser()
    p.add_argument("-m", "--overwrite-meta", action='store_true', help="Clobber parameter and attribute metadata.  This is usally fine.")
    p.add_argument("-b", "--overwrite-body", action='store_true', help="Overwrite documentation body (the main text).  THIS WILL DELETE ANY MANUAL EDITS.")
    args = p.parse_args()
    asyncio.run(main(args))