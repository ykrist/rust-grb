#!/usr/bin/env python
import pandas as pd
from utils import *
import asyncio



async def print_missing(session, kind, full_list, implemented):
    if kind == "attribute":
        row_fmt = "{name},{dtype},{otype}"
        fetch_data = fetch_attribute_data
        ignore = lambda data: data['otype'] in ("batch", "gconstr") or data['cl_only']
    elif kind == "parameter":
        row_fmt = "{name},{dtype}"
        fetch_data = fetch_parameter_data
        ignore = lambda data: data['cl_only']
    else:
        raise ValueError

    extra = {p for p in implemented if p not in full_list}
    missing = await asyncio.gather(*(
        fetch_data(session, p, path)
        for p, path in full_list.items() if p not in implemented
    ))
    missing = { d["name"] : d for d in missing}
    append_csv = []

    for p, data in sorted(missing.items()):
        if ignore(data):
            print("IGNORE", p)
            continue
        print("MISSING", p)
        append_csv.append(row_fmt.format_map(data))

    for p in sorted(extra):
        print("EXTRA", p)

    if append_csv:
        print("\n\n" + f" New {kind.capitalize()}s ".center(100, '-'))
        print("\n".join(append_csv))


async def main():
    params_csv = pd.read_csv("params.csv")
    attr_csv = pd.read_csv("attrs.csv")
    implemented_parameters = set(params_csv['param'])
    implemented_attributes = set(attr_csv['attr'])

    async with http_session() as session:
        parameters, attributes = await asyncio.gather(fetch_parameter_list(session), fetch_attribute_list(session))
        
        await print_missing(session, "parameter", parameters, implemented_parameters)
        await print_missing(session, "attribute", attributes, implemented_attributes)

if __name__ == '__main__':
    asyncio.run(main())